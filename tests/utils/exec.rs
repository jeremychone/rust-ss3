use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const ENV_CREDS: [(&str, &str); 6] = [
	("SS3_BUCKET_my_bucket_KEY_ID", "minio"),
	("SS3_BUCKET_my_bucket_KEY_SECRET", "miniominio"),
	("SS3_BUCKET_my_bucket_ENDPOINT", "http://127.0.0.1:9000"),
	// as fallback
	("AWS_ACCESS_KEY_ID", "minio"),
	("AWS_SECRET_ACCESS_KEY", "miniominio"),
	("AWS_ENDPOINT", "http://127.0.0.1:9000"),
];

pub fn exec_ss3(ss3_sub_cmd: &str, args: &[&str], print_exec: bool) -> Result<(bool, String)> {
	let cmd_args = [&["run"], &[ss3_sub_cmd], args].concat();

	let output = exec_output(
		"cargo",
		&cmd_args,
		&ExecConfig {
			print_exec,
			..ExecConfig::default()
		},
	)?;

	Ok(output)
}

struct ExecConfig {
	print_exec: bool,
	cwd: Option<PathBuf>,
	envs: Option<HashMap<&'static str, &'static str>>,
}

impl Default for ExecConfig {
	fn default() -> Self {
		Self {
			print_exec: true,
			cwd: None,
			envs: Some(ENV_CREDS.into()),
		}
	}
}

fn exec_output(cmd: &str, args: &[&str], config: &ExecConfig) -> Result<(bool, String)> {
	let ExecConfig { print_exec, cwd, envs } = config;

	if *print_exec {
		println!("> executing: {} {}", cmd, args.join(" "));
	}

	let mut proc = Command::new(cmd);

	if let Some(cwd) = cwd {
		proc.current_dir(cwd);
	}
	proc.args(args);

	if let Some(envs) = envs {
		for (name, val) in envs.iter() {
			proc.env(name, val);
		}
	}

	match proc.stdout(Stdio::piped()).output() {
		Err(ex) => Err(ex)?,
		Ok(output) => {
			let success: bool;
			let txt = if output.status.success() {
				success = true;
				String::from_utf8(output.stdout)
			} else {
				success = false;
				String::from_utf8(output.stderr)
			};

			match txt {
				Err(ex) => Err(ex)?,
				Ok(txt) => Ok((success, txt)),
			}
		}
	}
}
