pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

mod utils;

use crate::utils::{exec_ss3, get_test_bucket};

#[test]
fn test_mb_success_simple_create() -> Result<()> {
	// FIXTURE -- Make sure it is deleted
	let (bucket_url, bucket_name) = get_test_bucket("test_mb_success_simple_create");

	// EXEC
	let (success, out) = exec_ss3("mb", &[&bucket_url], false)?;

	// CHECK
	assert!(success, "mb success");
	assert!(
		out.contains(&bucket_name),
		"ss3 output should have contained '{bucket_name}' but:\n{out}"
	);

	// CLEAN
	exec_ss3("rb", &[&bucket_url], false)?;

	Ok(())
}

#[test]
fn test_mb_fail_already_exist() -> Result<()> {
	// FIXTURE -- Make sure already exist
	let (bucket_url, _bucket_name) = get_test_bucket("test_mb_fail_already_exist");
	exec_ss3("mb", &[&bucket_url], false)?;

	// EXEC
	let (success, out) = exec_ss3("mb", &[&bucket_url], false)?;

	// CHECK
	assert!(!success, "ss3 mb should have failed (but success).");
	assert!(
		out.contains("BucketAlreadyOwnedByYou"),
		"ss3 mb output should have 'BucketAlreadyOwnedByYou'"
	);

	// CLEAN
	exec_ss3("rb", &[&bucket_url], false)?;

	Ok(())
}
