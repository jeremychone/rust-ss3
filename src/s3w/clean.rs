use crate::s3w::{compute_dst_key, ListOptions, SBucket};
use crate::{Result, DEFAULT_UPLOAD_IGNORE_GLOBS};
use simple_fs::list_files;
use std::collections::HashSet;
use std::path::Path;

#[derive(Default, Clone)]
pub struct CleanOptions {
	pub force: bool,
}

impl SBucket {
	/// List the files to clean given the local path and base_s3_path
	/// IMPORTANT - For now, assume one self.list with no pagination
	///      TODO - Fix the
	pub async fn list_to_clean(&self, local_path: impl AsRef<Path>, base_s3_path: &str) -> Result<Vec<String>> {
		let local_path = local_path.as_ref();

		// -- get the sitems from the s3 (assume it will be ok to be within one page )
		let sitems = self.list(base_s3_path, &ListOptions::new(true)).await?.objects;

		// -- List files and target s3 keys Set
		let files = list_files(local_path, Some(&["**/*"]), Some(DEFAULT_UPLOAD_IGNORE_GLOBS))?;
		// the target s3 keys from the files
		let target_key_set: HashSet<String> = files
			.iter()
			.map(|f| compute_dst_key(Some(local_path), f.path(), base_s3_path, false))
			.collect::<Result<HashSet<_>>>()?;

		// -- Build the result
		let mut s3_keys: Vec<String> = Vec::new();
		// for each remote s3 item, if not in local target key set then we should remove.
		for sitem in sitems {
			if !target_key_set.contains(&sitem.key) {
				s3_keys.push(sitem.key)
			}
		}
		Ok(s3_keys)
	}
}
