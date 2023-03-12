use crate::utils::exec_ss3;
use anyhow::Result;
use serial_test::serial;

mod utils;

const TEST_MB_BUCKET: &str = "s3://test-mb-bucket";

#[test]
#[serial(test_mb)]
fn test_mb_success_simple_create() -> Result<()> {
	// FIXTURE -- Make sure it is deleted
	exec_ss3("rb", &[TEST_MB_BUCKET], false)?;

	// EXEC
	let (success, out) = exec_ss3("mb", &[TEST_MB_BUCKET], false)?;

	// CHECK
	assert!(success, "mb success");
	assert!(out.contains("test-mb-bucket"), "created bucket name");

	// CLEAN
	exec_ss3("rb", &[TEST_MB_BUCKET], false)?;

	Ok(())
}

#[test]
#[serial(test_mb)]
fn test_mb_fail_already_exist() -> Result<()> {
	// FIXTURE -- Make sure already exist
	exec_ss3("mb", &[TEST_MB_BUCKET], false)?;

	// EXEC
	let (success, out) = exec_ss3("mb", &[TEST_MB_BUCKET], false)?;

	// CHECK
	assert!(!success, "ss3 mb should have failed (but success).");
	assert!(
		out.contains("BucketAlreadyOwnedByYou"),
		"ss3 mb output should have 'BucketAlreadyOwnedByYou'"
	);

	// CLEAN
	exec_ss3("rb", &[TEST_MB_BUCKET], false)?;

	Ok(())
}
