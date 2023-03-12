#![allow(unused)]

// --- Re-exports
pub use exec::exec_ss3;

// --- Imports
use anyhow::{anyhow, bail, Result};
use std::path::Path;
use std::str::Lines;
use std::sync::Mutex;

// --- Sub-Modules
mod exec;

// region:    --- Consts
pub const FILE_FIXTURE_IMAGE_01: &str = "./tests-data/fixtures/fixture-01/image-01.jpg";
pub const FILE_FIXTURE_01_DIR: &str = "./tests-data/fixtures/fixture-01/";
// endregion: --- Consts

// region:    --- Fixture
static INIT_DONE: Mutex<bool> = Mutex::new(false);

pub fn lazy_init_fixture() -> Result<()> {
	let mut init_done = INIT_DONE.lock().unwrap();
	if !*init_done {
		println!("->> INIT!!!");
		exec_ss3("mb", &["s3://my-bucket/"], false)?;

		// CREATE - Create the fixtures/ folder if does not exist.
		let (_, out) = exec_ss3("ls", &["s3://my-bucket/"], false)?;
		if !out.x_has_line("fixtures/") {
			println!("TEST INFO - Create fixtures.");
			exec_ss3("cp", &["./tests-data/fixtures", "s3://my-bucket/fixtures", "-r"], false)?;
		}

		*init_done = true;
	}

	Ok(())
}
// endregion: --- Fixture

// region:    --- S3 Utils

pub fn create_bucket(bucket_url: &str) -> Result<()> {
	exec_ss3("mb", &[bucket_url], false)?;
	Ok(())
}

pub fn delete_bucket(bucket_url: &str) -> Result<()> {
	delete_folder(bucket_url)?;
	let (ok, out) = exec_ss3("rb", &[bucket_url], false)?;

	Ok(())
}

pub fn delete_folder(s3_url: &str) -> Result<()> {
	let (_, out) = exec_ss3("ls", &[s3_url, "-r"], false)?;
	let bucket_url = get_bucket_url(s3_url)?;
	for item in out.x_lines() {
		let obj_url = format!("{bucket_url}/{item}");
		let (ok, out) = exec_ss3("rm", &[&obj_url], false)?;
	}

	Ok(())
}

pub fn get_bucket_url(s3_url: &str) -> Result<String> {
	let bucket_name = s3_url
		.strip_prefix("s3://")
		.and_then(|s| s.split('/').next())
		.filter(|s| !s.is_empty())
		.ok_or_else(|| anyhow!("Wrong S3 URL format {}", s3_url))?;

	Ok(format!("s3://{bucket_name}"))
}

// endregion: --- S3 Utils

// region:    --- String Utils
// Note: Personal best practice, "x" prefix to note that this is just private crate interface.

pub trait XString {
	fn x_lines(&self) -> Lines;
	fn x_has_line(&self, line: &str) -> bool;
	fn x_file_name(&self) -> String;
}

impl XString for str {
	/// Return the str::Lines but for the trimmed text (so no starting or ending empty lines)
	fn x_lines(&self) -> Lines {
		self.trim().lines()
	}
	fn x_has_line(&self, line: &str) -> bool {
		self.x_lines().any(|l| l == line)
	}

	fn x_file_name(&self) -> String {
		let path = Path::new(self);
		path.file_name().and_then(|s| s.to_str()).expect("fileName").to_string()
	}
}

impl XString for String {
	/// Return the str::Lines but for the trimmed text (so no starting or ending empty lines)
	fn x_lines(&self) -> Lines {
		str::x_lines(self)
	}

	fn x_has_line(&self, line: &str) -> bool {
		str::x_has_line(self, line)
	}

	fn x_file_name(&self) -> String {
		str::x_file_name(self)
	}
}
// endregion: --- String Utils
