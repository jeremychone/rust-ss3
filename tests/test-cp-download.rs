use anyhow::{bail, Result};
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use utils::{exec_ss3, lazy_init_fixtures, S3_FIXTURE_01_DIR};
use walkdir::WalkDir;

mod utils;

const TEST_CP_DOWNLOAD_BASE_DIR: &str = "./tests-data/.tmp/test-cp-download-base-dir/";

#[test]
fn test_cp_download_s3_dir_flat() -> anyhow::Result<()> {
	let (dir_path, dir_str) = get_test_dir("test_cp_download_s3_dir_flat");

	// EXEC-CHECK-CLEAN
	let files = base_tcc_cp_download(&dir_path, &[S3_FIXTURE_01_DIR, &dir_str], 2)?;

	// CHECK - additional check
	assert!(
		files.contains("test_cp_download_s3_dir_flat/image-01.jpg"),
		"Does not contain 'image-01.jpg'. Content:\n{}",
		files
	);
	assert!(
		files.contains("/test_cp_download_s3_dir_flat/some-text.txt"),
		"Does not contain 'some-text.txt'. Content:\n{}",
		files
	);

	Ok(())
}

#[test]
fn test_cp_download_recursive_all() -> anyhow::Result<()> {
	let (dir_path, dir_str) = get_test_dir("test_cp_download_recursive_all");

	// EXEC-CHECK-CLEAN
	let files = base_tcc_cp_download(&dir_path, &[S3_FIXTURE_01_DIR, &dir_str, "-r"], 4)?;

	// CHECK - additional check
	assert!(
		files.contains("test_cp_download_recursive_all/image-01.jpg"),
		"Does not contain 'image-01.jpg'. Content:\n{}",
		files
	);
	assert!(
		files.contains("test_cp_download_recursive_all/sub-dir/sub-file-01.txt"),
		"Does not contain 'some-text.txt'. Content:\n{}",
		files
	);

	Ok(())
}

#[test]
fn test_cp_download_recursive_exclude_txt() -> anyhow::Result<()> {
	let (dir_path, dir_str) = get_test_dir("test_cp_download_recursive_exclude_txt");

	// EXEC-CHECK-CLEAN
	let files = base_tcc_cp_download(&dir_path, &[S3_FIXTURE_01_DIR, &dir_str, "-r", "-e", "*.txt"], 1)?;

	// CHECK - additional check
	assert!(
		files.contains("test_cp_download_recursive_exclude_txt/image-01.jpg"),
		"Does not contain 'image-01.jpg'. Content:\n{}",
		files
	);

	Ok(())
}

// region:    --- utils

/// Base test-check-clean for the cp tests.
/// - Exec the ss3 cp with the args into a local folder
/// - Do the expected_count check with as ss3 ls -r from the s3_base_dir
/// - Clean the s3_base_dir
/// - Return the (cp_output, ls_ouput) tuple for dditional check
fn base_tcc_cp_download(local_dir: &Path, args: &[&str], expected_count: usize) -> Result<String> {
	lazy_init_fixtures()?;
	create_dir_all(local_dir)?;

	// EXEC
	let (success, _cp_out) = exec_ss3("cp", args, false)?;

	// CHECK
	assert!(success, "cp success was false!");
	// check expected_count
	let files = list_folder_files(local_dir)?;
	assert_eq!(
		files.len(),
		expected_count,
		"Should have {expected_count} file in folder '{}'",
		local_dir.to_string_lossy()
	);

	// CLEAN
	safer_remove_dir_all(local_dir)?;

	Ok(files.join("\n"))
}

fn get_test_dir(test_name: &str) -> (PathBuf, String) {
	let test_dir = PathBuf::from(TEST_CP_DOWNLOAD_BASE_DIR).join(format!("{test_name}/"));
	let test_dir_str = test_dir.to_str().unwrap().to_string();
	(test_dir, test_dir_str)
}

fn list_folder_files(local_dir: &Path) -> Result<Vec<String>> {
	let walker = WalkDir::new(local_dir).max_depth(usize::MAX).into_iter();
	Ok(
		walker
			.flatten()
			.filter(|e| e.path().is_file())
			.map(|e| e.path().to_string_lossy().to_string())
			.collect(),
	)
}

fn safer_remove_dir_all(path: &Path) -> Result<()> {
	let path = path.canonicalize()?;

	if !path.to_str().unwrap().contains("/tests-data/.tmp") {
		bail!("Unsafe to delete path: {:?}", path);
	} else {
		remove_dir_all(path)?;
	}

	Ok(())
}

//endregion: --- utils
