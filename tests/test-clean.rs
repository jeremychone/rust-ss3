pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

mod utils;

use crate::utils::{copy_dir_all, delete_s3_folder};
use simple_fs::list_files;
use std::fs;
use utils::{exec_ss3, list_s3_folder};

const TEST_CLEAN_BUCKET: &str = "s3://test-clean-bucket";
const LOCAL_TEST_FOLDER: &str = "./tests-data/fixtures";

#[test]
fn test_clean_simple() -> Result<()> {
	let fx_s3_folder = "test_clean_simple_folder";
	let fx_partial_dir = "./.test-data/test_clean_simple_partial";
	let s3_folder = init_s3_folder(fx_s3_folder)?;

	// prep the partial dir to show to lean
	copy_dir_all(LOCAL_TEST_FOLDER, fx_partial_dir)?;
	let files_to_delete = list_files(fx_partial_dir, Some(&["**/*.jpg"]), None)?;
	let first_file = files_to_delete.first().ok_or("Should have a file to delete")?;
	fs::remove_file(first_file.path())?;

	// EXEC - delete images

	let (_count, _out) = exec_ss3("clean", &[fx_partial_dir, &s3_folder, "--force"], false)?;
	// CHECK
	let (count, _out) = list_s3_folder(&s3_folder)?;
	assert_eq!(count, 3, "Should be list of files minus one");

	Ok(())
}

// region:    --- Utils

/// Initialize a S3 folder with the fixture-01 content
fn init_s3_folder(test_name: &str) -> Result<String> {
	exec_ss3("mb", &[TEST_CLEAN_BUCKET], false)?;
	let s3_folder = format!("{TEST_CLEAN_BUCKET}/{test_name}/");
	delete_s3_folder(&s3_folder)?;
	exec_ss3("cp", &[LOCAL_TEST_FOLDER, &s3_folder, "-r"], false)?;

	Ok(s3_folder)
}

// endregion: --- Utils
