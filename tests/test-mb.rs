use crate::utils::exec_ss3;
use anyhow::Result;
use serial_test::serial;

mod utils;

const TEST_MB_BUCKET: &str = "s3://test-mb-bucket";

#[test]
#[serial]
fn test_mb_base() -> Result<()> {
	// FIXTURE --- Make sure it is deleted
	exec_ss3("rb", &[TEST_MB_BUCKET], true)?;

	// CHECK
	let (success, out) = exec_ss3("mb", &[TEST_MB_BUCKET], true)?;
	// println!("->> test_mb_base mb: {success} \n{out}");
	assert!(success, "mb success");
	assert!(out.contains("test-mb-bucket"));

	// CLEAN
	exec_ss3("rb", &[TEST_MB_BUCKET], true)?;

	Ok(())
}
