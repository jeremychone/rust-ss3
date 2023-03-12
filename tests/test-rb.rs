use crate::utils::{exec_ss3, XString, FILE_FIXTURE_IMAGE_01};
use anyhow::Result;
use serial_test::serial;

mod utils;

const RB_BUCKET: &str = "s3://test-rb-bucket";

#[test]
#[serial(test_rb)]
fn test_rb_success_empty() -> Result<()> {
	// FIXTURE
	exec_ss3("mb", &[RB_BUCKET], false)?;

	// CHECK - (and clean)
	let (success, out) = exec_ss3("rb", &[RB_BUCKET], false)?;
	assert!(success, "rb should be success=true. Cause: {out}");
	assert!(out.contains("test-rb-bucket"));

	Ok(())
}

#[test]
#[serial(test_rb)]
fn test_rb_fail_not_empty() -> Result<()> {
	// FIXTURE - rb bucket with some content.
	exec_ss3("mb", &[RB_BUCKET], false)?;
	let s3_object_url = format!("{RB_BUCKET}/{}", FILE_FIXTURE_IMAGE_01.x_file_name());
	exec_ss3("cp", &[FILE_FIXTURE_IMAGE_01, &s3_object_url], false)?;

	// EXEC
	let (success, out) = exec_ss3("rb", &[RB_BUCKET], false)?;

	// CHECK - Should fail to delete.
	assert!(!success, "rb success should be false");
	assert!(
		out.contains("BucketNotEmpty"),
		"rb result does not contain BucketNotEmpty. Out:\n{out}"
	);

	// CLEAN - Clean and delete the object and bucket.
	let (success, _) = exec_ss3("rm", &[&s3_object_url], false)?;
	assert!(success, "rm test bucket content should be success.");
	let (success, _) = exec_ss3("rb", &[RB_BUCKET], false)?;
	assert!(success, "rb test bucket should be success.");

	Ok(())
}
