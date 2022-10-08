use clap::{crate_version, Arg, ArgAction, Command};

pub const ARG_RECURSIVE: &str = "recursive";
pub const ARG_PATH_1: &str = "path_1";
pub const ARG_PATH_2: &str = "path_2";
pub const ARG_PROFILE: &str = "profile";
pub const ARG_EXCLUDE: &str = "exclude";
pub const ARG_INCLUDE: &str = "include";
pub const ARG_NOEXT_CT: &str = "noext-ct";
pub const ARG_OVER: &str = "over";

pub fn cmd_app() -> Command {
	Command::new("ss3")
		.version(crate_version!())
		.arg(arg_profile())
		.subcommand(sub_ls())
		.subcommand(sub_cp())
}

fn sub_ls() -> Command {
	Command::new("ls")
		.about("List from s3 url")
		.arg(arg_profile())
		.arg(arg_path_1())
		.arg(arg_recursive())
		.arg(
			Arg::new("info")
				.action(ArgAction::SetTrue)
				.long("info")
				.help("Display the info of the listing at the end of the listing (total files, total size, total size per extension)"),
		)
		.arg(
			Arg::new("info-only")
				.action(ArgAction::SetTrue)
				.long("info-only")
				.help("Display only info of the listing (total files, total size, total size per extension)"),
		)
}

fn sub_cp() -> Command {
	Command::new("cp")
		.about("Copy from s3 url / file path to s3 url / file path")
		.arg(arg_profile())
		.arg(arg_path_1())
		.arg(arg_path_2())
		.arg(arg_include())
		.arg(arg_exlude())
		.arg(arg_recursive())
		.arg(arg_noext_ct())
		.arg(
			Arg::new(ARG_OVER)
				.long("over")
				.num_args(1)
				.help("Overwrite mode. Default 'skip'. Can be 'skip', 'write', 'fail'"),
		)
}

// region:    --- Common Args
fn arg_path_1() -> Arg {
	Arg::new(ARG_PATH_1)
		.num_args(1)
		.required(true)
		.help("The first path to apply the action from.")
}

fn arg_path_2() -> Arg {
	Arg::new(ARG_PATH_2).num_args(1).required(true).help("The destination path.")
}

fn arg_recursive() -> Arg {
	Arg::new(ARG_RECURSIVE)
		.num_args(0)
		.action(ArgAction::SetTrue)
		.short('r')
		.help("Specify to list all keys recursively")
}

fn arg_profile() -> Arg {
	Arg::new(ARG_PROFILE)
		.required(false)
		.num_args(1)
		.short('p')
		.long("profile")
		.help("The profile to use if no bucket environment credentials.")
}
// endregion: --- Common Args

// region:    --- cp Args
fn arg_exlude() -> Arg {
	Arg::new(ARG_EXCLUDE)
		.num_args(1)
		.short('e')
		.long("exclude")
		.action(ArgAction::Append)
		.help("Exclude the items that match the glob expression.")
}

fn arg_include() -> Arg {
	Arg::new(ARG_INCLUDE)
		.num_args(1)
		.short('i')
		.long(ARG_INCLUDE)
		.action(ArgAction::Append)
		.help("Only process the item that match the glob expression.")
}

fn arg_noext_ct() -> Arg {
	Arg::new(ARG_NOEXT_CT)
		.num_args(1)
		.long(ARG_NOEXT_CT)
		.action(ArgAction::Append)
		.help("Content-Type when no file extension. e.g., --noext-ct 'html' (alias for 'text/html; charset=UTF-8')")
}

// endregion: --- cp Args
