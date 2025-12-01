use super::{Inex, ListOptions, ListResult, PathType, SBucket, SItem, compute_dst_path, compute_inex, get_file_name, path_type};
use crate::{Error, Result};
use aws_sdk_s3::primitives::ByteStream;
use std::collections::{HashSet, VecDeque};
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::ops::Deref;
use std::path::Path;
// use tokio_stream::StreamExt;
use crate::s3w::support::{CpOptions, validate_over_for_file_dest};

/// "cp download" Implementation
impl SBucket {
	pub async fn download_path(&self, base_key: &str, dst_path: &Path, opts: CpOptions) -> Result<()> {
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
				if let Some(dst_dir) = dst_file.parent()
					&& !dst_dir.exists()
				{
					create_dir_all(dst_dir)?;
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

	async fn download_file(&self, key: &str, dst_file: &Path, opts: &CpOptions) -> Result<()> {
		match compute_inex(key, &opts.includes, &opts.excludes) {
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
						buf_writer.write_all(&bytes)?;
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
