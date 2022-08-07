use std::collections::HashMap;

use self::app::{ARG_NOEXT_CT, ARG_OVER, ARG_PATH_1, ARG_PATH_2, ARG_RECURSIVE};
use crate::cmd::app::cmd_app;
use crate::s3w::{get_sbucket, CpOptions, ListInfo, ListOptions, ListResult, OverMode};
use crate::spath::SPath;
use crate::{s, Error, CT_HTML, CT_TEXT};
use clap::ArgMatches;
use file_size::fit_4;
use globset::{Glob, GlobSet, GlobSetBuilder};

mod app;

pub async fn cmd_run() -> Result<(), Error> {
	let argm = cmd_app().get_matches();

	// get the dir from the root command or sub command
	let profile = argm.value_of("profile").or_else(|| match &argm.subcommand() {
		Some((_, sub)) => sub.value_of("profile"),
		_ => None,
	});

	match argm.subcommand() {
		Some(("ls", sub_cmd)) => exec_ls(profile, sub_cmd).await?,
		Some(("cp", sub_cmd)) => exec_cp(profile, sub_cmd).await?,
		_ => {
			cmd_app().print_long_help()?;
			println!("\n");
		}
	}

	Ok(())
}

pub async fn exec_ls(profile: Option<&str>, argm: &ArgMatches) -> Result<(), Error> {
	let url_1 = get_path_1(argm)?;

	let s3_url = match url_1 {
		SPath::S3(s3_url) => s3_url,
		SPath::File(_) => return Err(Error::CmdInvalid("The 'ls' command requires a S3 url")),
	};

	// build the bucket
	let bucket = get_sbucket(profile, s3_url.bucket()).await?;

	// build the option (take ownership of the continuation_token)
	let mut options = ListOptions::from_argm(argm)?;

	let mut total_objects: i64 = 0;
	let mut total_size: i64 = 0;
	type Size = i64;
	type Count = i64;
	let mut size_per_ext: HashMap<String, (Size, Count)> = HashMap::new();

	// next continuation token (starts with none for the first request)
	let mut continuation_token: Option<String> = None;
	let show_list = match options.info {
		None | Some(ListInfo::WithInfo) => true,
		_ => false,
	};

	while {
		options.continuation_token = continuation_token;

		// execute the list
		let ListResult {
			prefixes,
			objects,
			next_continuation_token,
		} = bucket.list(s3_url.key(), &options).await?;

		// -- Do the prints
		// Print prefixes (dirs) first
		for item in prefixes.iter() {
			println!("{}", item.key);
		}

		// Print objects
		for item in objects.iter() {
			total_objects += 1;
			total_size += item.size;
			if let Some(ext_idx) = item.key.rfind(".") {
				let ext = &item.key[ext_idx..];
				let val = size_per_ext.entry(ext.to_string()).or_insert((0, 0));
				val.0 += item.size;
				val.1 += 1;
			}

			if show_list {
				println!("{}", item.key);
			}
		}

		// -- Condition to continue
		continuation_token = next_continuation_token;
		continuation_token.is_some()
	} {} // this is the way to do `do while` in rust

	if let Some(ListInfo::InfoOnly | ListInfo::WithInfo) = options.info {
		println!("\n--- Info:");
		let mut exts: Vec<(&String, &(Size, Count))> = size_per_ext.iter().map(|e| (e.0, e.1)).collect();
		exts.sort_by(|a, b| a.0.cmp(b.0));
		for (ext, (size, count)) in exts.into_iter() {
			println!("{ext:<5} - size: {:<5} count: {count} ", fit_4(*size as u64))
		}

		println!("");
		let total_size_fit_4 = fit_4(total_size as u64);
		println!("total size: {total_size_fit_4:5} total count: {total_objects} ");
	}

	Ok(())
}

pub async fn exec_cp(profile: Option<&str>, argm: &ArgMatches) -> Result<(), Error> {
	let url_1 = get_path_1(argm)?;
	let url_2 = get_path_2(argm)?;

	let opts = CpOptions::from_argm(argm);

	match (url_1, url_2) {
		// DOWNLOAD
		(SPath::S3(src_s3), SPath::File(dst_path)) => {
			// build the bucket
			let src_bucket = get_sbucket(profile, src_s3.bucket()).await?;
			// perform the copy
			src_bucket.download_path(src_s3.key(), &dst_path, opts).await?;
		}

		// UPLOAD
		(SPath::File(src_path), SPath::S3(dst_s3)) => {
			// fail if src path does not exist
			if !src_path.exists() {
				return Err(Error::FilePathNotFound(src_path.display().to_string()));
			}

			// get the destination sbucket
			let dst_bucket = get_sbucket(profile, dst_s3.bucket()).await?;
			// perform the copy
			dst_bucket.upload_path(&src_path, dst_s3.key(), opts).await?;
		}
		// UNSUPPORTED - for now, s3<->s3 or file<->file
		(url_1, url_2) => {
			println!("NOT SUPPORTED - from {:?} to {:?} not supported", url_1, url_2);
		}
	}

	Ok(())
}

// region:    Args Utils
fn get_path_1(argm: &ArgMatches) -> Result<SPath, Error> {
	let path = argm
		.value_of(ARG_PATH_1)
		.ok_or(Error::CmdInvalid("This command requires a S3 url or file path"))?;

	Ok(SPath::from_str(path)?)
}

fn get_path_2(argm: &ArgMatches) -> Result<SPath, Error> {
	let path = argm
		.value_of(ARG_PATH_2)
		.ok_or(Error::CmdInvalid("This command require a second S3 url or file path"))?;

	Ok(SPath::from_str(path)?)
}
// endregion: Args Utils

// region:    --- ListOptions Builder
impl ListOptions {
	fn from_argm(argm: &ArgMatches) -> Result<ListOptions, Error> {
		let recursive = argm.is_present(ARG_RECURSIVE);
		let info = match (argm.is_present("info"), argm.is_present("info-only")) {
			// --info
			(true, false) => Ok(Some(ListInfo::WithInfo)),
			// --info-only
			(false, true) => Ok(Some(ListInfo::InfoOnly)),
			// no info
			(false, false) => Ok(None),
			// both, error!
			(true, true) => Err(Error::ComamndInvalid("Cannot have '--info' and '--info-only' at the same time")),
		}?;

		Ok(ListOptions {
			recursive,
			info,
			..Default::default()
		})
	}
}
// endregion: --- ListOptions Builder

// region:    --- CpOptions Builder
impl CpOptions {
	fn from_argm(argm: &ArgMatches) -> CpOptions {
		// extract recursive flag
		let recursive = argm.is_present(ARG_RECURSIVE);

		// extract the eventual strings
		let excludes = build_glob_set(argm, "exclude");
		let includes = build_glob_set(argm, "include");

		// extract the over mode
		let over = match argm.value_of(ARG_OVER) {
			Some("write") => OverMode::Write,
			Some("skip") => OverMode::Skip,
			Some("fail") => OverMode::Fail,
			Some(other) => panic!("Invalid over mode {}. Must be 'write', 'skip', 'fail'", other),
			None => OverMode::default(),
		};

		let noext_ct = argm.value_of(ARG_NOEXT_CT).map(|v| match v {
			"html" => s!(CT_HTML),
			"text" => s!(CT_TEXT),
			_ => s!(v),
		});

		// build the options
		CpOptions {
			recursive,
			excludes,
			includes,
			over,
			noext_ct,
		}
	}
}

fn build_glob_set(argm: &ArgMatches, name: &str) -> Option<GlobSet> {
	let globs: Option<Vec<&str>> = argm.values_of(name).map(|vs| vs.map(|v| v).collect());
	globs.map(|globs| {
		let mut builder = GlobSetBuilder::new();
		for glob in globs {
			builder.add(Glob::new(glob).unwrap());
		}
		builder.build().unwrap()
	})
}
// endregion: --- CpOptions Builder
