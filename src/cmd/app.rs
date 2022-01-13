use clap::{crate_version, App, Arg};

pub const ARG_URL_1: &str = "url_1";
pub const ARG_PROFILE: &str = "profile";

pub fn cmd_app() -> App<'static> {
	App::new("ss3")
		.version(&crate_version!()[..])
		.arg(arg_profile())
		.subcommand(sub_ls())
}

fn sub_ls() -> App<'static> {
	App::new("ls").arg(arg_profile()).arg(arg_url_1())
}

// region:    Common Args
fn arg_url_1() -> Arg<'static> {
	Arg::new(ARG_URL_1).required(true)
}

fn arg_profile() -> Arg<'static> {
	Arg::new(ARG_PROFILE).short('p').takes_value(true).help("The profile to be used")
}
// endregion: Common Args
