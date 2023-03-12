//!
//! Note: Those tests not need to have #[serial] as it is read global fixtures only.

use anyhow::Result;
use utils::{exec_ss3, lazy_init_fixture, XString};

mod utils;

#[test]
fn test_ls_base() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;

	// EXEC
	let (success, out) = exec_ss3("ls", &["s3://my-bucket/"], true)?;

	// CHECK
	println!("->> out: {success} \n{out}");
	assert!(out.x_has_line("fixtures/"), "'fixtures/' was not found");

	Ok(())
}

#[test]
fn test_ls_fixture_01_count_base() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;

	// EXEC
	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/"], true)?;

	// CHECK
	// NOTE: With non recursive, the "folder" in the base path will be returned.
	assert_eq!(out.x_lines().count(), 3);

	Ok(())
}

#[test]
fn test_ls_fixture_01_count_recursive() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;

	// EXEC
	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/", "-r"], true)?;

	// CHECK
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 4);

	Ok(())
}

#[test]
fn test_ls_fixture_01_count_includes_txt() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;

	// EXEC
	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/", "-r", "-i", "*.txt"], true)?;

	// CHECK
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 3);

	Ok(())
}

#[test]
fn test_ls_fixture_01_count_excludes_txt() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;

	// EXEC
	let (_, out) = exec_ss3("ls", &["s3://my-bucket/fixtures/fixture-01/", "-r", "-e", "*.txt"], true)?;

	// CHECK
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 1);

	Ok(())
}

#[test]
fn test_ls_fixture_01_count_includes_multiple() -> Result<()> {
	// FIXTURE
	lazy_init_fixture()?;

	// EXEC
	let (_, out) = exec_ss3(
		// CHECK
		"ls",
		&["s3://my-bucket/fixtures/fixture-01/", "-r", "-i", "**/sub*.*", "-i", "*.jpg"],
		true,
	)?;
	// NOTE: With recursive, the 'folder' are not returned.
	assert_eq!(out.x_lines().count(), 3);

	Ok(())
}
