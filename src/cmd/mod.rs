use self::app::{ARG_PATH_1, ARG_PATH_2, ARG_RECURSIVE};
use crate::cmd::app::cmd_app;
use crate::s3w::{get_sbucket, ListOptions, UploadOptions};
use crate::spath::SPath;
use crate::Error;
use clap::ArgMatches;

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
	let options = ListOptions::new(recursive, s3_url.key());

	// execute the list
	let items = bucket.list(&options).await?;

	// TODO - fix print
	for obj in items.iter() {
		println!("{}", obj.key);
	}

	Ok(())
}

pub async fn exec_cp(profile: Option<&str>, argm: &ArgMatches) -> Result<(), Error> {
	// build the list options
	let recursive = argm.is_present(ARG_RECURSIVE);

	let url_1 = get_path_1(argm)?;
	let url_2 = get_path_2(argm)?;

	match (url_1, url_2) {
		(SPath::S3(src_s3), SPath::File(dst_file)) => {
			// build the bucket
			let src_bucket = get_sbucket(profile, src_s3.bucket()).await?;
			println!("->> will copy from {} to {}", src_s3, dst_file.display());
		}

		// upload a file or dir to a s3 url
		(SPath::File(src_file), SPath::S3(dst_s3)) => {
			let upload_options = UploadOptions {
				recursive,
				..UploadOptions::default()
			};
			if !src_file.exists() {
				return Err(Error::FilePathNotFound(src_file.display().to_string()));
			}
			let dst_bucket = get_sbucket(profile, dst_s3.bucket()).await?;
			dst_bucket.upload_path(&src_file, dst_s3.key(), upload_options).await?;
		}
		(url_1, url_2) => {
			println!("->> not supported yet from {:?} to {:?}", url_1, url_2);
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
