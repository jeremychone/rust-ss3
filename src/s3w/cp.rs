use crate::Error;

use super::s3_bucket::{SBucket, SItem};
use super::{compute_dst_key, compute_dst_path, get_file_name, path_type, ListOptions, ListResult, PathType};
use aws_sdk_s3::ByteStream;
use globset::GlobSet;
use std::collections::{HashSet, VecDeque};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::Path;
use tokio_stream::StreamExt;
use walkdir::WalkDir;

pub struct CpOptions {
	pub recursive: bool,
	pub excludes: Option<GlobSet>,
}

impl Default for CpOptions {
	fn default() -> Self {
		Self {
			recursive: false,
			excludes: None,
		}
	}
}

/// "cp upload" Implementation
impl SBucket {
	/// Upload a file or files in a directory into a this bucket at the given prefix. By default it wont be recursive.
	/// - TODO - add support for rename (when prefix has same extension as file and src_path is a file)
	/// - DECIDE - if prefix should end with '/' to denote a directory prefix rather than a file rename (with not extension)
	///            This could be done with a options.force_prefix_as_file_key or something similar
	pub async fn upload_path(&self, src_path: &Path, prefix: &str, opts: CpOptions) -> Result<(), Error> {
		// When copy only a given file
		if src_path.is_file() {
			let key = compute_dst_key(None, src_path, prefix, true)?;
			self.upload_file(src_path, &key, &opts).await?;
		}
		// When copying all file from a directory (recursive if opts.recursive)
		else if src_path.is_dir() {
			let max_depth = if opts.recursive { ::std::usize::MAX } else { 1 };
			let walker = WalkDir::new(src_path).max_depth(max_depth).into_iter();
			for entry in walker.filter_map(|e| e.ok()) {
				let file = entry.path();
				if file.is_file() {
					let key = compute_dst_key(Some(src_path), file, prefix, false)?;
					self.upload_file(file, &key, &opts).await?;
				}
			}
		}
		// if not file or dir, we fail for now. Needs to decide what to do with symlink
		else {
			return Err(Error::FilePathNotFound(src_path.to_string_lossy().to_string()));
		}

		Ok(())
	}

	/// Lower level function that upload a single file to a fully resolved key
	async fn upload_file(&self, src_file: &Path, key: &str, opts: &CpOptions) -> Result<(), Error> {
		// Make sure it is a file
		if !src_file.is_file() {
			panic!("sbucket.upload_file should only get a file object. Code error.");
		}

		if let Some(src_file_str) = src_file.to_str() {
			if accept_path(&src_file_str, &opts) {
				// BUILD - the src file info
				let mime_type = mime_guess::from_path(src_file).first_or_octet_stream().to_string();
				let body = ByteStream::from_path(&src_file).await?;

				println!(
					"Uploading  {:40} to   s3://{}/{:40} (content-type: {})",
					src_file.display(),
					self.name,
					key,
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
			} else {
				println!("Excludes   {src_file_str}");
			}
		}

		Ok(())
	}
}

/// "cp download" Implementation
impl SBucket {
	pub async fn download_path(&self, base_key: &str, dst_path: &Path, opts: CpOptions) -> Result<(), Error> {
		let key_path = Path::new(base_key);
		match (path_type(key_path), path_type(dst_path)) {
			// S3 File to Path File or Dir
			(PathType::File, dst_type) => {
				// compute the dst_file
				let file_name = get_file_name(key_path)?;
				let dst_file = match dst_type {
					PathType::File => dst_path.to_path_buf(),
					PathType::Dir => dst_path.join(file_name),
				};

				// create parent
				if let Some(dst_dir) = dst_file.parent() {
					if !dst_dir.exists() {
						create_dir_all(dst_dir)?;
					}
				}
				// perform the copy
				self.download_file(base_key, &dst_file, &opts).await?;
			}
			// S3 Dir Path dir
			(PathType::Dir, PathType::Dir) => {
				// prefix queue to avoid recursive function calls (leaner & simpler)
				let mut prefix_queue: VecDeque<SItem> = VecDeque::new();
				prefix_queue.push_front(SItem::from_prefix_str(base_key));

				// default options for the list(...) calls
				// Note: For now, the list(...) does not do the recursive calls, but folder by folder
				//       pros - assuming a folder does not have more than the fetch limit, it will scale well
				//       cons - will require to make list request per folder if the donload_path is recursive
				let list_opts = ListOptions::default();

				// cheap optimization to not check parent dir all the time
				let mut dir_exist_set: HashSet<String> = HashSet::new();

				while let Some(prefix) = prefix_queue.pop_front() {
					// get the objects and prefixes
					let ListResult { prefixes, objects } = self.list(&prefix.key, &list_opts).await?;

					// download the objects of this prefix
					for item in objects.iter() {
						let dst_file = compute_dst_path(base_key, &item.key, dst_path)?;

						if let Some(dst_file_parent) = dst_file.parent() {
							let parent_dir_string = dst_file_parent.to_string_lossy();
							if !dir_exist_set.contains(parent_dir_string.deref()) || !dst_file_parent.exists() {
								create_dir_all(dst_file_parent)?;
								dir_exist_set.insert(parent_dir_string.to_string());
							}
						}

						self.download_file(&item.key, &dst_file, &opts).await?;
					}

					// if the download is recursive ass those prefixes to the prefix_queue
					if opts.recursive {
						prefix_queue.extend(prefixes);
					}
				}
			}
			// S3 dir to file (NOT supported)
			(PathType::Dir, PathType::File) => return Err(Error::NotSupported("S3 Dir to Path File")),
		}
		Ok(())
	}

	async fn download_file(&self, key: &str, dst_file: &Path, opts: &CpOptions) -> Result<(), Error> {
		if accept_path(key, opts) {
			println!("Downloading s3://{}/{:40} to {}", self.name, key, dst_file.to_string_lossy());
			// BUILD - aws s3 get request
			let builder = self.client.get_object().bucket(&self.name).key(key);

			let resp = builder.send().await?;

			// Streaming
			let mut data: ByteStream = resp.body;
			let file = File::create(dst_file)?;
			let mut buf_writer = BufWriter::new(file);
			while let Some(bytes) = data.try_next().await? {
				buf_writer.write(&bytes)?;
			}
			buf_writer.flush()?;
		} else {
			println!("Excludes    s3://{}/{:40}", self.name, key);
		}

		Ok(())
	}
}

/// validate that the file path or s3 key pass the opts excludes / includes rules.
fn accept_path(path: &str, opts: &CpOptions) -> bool {
	match &opts.excludes {
		Some(excludes) => {
			let m = excludes.matches(path);
			m.len() == 0 // if no match, true
		}
		None => true,
	}
}
