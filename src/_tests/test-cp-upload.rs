//! Unit test are not fully implemented yet

use crate::_test_support::{delete_s3_folder, new_test_ss3_bucket};
use crate::s3w::{CpOptions, ListOptions, OverMode};

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
		show_skip: true,
		..Default::default()
	};

	delete_s3_folder(&sbucket, fx_s3_folder).await?;

	// First upload
	for file in fx_files {
		sbucket.upload_path(file, fx_s3_folder, opts.clone()).await?;
	}

	// -- Exec
	// Second upload
	for file in fx_files {
		sbucket.upload_path(file, fx_s3_folder, opts.clone()).await?;
	}

	// -- Check
	// TODO: need to the check the upload etags (upload_path should return a response like
	//       `UploadResponse { uploaded_count: u64, skip_total: u64, skip_etag: u64, ...}`)
	let res = sbucket.list(fx_s3_folder, &ListOptions::new(true)).await?;
	assert_eq!(res.objects.len(), 2, "Should have 2 s3 objects");

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

	// -- Check
	let _res = sbucket.list("", &ListOptions::new(true)).await?;
	// TODO: need to echeck

	Ok(())
}
