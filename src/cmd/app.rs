use clap::{crate_version, Arg, Command};

pub const ARG_RECURSIVE: &str = "recursive";
pub const ARG_PATH_1: &str = "path_1";
pub const ARG_PATH_2: &str = "path_2";
pub const ARG_PROFILE: &str = "profile";
pub const ARG_EXCLUDE: &str = "exclude";
pub const ARG_INCLUDE: &str = "include";
pub const ARG_OVER: &str = "over";

pub fn cmd_app() -> Command<'static> {
	Command::new("ss3")
		.version(&crate_version!()[..])
		.arg(arg_profile())
		.subcommand(sub_ls())
		.subcommand(sub_cp())
}

fn sub_ls() -> Command<'static> {
	Command::new("ls")
		.about("List from s3 url")
		.arg(arg_profile())
		.arg(arg_path_1())
		.arg(arg_recursive())
		.arg(
			Arg::new("info")
				.long("info")
				.help("Display the info of the listing at the end of the listing (total files, total size, total size per extension)"),
		)
		.arg(
			Arg::new("info-only")
				.long("info-only")
				.help("Display only info of the listing (total files, total size, total size per extension)"),
		)
}

fn sub_cp() -> Command<'static> {
	Command::new("cp")
		.about("Copy from s3 url / file path to s3 url / file path")
		.arg(arg_profile())
		.arg(arg_path_1())
		.arg(arg_path_2())
		.arg(arg_include())
		.arg(arg_exlude())
		.arg(arg_recursive())
		.arg(
			Arg::new(ARG_OVER)
				.long("over")
				.takes_value(true)
				.help("Overwrite mode. Default 'skip'. Can be 'skip', 'write', 'fail'"),
		)
}

// region:    --- Common Args
fn arg_path_1() -> Arg<'static> {
	Arg::new(ARG_PATH_1).required(true).help("The first path to apply the action from.")
}

fn arg_path_2() -> Arg<'static> {
	Arg::new(ARG_PATH_2).required(true).help("The destination path.")
}

fn arg_recursive() -> Arg<'static> {
	Arg::new(ARG_RECURSIVE).short('r').help("Specify to list all keys recursively")
}

fn arg_profile() -> Arg<'static> {
	Arg::new(ARG_PROFILE)
		.short('p')
		.long("profile")
		.takes_value(true)
		.help("The profile to use if no bucket environment credentials.")
}
// endregion: --- Common Args

// region:    --- cp Args
fn arg_exlude() -> Arg<'static> {
	Arg::new(ARG_EXCLUDE)
		.short('e')
		.long("exclude")
		.takes_value(true)
		.multiple_occurrences(true)
		.help("Exclude the items that match the glob expression.")
}

fn arg_include() -> Arg<'static> {
	Arg::new(ARG_INCLUDE)
		.short('i')
		.long("include")
		.takes_value(true)
		.multiple_occurrences(true)
		.help("Only process the item that match the glob expression.")
}
// endregion: --- cp Args
