//! AWS API Wrapper

use crate::Error;
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};

mod cp;
mod cred;
mod list;
mod list_buckets;
mod sbucket;
mod sitem;

// re-export
pub use self::cp::CpOptions;
pub use self::cp::OverMode;
pub use self::cred::get_sbucket;
pub use self::cred::new_s3_client;
pub use self::list::{ListInfo, ListOptions, ListResult};
pub use self::list_buckets::list_buckets;
pub use self::sbucket::{SBucket, SBucketConfig};
pub use self::sitem::{SItem, SItemType};

// region:    --- Mod Utils

/// Compute the destination key given the eventual base_dir and src_file
/// * `dst_prefix` - the base prefix (directory like) or potentially the target key if renamable true
/// * `renamable` - when this flag, if the dst_prefix has a extension same as src_file (case insensitive)
fn compute_dst_key(base_dir: Option<&Path>, src_file: &Path, dst_prefix: &str, renamable: bool) -> Result<String, Error> {
	let file_name = src_file
		.file_name()
		.and_then(|s| s.to_str())
		.ok_or_else(|| Error::FilePathNotFound(src_file.display().to_string()))?;

	// Determine if it is an rename operation (if )
	let rename_only = if renamable {
		let dst_path = Path::new(dst_prefix);
		match (
			src_file.extension().and_then(|ext| ext.to_str().map(|v| v.to_lowercase())),
			dst_path.extension().and_then(|ext| ext.to_str().map(|v| v.to_lowercase())),
		) {
			(Some(src_ext), Some(dst_ext)) => src_ext == dst_ext,
			(_, _) => false,
		}
	} else {
		false
	};

	if rename_only {
		Ok(dst_prefix.to_string())
	} else {
		let diff_path = base_dir.and_then(|base_dir| diff_paths(src_file, base_dir));

		let key = match diff_path {
			None => Path::new(dst_prefix).join(file_name),
			Some(diff_path) => Path::new(dst_prefix).join(diff_path),
		};

		// TODO - Should throw an error if not a unicode string
		let key = key.display().to_string();

		Ok(key)
	}
}

/// Compute the destination file path given a base key and object key
/// Note: For now simple substring
fn compute_dst_path(base_key: &str, object_key: &str, base_dir: &Path) -> Result<PathBuf, Error> {
	// validate params
	if !object_key.starts_with(base_key) {
		panic!(
			"CODE ERROR - compute_dst_path - Base key '{}' is not the base for object_key '{}'",
			base_key, object_key
		);
	}

	// key diff
	let rel_key = object_key[base_key.len()..].to_string();

	Ok(base_dir.join(rel_key))
}

/// Determine if a key a directory (end with '/')
fn get_file_name(path: &Path) -> Result<String, Error> {
	path
		.file_name()
		.and_then(|s| s.to_str().map(|v| v.to_string()))
		.ok_or_else(|| Error::InvalidPath(path.to_string_lossy().to_string()))
}

enum PathType {
	File,
	Dir,
}

fn path_type(path: &Path) -> PathType {
	match path.extension().is_some() {
		true => PathType::File,
		false => PathType::Dir,
	}
}

// endregion: --- Mod Utils
