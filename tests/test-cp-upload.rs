use anyhow::Result;
use utils::{create_bucket, delete_folder, exec_ss3, XString, FILE_FIXTURE_01_DIR, FILE_FIXTURE_IMAGE_01};

mod utils;

const TEST_CP_UPLOAD_BUCKET: &str = "s3://test-cp-upload-bucket";

#[test]
fn test_cp_upload_file_to_key() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_file_to_key/");
	let args = &[FILE_FIXTURE_IMAGE_01, &s3_base_dir];
	let s3_object_url = format!("{s3_base_dir}{}", FILE_FIXTURE_IMAGE_01.x_file_name());

	// EXEC
	let (cp_out, _ls_out) = base_test_and_clean_cp(&s3_base_dir, args, 1)?;

	// CHECK -- Addtiional check
	assert!(cp_out.contains(&s3_object_url));

	// CLEAN
	delete_folder(&s3_base_dir)?;

	Ok(())
}

#[test]
fn test_cp_upload_file_to_s3dir() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_file_to_s3dir/");
	let args = &[FILE_FIXTURE_IMAGE_01, &s3_base_dir];

	// EXEC-CHECK-CLEAN
	let (cp_out, _ls_out) = base_test_and_clean_cp(&s3_base_dir, args, 1)?;

	// CHECK - Addional check on cp_out
	assert!(cp_out.contains(&s3_base_dir), "ss3 cp output should contain {s3_base_dir}");
	let file_name = FILE_FIXTURE_IMAGE_01.x_file_name();
	assert!(cp_out.contains(&file_name), "ss3 cp output should contain {file_name}");

	Ok(())
}

#[test]
fn test_cp_upload_dir_non_recursive() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_dir_non_recursive/");
	let args = &[FILE_FIXTURE_01_DIR, &s3_base_dir];

	// EXEC-CHECK-CLEAN
	base_test_and_clean_cp(&s3_base_dir, args, 2)?;

	Ok(())
}

#[test]
fn test_cp_upload_dir_recursive() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_dir_recursive/");
	let args = &[FILE_FIXTURE_01_DIR, &s3_base_dir, "-r"];

	// EXEC-CHECK-CLEAN
	base_test_and_clean_cp(&s3_base_dir, args, 4)?;

	Ok(())
}

// region:    --- Base Test Functions

/// Base test for the cp tests.
/// - Exec the ss3 cp with the args,
/// - Do the expected_count check with as ss3 ls -r from the s3_base_dir
/// - Clean the s3_base_dir
/// - Return the (cp_output, ls_ouput) tuple for dditional check
fn base_test_and_clean_cp(s3_base_dir: &str, args: &[&str], expected_count: usize) -> Result<(String, String)> {
	create_bucket(TEST_CP_UPLOAD_BUCKET)?;

	// EXEC
	let (success, cp_out) = exec_ss3("cp", args, false)?;

	// CHECK
	assert!(success, "success");
	// check expected_count
	let (success, ls_out) = exec_ss3("ls", &[s3_base_dir, "-r"], false)?;
	assert!(success, "ls should be success");
	assert_eq!(
		ls_out.x_lines().count(),
		expected_count,
		"Should have {expected_count} items in folder and sub folders of '{s3_base_dir}'"
	);

	// CLEAN
	delete_folder(s3_base_dir)?;

	Ok((cp_out, ls_out))
}
// endregion: --- Base Test Functions
