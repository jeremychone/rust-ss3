use anyhow::Result;
use utils::{exec_ss3, lazy_init_fixture, XString};

mod utils;

#[test]
fn test_ls_base() -> Result<()> {
	lazy_init_fixture()?;

	let (_, out) = exec_ss3("ls", &["s3://my-bucket/"], true)?;
	assert!(out.x_has_line("fixtures/"), "'fixtures/' was not found");
	Ok(())
}

#[test]
fn test_ls_fixture_01_count_base() -> Result<()> {
	lazy_init_fixture()?;

	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/"], true)?;
	// NOTE: With non recursive, the "folder" in the base path will be returned.
	assert_eq!(out.x_lines().count(), 3);
	Ok(())
}

#[test]
fn test_ls_fixture_01_count_recursive() -> Result<()> {
	lazy_init_fixture()?;

	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/", "-r"], true)?;
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 4);
	Ok(())
}

#[test]
fn test_ls_fixture_01_count_includes_txt() -> Result<()> {
	lazy_init_fixture()?;

	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/", "-r", "-i", "*.txt"], true)?;
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 3);
	Ok(())
}

#[test]
fn test_ls_fixture_01_count_excludes_txt() -> Result<()> {
	lazy_init_fixture()?;

	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/", "-r", "-e", "*.txt"], true)?;
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 1);
	Ok(())
}

#[test]
fn test_ls_fixture_01_count_includes_multiple() -> Result<()> {
	lazy_init_fixture()?;

	let (_, out) = exec_ss3(
		"ls",
		&["s3://my-bucket/fixtures/fixture-01/", "-r", "-i", "**/sub*.*", "-i", "*.jpg"],
		true,
	)?;
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 3);
	Ok(())
}
