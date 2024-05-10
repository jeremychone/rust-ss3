#![allow(unused)]

// region:    --- Modules

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

// -- Sub-modules
mod exec;

// --- Re-exports
pub use exec::exec_ss3;

// --- Imports
use std::path::Path;
use std::str::Lines;
use std::sync::Mutex;

// endregion: --- Modules

// region:    --- Consts
pub const FILE_FIXTURE_IMAGE_01: &str = "./tests-data/fixtures/fixture-01/image-01.jpg";
pub const FILE_FIXTURE_01_DIR: &str = "./tests-data/fixtures/fixture-01/";

pub const S3_FIXTURES_BUCKET: &str = "s3://test-fixtures";
pub const S3_FIXTURE_01_DIR: &str = "s3://test-fixtures/fixture-01/";
// endregion: --- Consts

// region:    --- Fixture
static INIT_DONE: Mutex<bool> = Mutex::new(false);

/// Returns the s3 url of the fixtures bucket
pub fn lazy_init_fixtures() -> Result<&'static str> {
	let mut init_done = INIT_DONE.lock().unwrap();
	if !*init_done {
		exec_ss3("mb", &[S3_FIXTURES_BUCKET], false)?;

		// CREATE - Create the fixtures/ folder if does not exist.
		let (_, out) = exec_ss3("ls", &[S3_FIXTURES_BUCKET], false)?;
		if !out.x_has_line("fixtures/") {
			println!("TEST INFO - Create fixtures.");
			exec_ss3("cp", &["./tests-data/fixtures", S3_FIXTURES_BUCKET, "-r"], false)?;
		}

		*init_done = true;
	}

	Ok(S3_FIXTURES_BUCKET)
}

pub fn upload_fixture_01(s3_folder: &str) -> Result<()> {
	exec_ss3("cp", &[FILE_FIXTURE_01_DIR, s3_folder, "-r"], false)?;

	Ok(())
}

// endregion: --- Fixture

// region:    --- S3 Utils

pub fn create_bucket(bucket_url: &str) -> Result<()> {
	exec_ss3("mb", &[bucket_url], false)?;
	Ok(())
}

pub fn delete_bucket(bucket_url: &str) -> Result<()> {
	delete_s3_folder(bucket_url)?;
	let (ok, out) = exec_ss3("rb", &[bucket_url], false)?;

	Ok(())
}

pub fn delete_s3_folder(s3_url: &str) -> Result<()> {
	let (_, out) = exec_ss3("ls", &[s3_url, "-r"], false)?;
	let bucket_url = get_bucket_from_s3_url(s3_url)?;
	for item in out.x_lines() {
		let obj_url = format!("{bucket_url}/{item}");
		let (ok, out) = exec_ss3("rm", &[&obj_url], false)?;
	}

	Ok(())
}

/// Returns (bucket_url, bucket_name)
pub fn get_test_bucket(test_name: &str) -> (String, String) {
	// Note: '_' are not valid in bucket names
	let bucket_name = format!("test-bucket-{}", test_name.replace('_', "-"));
	let bucket_url = format!("s3://{bucket_name}");
	(bucket_url, bucket_name)
}

pub fn get_bucket_from_s3_url(s3_url: &str) -> Result<String> {
	let bucket_name = s3_url
		.strip_prefix("s3://")
		.and_then(|s| s.split('/').next())
		.filter(|s| !s.is_empty())
		.ok_or_else(|| format!("Wrong S3 URL format {}", s3_url))?;

	Ok(format!("s3://{bucket_name}"))
}

/// Execute a ss3 ls recursive and return count and content (new line per keys)
pub fn list_s3_folder(s3_url: &str) -> Result<(usize, String)> {
	let (success, out) = exec_ss3("ls", &[s3_url, "-r"], false)?;
	if !success {
		return Err(format!("Fail to do ss3 ls {s3_url}. Cause:\n {out}").into());
	}
	let count = out.trim().split('\n').count();
	Ok((count, out))
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
