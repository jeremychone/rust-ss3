use crate::cmd::app::cmd_app;
use crate::s3w::get_s3_bucket;
use crate::spath::SPath;
use crate::Error;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Credentials, Region};
use aws_types::credentials::{ProvideCredentials, SharedCredentialsProvider};
use clap::ArgMatches;

use self::app::ARG_URL_1;

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
		_ => {
			cmd_app().print_long_help()?;
			println!("\n");
		}
	}

	Ok(())
}

pub async fn exec_ls(profile: Option<&str>, argm: &ArgMatches) -> Result<(), Error> {
	let url_1 = get_url_1(argm)?;

	let s3_url = match url_1 {
		SPath::S3(s3_url) => s3_url,
		SPath::File(_) => return Err(Error::CmdInvalid("This command require a S3 url")),
	};

	let bucket = get_s3_bucket(profile, s3_url.bucket()).await?;
	let objects = bucket.list().await?;

	for obj in objects.iter() {
		println!("->> {}", obj.key);
	}

	Ok(())
}

// region:    Args Utils
fn get_url_1(argm: &ArgMatches) -> Result<SPath, Error> {
	let path = argm
		.value_of(ARG_URL_1)
		.ok_or(Error::CmdInvalid("This command require a S3 url or file path"))?;

	Ok(SPath::from_str(path)?)
}
// endregion: Args Utils
