use crate::utils::{exec_ss3, get_test_bucket, XString, FILE_FIXTURE_IMAGE_01};
use anyhow::Result;

mod utils;

#[test]
fn test_rb_success_empty() -> Result<()> {
	// FIXTURE
	let (bucket_url, bucket_name) = get_test_bucket("test_rb_success_empty");
	exec_ss3("mb", &[&bucket_url], false)?;

	// EXEC
	let (success, out) = exec_ss3("rb", &[&bucket_url], false)?;

	// CHECK
	assert!(success, "rb should be success=true. Cause: {out}");
	assert!(out.contains(&bucket_name));

	Ok(())
}

#[test]
fn test_rb_fail_not_empty() -> Result<()> {
	// FIXTURE - rb bucket with some content.
	let (bucket_url, _bucket_name) = get_test_bucket("test_rb_fail_not_empty");
	exec_ss3("mb", &[&bucket_url], false)?;
	let s3_object_url = format!("{bucket_url}/{}", FILE_FIXTURE_IMAGE_01.x_file_name());
	exec_ss3("cp", &[FILE_FIXTURE_IMAGE_01, &s3_object_url], false)?;

	// EXEC
	let (success, out) = exec_ss3("rb", &[&bucket_url], false)?;

	// CHECK - Should fail to delete.
	assert!(!success, "rb success should be false");
	assert!(
		out.contains("BucketNotEmpty"),
		"rb result does not contain BucketNotEmpty. Out:\n{out}"
	);

	// CLEAN - Clean and delete the object and bucket.
	let (success, _) = exec_ss3("rm", &[&s3_object_url], false)?;
	assert!(success, "rm test bucket content should be success.");
	let (success, _) = exec_ss3("rb", &[&bucket_url], false)?;
	assert!(success, "rb test bucket should be success.");

	Ok(())
}
