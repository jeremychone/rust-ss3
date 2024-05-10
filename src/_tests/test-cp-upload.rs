use crate::_test_support::new_test_ss3_bucket;
use crate::s3w::{CpOptions, ListOptions, OverMode};
use crate::utils::md5::compute_md5;
use std::path::Path;

pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

pub const FILE_FIXTURE_IMAGE_01: &str = "./tests-data/fixtures/fixture-01/image-01.jpg";

#[tokio::test]
async fn test_cp_write_etag() -> Result<()> {
	// -- Setup & Fixtures
	let sbucket = new_test_ss3_bucket().await?;
	let opts = CpOptions {
		over: OverMode::Etag,
		..Default::default()
	};

	// first upload to make sure it is there
	sbucket
		.upload_path(Path::new(FILE_FIXTURE_IMAGE_01), "/test_cp_write_etag/", opts.clone())
		.await?;

	// -- Exec
	sbucket
		.upload_path(Path::new(FILE_FIXTURE_IMAGE_01), "/test_cp_write_etag/", opts)
		.await?;

	// -- Check
	// TODO: need to echeck

	Ok(())
}
