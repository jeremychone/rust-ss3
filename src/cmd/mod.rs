// region:    --- Modules

// -- Sub-modules
mod app;

// -- Imports
use crate::cmd::app::{cmd_app, ARG_REGION};
use crate::s3w::{
	create_bucket, delete_bucket, get_sbucket, list_buckets, new_s3_client, CpOptions, ListInfo, ListOptions, ListResult, OverMode,
	RegionProfile,
};
use crate::spath::{S3Url, SPath};
use crate::{s, Error, Result, CT_HTML, CT_TEXT};
use app::{ARG_NOEXT_CT, ARG_OVER, ARG_PATH_1, ARG_PATH_2, ARG_PROFILE, ARG_RECURSIVE};
use clap::ArgMatches;
use file_size::fit_4;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashMap;

// endregion: --- Modules

pub async fn cmd_run() -> Result<()> {
	let argm = cmd_app().get_matches();

	// get the dir from the root command or sub command
	let profile = argm
		.get_one::<String>(ARG_PROFILE.0)
		.or_else(|| match &argm.subcommand() {
			Some((_, sub)) => sub.get_one::<String>(ARG_PROFILE.0),
			_ => None,
		})
		.map(String::from);

	let region = argm
		.get_one::<String>(ARG_REGION)
		.or_else(|| match &argm.subcommand() {
			Some((_, sub)) => sub.get_one::<String>(ARG_REGION),
			_ => None,
		})
		.map(String::from);

	let reg_pro = RegionProfile { region, profile };

	match argm.subcommand() {
		Some(("ls", sub_cmd)) => exec_ls(reg_pro, sub_cmd).await?,
		Some(("cp", sub_cmd)) => exec_cp(reg_pro, sub_cmd).await?,
		Some(("rm", sub_cmd)) => exec_rm(reg_pro, sub_cmd).await?,
		Some(("mb", sub_cmd)) => exec_mb(reg_pro, sub_cmd).await?,
		Some(("rb", sub_cmd)) => exec_rb(reg_pro, sub_cmd).await?,
		_ => {
			cmd_app().print_long_help()?;
			println!("\n");
		}
	}

	Ok(())
}

pub async fn exec_ls(reg_pro: RegionProfile, argm: &ArgMatches) -> Result<()> {
	let s3_url = argm
		.get_one::<String>(ARG_PATH_1)
		.ok_or(Error::CmdInvalid("This command requires a S3 url or file path"))?;

	if s3_url == "s3://" {
		exec_ls_buckets(reg_pro).await?;
	} else {
		exec_ls_objects(reg_pro, SPath::from_str(s3_url)?, argm).await?;
	}

	Ok(())
}

async fn exec_ls_buckets(reg_pro: RegionProfile) -> Result<()> {
	let client = new_s3_client(reg_pro, None).await?;
	let buckets = list_buckets(&client).await?;
	for bucket in buckets {
		println!("{bucket}");
	}
	Ok(())
}

async fn exec_ls_objects(reg_pro: RegionProfile, spath: SPath, argm: &ArgMatches) -> Result<()> {
	let s3_url = match spath {
		SPath::S3(s3_url) => s3_url,
		SPath::File(_) => return Err(Error::CmdInvalid("The 'ls' command requires a S3 url.")),
	};

	// build the bucket
	let bucket = get_sbucket(reg_pro, s3_url.bucket()).await?;

	// build the option (take ownership of the continuation_token)
	let mut options = ListOptions::from_argm(argm)?;

	let mut total_objects: i64 = 0;
	let mut total_size: i64 = 0;
	type Size = i64;
	type Count = i64;
	let mut size_per_ext: HashMap<String, (Size, Count)> = HashMap::new();

	// next continuation token (starts with none for the first request)
	let mut continuation_token: Option<String> = None;
	let show_list = matches!(options.info, None | Some(ListInfo::WithInfo));

	while {
		options.continuation_token = continuation_token;

		// -- Execute the bucket.list command
		let ListResult {
			prefixes,
			objects,
			next_continuation_token,
		} = bucket.list(s3_url.key(), &options).await?;

		// -- Do the prints
		// Print prefixes (dirs) first
		// Note: When recursive, the list of prefixes is not given by the aws sdk
		for item in prefixes.iter() {
			println!("{}", item.key);
		}

		// -- Print objects
		for item in objects.iter() {
			total_objects += 1;
			total_size += item.size;
			if let Some(ext_idx) = item.key.rfind('.') {
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
		continuation_token.is_some() // will continue the while loop if not none
	} {} // this is the way to do `do while` in rust

	if let Some(ListInfo::InfoOnly | ListInfo::WithInfo) = options.info {
		println!("\n--- Info:");
		let mut exts: Vec<(&String, &(Size, Count))> = size_per_ext.iter().map(|e| (e.0, e.1)).collect();
		exts.sort_by(|a, b| a.0.cmp(b.0));
		for (ext, (size, count)) in exts.into_iter() {
			println!("{ext:<5} - size: {:<5} count: {count} ", fit_4(*size as u64))
		}

		println!();
		let total_size_fit_4 = fit_4(total_size as u64);
		println!("total size: {total_size_fit_4:5} total count: {total_objects} ");
	}

	Ok(())
}

pub async fn exec_mb(reg_pro: RegionProfile, argm: &ArgMatches) -> Result<()> {
	let s3_url = get_s3_url_1(argm)?;
	let bucket_name = s3_url.bucket();

	let client = new_s3_client(reg_pro, Some(bucket_name)).await?;
	let bucket_created = create_bucket(&client, bucket_name).await?;
	if let Some(bucket_created) = bucket_created {
		println!("Bucket Created: {bucket_created}");
	}

	Ok(())
}

pub async fn exec_rb(reg_pro: RegionProfile, argm: &ArgMatches) -> Result<()> {
	let s3_url = get_s3_url_1(argm)?;
	let bucket_name = s3_url.bucket();

	let client = new_s3_client(reg_pro, Some(bucket_name)).await?;
	delete_bucket(&client, bucket_name).await?;
	println!("Bucket Deleted: {bucket_name}");

	Ok(())
}

pub async fn exec_cp(reg_pro: RegionProfile, argm: &ArgMatches) -> Result<()> {
	let url_1 = get_path_1(argm)?;
	let url_2 = get_path_2(argm)?;

	let opts = CpOptions::from_argm(argm);

	match (url_1, url_2) {
		// DOWNLOAD
		(SPath::S3(src_s3), SPath::File(dst_path)) => {
			// build the bucket
			let src_bucket = get_sbucket(reg_pro, src_s3.bucket()).await?;
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
			let dst_bucket = get_sbucket(reg_pro, dst_s3.bucket()).await?;
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

pub async fn exec_rm(reg_pro: RegionProfile, argm: &ArgMatches) -> Result<()> {
	let s3_url = get_s3_url_1(argm)?;

	let bucket = get_sbucket(reg_pro, s3_url.bucket()).await?;

	bucket.delete_object(s3_url.key()).await?;
	println!("Object Deleted: {s3_url}");

	Ok(())
}
// region:    Args Utils
fn get_s3_url_1(argm: &ArgMatches) -> Result<S3Url> {
	let path = argm
		.get_one::<String>(ARG_PATH_1)
		.ok_or(Error::CmdInvalid("This command requires a S3 url"))?;

	let spath = SPath::from_str(path)?;

	let SPath::S3(s3_url) = spath else {
		return Err(Error::NotValidS3Url(path.to_string()));
	};

	Ok(s3_url)
}

fn get_path_1(argm: &ArgMatches) -> Result<SPath> {
	let path = argm
		.get_one::<String>(ARG_PATH_1)
		.ok_or(Error::CmdInvalid("This command requires a S3 url or file path"))?;

	SPath::from_str(path)
}

fn get_path_2(argm: &ArgMatches) -> Result<SPath> {
	let path = argm
		.get_one::<String>(ARG_PATH_2)
		.ok_or(Error::CmdInvalid("This command require a second S3 url or file path"))?;

	SPath::from_str(path)
}
// endregion: Args Utils

// region:    --- ListOptions Builder
impl ListOptions {
	fn from_argm(argm: &ArgMatches) -> Result<ListOptions> {
		let recursive = argm.get_flag(ARG_RECURSIVE.0);
		let info = match (argm.get_flag("info"), argm.get_flag("info-only")) {
			// --info
			(true, false) => Ok(Some(ListInfo::WithInfo)),
			// --info-only
			(false, true) => Ok(Some(ListInfo::InfoOnly)),
			// no info
			(false, false) => Ok(None),
			// both, error!
			(true, true) => Err(Error::ComamndInvalid("Cannot have '--info' and '--info-only' at the same time")),
		}?;

		let excludes = build_glob_set(argm, "exclude");
		let includes = build_glob_set(argm, "include");

		Ok(ListOptions {
			recursive,
			includes,
			excludes,
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
		let recursive = argm.get_flag(ARG_RECURSIVE.0);

		// extract the eventual strings
		let excludes = build_glob_set(argm, "exclude");
		let includes = build_glob_set(argm, "include");

		// extract the over mode
		let over = match argm.get_one::<String>(ARG_OVER).map(|v| v.as_str()) {
			Some("write") => OverMode::Write,
			Some("skip") => OverMode::Skip,
			Some("fail") => OverMode::Fail,
			Some(other) => panic!("Invalid over mode {}. Must be 'write', 'skip', 'fail'", other),
			None => OverMode::default(),
		};

		let noext_ct = argm.get_one::<String>(ARG_NOEXT_CT).map(|v| match v.as_str() {
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
	let globs = argm.get_many::<String>(name).map(|vals| vals.collect::<Vec<_>>());
	globs.map(|globs| {
		let mut builder = GlobSetBuilder::new();
		for glob in globs {
			builder.add(Glob::new(glob).unwrap());
		}
		builder.build().unwrap()
	})
}
// endregion: --- CpOptions Builder
