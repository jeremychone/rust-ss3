use super::{compute_dst_key, compute_inex, Inex, SBucket};
use crate::{s, Error, Result};
use aws_sdk_s3::primitives::ByteStream;
use std::path::Path;
// use tokio_stream::StreamExt;
use crate::s3w::support::{validate_over_for_s3_dest, CpOptions, OverMode};
use crate::s3w::SItemsCache;
use walkdir::WalkDir;

/// "cp upload" Implementation
impl SBucket {
	/// Upload a file or files in a directory into a this bucket at the given prefix. By default it wont be recursive.
	///
	/// - IMPORANT - Right now, a leading '/' on prefix will be stripped and act if there are now. All prefix are from root.
	///
	/// - TODO - add support for rename (when prefix has same extension as file and src_path is a file)
	pub async fn upload_path(&self, src_path: impl AsRef<Path>, prefix: &str, opts: CpOptions) -> Result<()> {
		let src_path = src_path.as_ref();

		// When copy only a given file
		if src_path.is_file() {
			let key = compute_dst_key(None, src_path, prefix, true)?;
			self.upload_file(src_path, &key, &opts, None).await?;
		}
		// When copying all file from a directory (recursive if opts.recursive)
		else if src_path.is_dir() {
			let max_depth = if opts.recursive { usize::MAX } else { 1 };
			let walker = WalkDir::new(src_path).max_depth(max_depth).into_iter();

			let sitems_cache = if matches!(opts.over, OverMode::Etag) {
				Some(self.sitems_cache(Some(prefix)).await?)
			} else {
				None
			};

			for entry in walker.filter_map(|e| e.ok()) {
				let file = entry.path();

				if file.is_file() {
					let key = compute_dst_key(Some(src_path), file, prefix, false)?;
					self.upload_file(file, &key, &opts, sitems_cache.as_ref()).await?;
				}
			}
		}
		// If not file or dir, we fail for now.
		// TODO: Needs to decide what to do with symlink
		else {
			return Err(Error::FilePathNotFound(src_path.to_string_lossy().to_string()));
		}

		Ok(())
	}

	/// Lower level function that upload a single file to a fully resolved key
	async fn upload_file(&self, src_file: &Path, key: &str, opts: &CpOptions, sitems_cache: Option<&SItemsCache>) -> Result<()> {
		// --- Make sure it is a file
		if !src_file.is_file() {
			panic!("CODE-ERROR - sbucket.upload_file should only get a file object. Code error.");
		}

		if let Some(file_name) = src_file.file_name().and_then(|f| f.to_str())
			&& let Some(ignore_set) = &self.default_ignore_upload_names
			&& ignore_set.contains(file_name)
		{
			if opts.show_skip {
				println!("{:13} {file_name}", "Skip (default)");
			}

			return Ok(());
		}

		if let Some(src_file_str) = src_file.to_str() {
			match compute_inex(key, &opts.includes, &opts.excludes) {
				Inex::Include => {
					if validate_over_for_s3_dest(self, key, src_file, opts, sitems_cache).await? {
						// BUILD - the src file info
						let mime_type = match (&opts.noext_ct, src_file.extension()) {
							(Some(noext_ct), None) => s!(noext_ct),
							_ => mime_guess::from_path(src_file).first_or_octet_stream().to_string(),
						};
						let body = ByteStream::from_path(&src_file).await?;

						println!(
							"{:13} {:50} --> {}   (content-type: {})",
							"Uploading",
							src_file.display(),
							self.s3_url(key),
							mime_type
						);

						// BUILD - aws s3 put request
						let builder = self
							.client
							.put_object()
							.key(key)
							.bucket(&self.name)
							.body(body)
							.content_type(mime_type);

						// EXECUTE - aws request
						builder.send().await?;
					} else if opts.show_skip {
						let msg = format!("Skip ({})", opts.over.label());
						println!("{:13} - {}", msg, self.s3_url(key));
					}
				}
				Inex::ExcludeInExclude => println!("{:20} {src_file_str}", "Excludes"),
				// if exclude because not in include, then, quiet
				Inex::ExcludeNotInInclude => (),
			}
		}

		Ok(())
	}
}

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/test-cp-upload.rs"]
mod tests;

// endregion: --- Tests
