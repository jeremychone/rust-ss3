use anyhow::Result;
use serial_test::serial;
use utils::{exec_ss3, lazy_init_fixture, XString, FILE_FIXTURE_IMAGE_01};

mod utils;

const TEST_CP_BUCKET: &str = "s3://test-cp-upload-bucket";

#[test]
#[serial(test_cp_upload)]
fn test_cp_upload_simple() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;
	exec_ss3("mb", &[TEST_CP_BUCKET], true)?;
	let s3_object_url = format!("{TEST_CP_BUCKET}/{}", FILE_FIXTURE_IMAGE_01.x_file_name());

	// CHECK
	let (success, out) = exec_ss3("cp", &[FILE_FIXTURE_IMAGE_01, &s3_object_url], true)?;
	println!("->> test_cp_updload_simple: {success} \n{out}");
	assert!(success, "success");
	assert!(out.contains(&s3_object_url));

	// CLEAN
	let (ok, _) = exec_ss3("rm", &[&s3_object_url], true)?;
	assert!(ok, "rm s3 object");
	let (ok, _) = exec_ss3("rb", &[TEST_CP_BUCKET], true)?;
	assert!(ok, "rb bucket");

	Ok(())
}
