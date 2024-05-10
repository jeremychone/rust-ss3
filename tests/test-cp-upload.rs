pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

mod utils;

use utils::{create_bucket, delete_s3_folder, exec_ss3, XString, FILE_FIXTURE_01_DIR, FILE_FIXTURE_IMAGE_01};

const TEST_CP_UPLOAD_BUCKET: &str = "s3://test-cp-upload-bucket";

#[test]
fn test_cp_upload_file_to_key() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_file_to_key/");
	let s3_object_url = format!("{s3_base_dir}test-image.jpg");
	let args = &[FILE_FIXTURE_IMAGE_01, &s3_object_url];

	// EXEC-CHECK-CLEAN
	let (cp_out, _ls_out) = base_tcc_cp_upload(&s3_base_dir, args, 1)?;

	// CHECK -- Additional check
	assert!(
		cp_out.contains(&s3_object_url),
		"Should contain: {s3_object_url}\nbut contained: {cp_out}"
	);

	Ok(())
}

#[test]
fn test_cp_upload_file_to_s3dir() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_file_to_s3dir/");
	let args = &[FILE_FIXTURE_IMAGE_01, &s3_base_dir];

	// EXEC-CHECK-CLEAN
	let (cp_out, _ls_out) = base_tcc_cp_upload(&s3_base_dir, args, 1)?;

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
	base_tcc_cp_upload(&s3_base_dir, args, 2)?;

	Ok(())
}

#[test]
fn test_cp_upload_dir_recursive_all() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_dir_recursive/");
	let args = &[FILE_FIXTURE_01_DIR, &s3_base_dir, "-r"];

	// EXEC-CHECK-CLEAN
	base_tcc_cp_upload(&s3_base_dir, args, 4)?;

	Ok(())
}

#[test]
fn test_cp_upload_dir_recursive_includes_txt() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_dir_recursive_includes_txt/");
	let args = &[FILE_FIXTURE_01_DIR, &s3_base_dir, "-r", "-i", "*.txt"];

	// EXEC-CHECK-CLEAN
	base_tcc_cp_upload(&s3_base_dir, args, 3)?;

	Ok(())
}

#[test]
fn test_cp_upload_dir_recursive_excludes_txt() -> Result<()> {
	// FIXTURE
	let s3_base_dir = format!("{TEST_CP_UPLOAD_BUCKET}/test_cp_upload_dir_recursive_excludes_txt/");
	let args = &[FILE_FIXTURE_01_DIR, &s3_base_dir, "-r", "-e", "*.txt"];

	// EXEC-CHECK-CLEAN - Should have only 1 file
	base_tcc_cp_upload(&s3_base_dir, args, 1)?;

	Ok(())
}

// region:    --- Utils

/// Base test-check-clean for the cp upload test.
/// - Exec the ss3 cp with the args,
/// - Do the expected_count check with as ss3 ls -r from the s3_base_dir
/// - Clean the s3_base_dir
/// - Return the (cp_output, ls_ouput) tuple for dditional check
fn base_tcc_cp_upload(s3_base_dir: &str, args: &[&str], expected_count: usize) -> Result<(String, String)> {
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
	delete_s3_folder(s3_base_dir)?;

	Ok((cp_out, ls_out))
}

// endregion: --- Utils
