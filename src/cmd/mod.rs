use self::app::{ARG_PATH_1, ARG_PATH_2, ARG_RECURSIVE};
use crate::cmd::app::cmd_app;
use crate::s3w::{get_sbucket, CpOptions, ListOptions, ListResult};
use crate::spath::SPath;
use crate::Error;
use clap::ArgMatches;
use globset::{Glob, GlobSetBuilder};

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
	// build the list options
	let recursive = argm.is_present(ARG_RECURSIVE);
	let options = ListOptions::new(recursive);

	// execute the list
	let ListResult { prefixes, objects } = bucket.list(s3_url.key(), &options).await?;

	// Print prefixes (dirs) first
	for item in prefixes.iter() {
		println!("{}", item.key);
	}
	// Print objects
	for item in objects.iter() {
		println!("{}", item.key);
	}

	Ok(())
}

pub async fn exec_cp(profile: Option<&str>, argm: &ArgMatches) -> Result<(), Error> {
	let url_1 = get_path_1(argm)?;
	let url_2 = get_path_2(argm)?;

	let opts = CpOptions::from_args(argm);

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

// region:    --- CpOptions Builder
impl CpOptions {
	fn from_args(argm: &ArgMatches) -> CpOptions {
		// build the list options
		let recursive = argm.is_present(ARG_RECURSIVE);

		// extract the eventual strings
		let globs: Option<Vec<&str>> = argm.values_of("exclude").map(|vs| vs.map(|v| v).collect());
		let globs = globs.map(|globs| {
			let mut builder = GlobSetBuilder::new();
			for glob in globs {
				builder.add(Glob::new(glob).unwrap());
			}
			builder.build().unwrap()
		});

		// build the options
		CpOptions {
			recursive,
			excludes: globs,
		}
	}
}
// endregion: --- CpOptions Builder
