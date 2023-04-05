//! AWS API Wrapper

// --- Re-exports
pub use self::bucket_ops::{create_bucket, delete_bucket, list_buckets};
pub use self::cp::CpOptions;
pub use self::cp::OverMode;
pub use self::cred::{get_sbucket, new_s3_client, RegionProfile};
pub use self::list::{ListInfo, ListOptions, ListResult};
pub use self::sbucket::{SBucket, SBucketConfig};
pub use self::sitem::{SItem, SItemType};

// --- Imports
use crate::{Error, Result};
use globset::GlobSet;
use pathdiff::diff_paths;
use std::path::{Path, PathBuf};

// --- Sub Modules
mod bucket_ops;
mod cp;
mod cred;
mod list;
mod rm;
mod sbucket;
mod sitem;

// region:    --- Includes/Excludes Utils
/// Inclusion/Exclusion result
enum Inex {
	Include,
	ExcludeInExclude,
	ExcludeNotInInclude,
}

/// validate the Include / Exclusion rules
fn compute_inex(key: &str, includes: &Option<GlobSet>, excludes: &Option<GlobSet>) -> Inex {
	// Note: Those match_... will have 3 states, None (if no rule), Some(true), Some(false)
	let match_include = includes.as_ref().map(|gs| !gs.matches(key).is_empty());
	let match_exclude = excludes.as_ref().map(|gs| !gs.matches(key).is_empty());

	match (match_include, match_exclude) {
		// if pass the include gate (no include rule or matched it) and not in eventual exclude
		(None | Some(true), None | Some(false)) => Inex::Include,
		// passed the include gate, but is explicity excluded
		(None | Some(true), Some(true)) => Inex::ExcludeInExclude,
		// Did not pass the include gate
		(Some(false), _) => Inex::ExcludeNotInInclude,
	}
}

fn validate_key(key: &str, includes: &Option<GlobSet>, excludes: &Option<GlobSet>) -> bool {
	matches!(compute_inex(key, includes, excludes), Inex::Include)
}

// endregion: --- Includes/Excludes Utils

// region:    --- Mod Utils

/// Compute the destination key given the eventual base_dir and src_file
/// * `dst_prefix` - the base prefix (directory like) or potentially the target key if renamable true
/// * `renamable` - when this flag, if the dst_prefix has a extension same as src_file (case insensitive)
fn compute_dst_key(base_dir: Option<&Path>, src_file: &Path, dst_prefix: &str, renamable: bool) -> Result<String> {
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
fn compute_dst_path(base_key: &str, object_key: &str, base_dir: &Path) -> Result<PathBuf> {
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
fn get_file_name(path: &Path) -> Result<String> {
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
