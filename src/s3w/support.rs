use super::SBucket;
use crate::{Error, Result};
use globset::GlobSet;
use std::path::Path;
// use tokio_stream::StreamExt;
use crate::s3w::SItemsCache;
use crate::utils::md5::compute_md5;

// region:    --- Upload/Download Types

#[derive(Debug, Default, Clone, strum::AsRefStr, strum::EnumString)]
pub enum OverMode {
	/// Overwrite no matter what.
	Write,

	/// Skip if exists.
	#[default]
	Skip,

	/// Etag (only if different etag)
	/// NOTE: Does not support multi-part etag. Just assume simple md5 etag
	Etag,

	/// Fail if exists.
	Fail,
}

impl OverMode {
	pub fn label(&self) -> &'static str {
		match self {
			OverMode::Write => "Write",
			OverMode::Skip => "Exists",
			OverMode::Etag => "Etag",
			OverMode::Fail => "Fail",
		}
	}
}

#[derive(Default, Clone)]
pub struct CpOptions {
	pub recursive: bool,
	pub excludes: Option<GlobSet>,
	pub includes: Option<GlobSet>,
	pub over: OverMode,
	pub show_skip: bool,
	/// File with no extension content type
	pub noext_ct: Option<String>,
}

// endregion: --- Upload/Download Types

pub(super) async fn validate_over_for_s3_dest(
	sbucket: &SBucket,
	key: &str,
	src_file: &Path,
	opts: &CpOptions,
	sitems_cache: Option<&SItemsCache>,
) -> Result<bool> {
	match opts.over {
		// if over: Write, then always true, we overwrite
		OverMode::Write => Ok(true),

		// if skip, then the opposite of the exists state
		OverMode::Skip => Ok(!sbucket.exists(key).await),

		OverMode::Etag => Ok(!check_has_and_same_etags(sbucket, key, src_file, sitems_cache).await),

		// if fail mode, then if exists fail with error
		OverMode::Fail => {
			if sbucket.exists(key).await {
				Err(Error::ObjectExistsOverFailMode(format!("s3://{}/{key}", sbucket.name)))
			} else {
				Ok(true)
			}
		}
	}
}

pub(super) fn validate_over_for_file_dest(file: &Path, opts: &CpOptions) -> Result<bool> {
	match opts.over {
		// if over: Write, then always true, we overwrite
		OverMode::Write => Ok(true),

		// if skip, then the opposite of the exists state
		OverMode::Skip => Ok(!file.exists()),

		OverMode::Etag => panic!("->> FATAL ERROR - '--over etag' download not implemented yet"),

		// if fail mode, then if exists fail with error
		OverMode::Fail => {
			if file.exists() {
				Err(Error::FileExistsOverFailMode(file.display().to_string()))
			} else {
				Ok(true)
			}
		}
	}
}

/// returns true if both s3 object and files has successful etag, and the etcat match
async fn check_has_and_same_etags(sbucket: &SBucket, s3_key: &str, file: impl AsRef<Path>, sitems_cache: Option<&SItemsCache>) -> bool {
	// -- Get from cache or from s3 server if not found in cache
	// A little odd block, but necessary give ownership constraints.
	let sitem = sitems_cache.and_then(|c| c.get(s3_key));
	let sitem_owned = if sitem.is_none() {
		sbucket.get_sitem(s3_key).await.ok()
	} else {
		None
	};
	let sitem = sitem.or(sitem_owned.as_ref());

	if let (Ok(file_etag), Some(s3_etag)) = (compute_md5(file), sitem.and_then(|i| i.etag.as_deref())) {
		// We copy if the tags are different
		file_etag == s3_etag
	}
	// if no etag or object found, then, We do the copy
	else {
		false
	}
}
