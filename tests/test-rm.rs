pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

mod utils;

use utils::{exec_ss3, list_s3_folder, upload_fixture_01};

const TEST_RM_BUCKET: &str = "s3://test-rm-bucket";

#[test]
fn test_rm_single_key() -> Result<()> {
	let s3_folder = init_s3_folder("test_rm_single_key")?;

	// EXEC - delete images
	let key_to_delete = format!("{s3_folder}image-01.jpg");
	exec_ss3("rm", &[&key_to_delete], false)?;

	// CHECK
	let (count, out) = list_s3_folder(&s3_folder)?;
	assert_eq!(count, 3, "Number of files in the {s3_folder}");
	assert!(!out.contains("image-01.jpg"), "Should not contain 'image-01.jpg'");

	Ok(())
}

// region:    --- Utils

/// Initialize a S3 folder with the fixture-01 content
fn init_s3_folder(test_name: &str) -> Result<String> {
	exec_ss3("mb", &[TEST_RM_BUCKET], false)?;
	let s3_folder = get_test_s3_folder(test_name);
	upload_fixture_01(&s3_folder)?;
	Ok(s3_folder)
}

fn get_test_s3_folder(test_name: &str) -> String {
	format!("{TEST_RM_BUCKET}/{test_name}/")
}

// endregion: --- Utils
