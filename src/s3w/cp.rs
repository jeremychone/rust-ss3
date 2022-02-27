use super::s3_bucket::{SBucket, SItem};
use super::{compute_dst_key, compute_dst_path, get_file_name, path_type, ListOptions, ListResult, PathType};
use crate::Error;
use aws_sdk_s3::types::ByteStream;
use globset::GlobSet;
use std::collections::{HashSet, VecDeque};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::Path;
use tokio_stream::StreamExt;
use walkdir::WalkDir;

#[derive(Debug)]
pub enum OverMode {
	Write,
	Skip,
	Fail,
}
impl Default for OverMode {
	fn default() -> Self {
		OverMode::Skip
	}
}

pub struct CpOptions {
	pub recursive: bool,
	pub excludes: Option<GlobSet>,
	pub includes: Option<GlobSet>,
	pub over: OverMode,
}

impl Default for CpOptions {
	fn default() -> Self {
		Self {
			recursive: false,
			excludes: None,
			includes: None,
			over: OverMode::default(),
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
			match validate_inex_rules(key, opts) {
				Inex::Include => {
					if validate_over_for_s3_dest(self, key, &opts).await? {
						// BUILD - the src file info
						let mime_type = mime_guess::from_path(src_file).first_or_octet_stream().to_string();
						let body = ByteStream::from_path(&src_file).await?;

						self.exists(key).await;

						println!(
							"{:15} {:50} --> {}   (content-type: {})",
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
					} else {
						println!("{:15} {}", "Skip (exists)", self.s3_url(key));
					}
				}
				Inex::ExcludeInExclude => println!("{:15} {src_file_str}", "Excludes"),
				// if exclude because not in include, then, quiet
				Inex::ExcludeNotInInclude => (),
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

				// cheap optimization to not check parent dir all the time
				let mut dir_exist_set: HashSet<String> = HashSet::new();

				// Note: For now, the list(...) does not do the recursive calls, but folder by folder
				//       pros - assuming a folder does not have more than the fetch limit, it will scale well
				//       cons - will require to make list request per folder if the donload_path is recursive
				let mut list_opts = ListOptions::new(false);

				while let Some(prefix) = prefix_queue.pop_front() {
					let mut continuation_token: Option<String> = None;

					while {
						list_opts.continuation_token = continuation_token;

						// get the objects and prefixes
						let ListResult {
							prefixes,
							objects,
							next_continuation_token,
						} = self.list(&prefix.key, &list_opts).await?;
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

						// if the download is recursive add those prefixes to the prefix_queue
						if opts.recursive {
							prefix_queue.extend(prefixes);
						}

						// continuation if needed
						continuation_token = next_continuation_token;
						continuation_token.is_some()
					} {}
				}
			}
			// S3 dir to file (NOT supported)
			(PathType::Dir, PathType::File) => return Err(Error::NotSupported("S3 Dir to Path File")),
		}
		Ok(())
	}

	async fn download_file(&self, key: &str, dst_file: &Path, opts: &CpOptions) -> Result<(), Error> {
		match validate_inex_rules(key, opts) {
			Inex::Include => {
				if validate_over_for_file_dest(dst_file, opts)? {
					println!(
						"{:20} s3://{}/{:40} to {}",
						"Downloading",
						self.name,
						key,
						dst_file.to_string_lossy()
					);
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
					println!("{:20} {}", "Skip (exists)", dst_file.display());
				}
			}
			Inex::ExcludeInExclude => {
				println!("{:20} {}", "Excludes", self.s3_url(key));
			}
			// if there is an include and not in incluse, we silently skip it
			Inex::ExcludeNotInInclude => (),
		}
		Ok(())
	}
}

/// Inclusion/Exclusion result
enum Inex {
	Include,
	ExcludeInExclude,
	ExcludeNotInInclude,
}

/// validate the Include / Exclusion rules
fn validate_inex_rules(path: &str, opts: &CpOptions) -> Inex {
	// Note: Those match_... will have 3 states, None (if no rule), Some(true), Some(false)
	let match_include = opts.includes.as_ref().map(|gs| gs.matches(path).len() > 0);
	let match_exclude = opts.excludes.as_ref().map(|gs| gs.matches(path).len() > 0);

	match (match_include, match_exclude) {
		// if passe the include gate (no include rule or matched it) and not in eventual exclude
		(None | Some(true), None | Some(false)) => Inex::Include,
		// passed the include gate, but is explicity excluded
		(None | Some(true), Some(true)) => Inex::ExcludeInExclude,
		// Did not pass the include gate
		(Some(false), _) => Inex::ExcludeNotInInclude,
	}
}

async fn validate_over_for_s3_dest(sbucket: &SBucket, key: &str, opts: &CpOptions) -> Result<bool, Error> {
	match opts.over {
		// if over: Write, then always true, we overwrite
		OverMode::Write => Ok(true),
		// if skip, then the opposite of the exists state
		OverMode::Skip => Ok(!sbucket.exists(key).await),
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

fn validate_over_for_file_dest(file: &Path, opts: &CpOptions) -> Result<bool, Error> {
	match opts.over {
		// if over: Write, then always true, we overwrite
		OverMode::Write => Ok(true),
		// if skip, then the opposite of the exists state
		OverMode::Skip => Ok(!file.exists()),
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
