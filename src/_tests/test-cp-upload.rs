//! Unit test are not fully implemented yet

use crate::_test_support::{delete_s3_folder, new_test_ss3_bucket};
use crate::s3w::{CpOptions, ListOptions, OverMode};
use crate::utils::md5::compute_md5;
use std::path::Path;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

pub const FILE_FIXTURE_FILE_01: &str = "./tests-data/fixtures/fixture-01/image-01.jpg";
pub const FILE_FIXTURE_FILE_02: &str = "./tests-data/fixtures/fixture-01/sub-dir/sub-file-01.txt";

#[tokio::test]
async fn test_cp_write_etag_single_file() -> Result<()> {
	// -- Setup & Fixtures
	let fx_s3_folder = "test_cp_write_etag_single_file";
	let fx_files = &[FILE_FIXTURE_FILE_01, FILE_FIXTURE_FILE_02];
	let sbucket = new_test_ss3_bucket().await?;
	let opts = CpOptions {
		over: OverMode::Etag,
		recursive: true,
		..Default::default()
	};

	delete_s3_folder(&sbucket, fx_s3_folder).await?;

	// first upload to make sure it is there
	for file in fx_files {
		sbucket.upload_path(file, fx_s3_folder, opts.clone()).await?;
	}

	// -- Exec
	// TODO: needs to do the upload again to check that etag was there
	let res = sbucket.list("", &ListOptions::new(true)).await?;

	// -- Check
	// TODO: need to echeck

	Ok(())
}

#[tokio::test]
async fn test_cp_write_etag_dir() -> Result<()> {
	// -- Setup & Fixtures
	let fx_s3_folder = "test_cp_write_etag_dir";
	let fx_dir = "./tests-data/fixtures";
	let sbucket = new_test_ss3_bucket().await?;
	let opts = CpOptions {
		over: OverMode::Etag,
		recursive: true,
		..Default::default()
	};

	delete_s3_folder(&sbucket, fx_s3_folder).await?;

	// first upload to make sure it is there
	sbucket.upload_path(fx_dir, fx_s3_folder, opts.clone()).await?;

	println!("\nSECOND GET\n");

	sbucket.upload_path(fx_dir, fx_s3_folder, opts.clone()).await?;

	// -- Exec
	let res = sbucket.list("", &ListOptions::new(true)).await?;

	// -- Check
	// TODO: need to echeck

	Ok(())
}
