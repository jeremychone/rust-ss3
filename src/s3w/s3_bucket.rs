use crate::Error;
use aws_sdk_s3::model::{CommonPrefix, Object};
use aws_sdk_s3::{ByteStream, Client};
use pathdiff::diff_paths;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use tokio_stream::StreamExt;
use walkdir::WalkDir;

// region:    S3Item
pub enum SItemType {
	Object,
	Prefix,
}

pub struct SItem {
	pub typ: SItemType,
	pub key: String,
}

// builders
impl SItem {
	fn from_object(obj: &Object) -> SItem {
		let key = obj.key().unwrap_or_default().to_string();
		SItem {
			key,
			typ: SItemType::Object,
		}
	}

	fn from_prefix(prefix: &CommonPrefix) -> SItem {
		let key = prefix.prefix().unwrap_or_default().to_string();
		SItem {
			key,
			typ: SItemType::Prefix,
		}
	}
}
// endregion: S3Item

// region:    ListOptions
pub struct ListOptions {
	recursive: bool,
	prefix: String,
}

impl ListOptions {
	pub fn new(recursive: bool, prefix: &str) -> ListOptions {
		ListOptions {
			recursive,
			prefix: prefix.to_string(),
		}
	}
}
// endregion: ListOptions

// region:    UploadOptions
pub struct CpOptions {
	pub recursive: bool,
}
impl Default for CpOptions {
	fn default() -> Self {
		Self { recursive: false }
	}
}
// endregion: UploadOptions

// region:    S3Bucket
pub struct SBucket {
	client: Client,
	name: String,
}

impl SBucket {
	pub fn from_client_and_name(client: Client, name: String) -> SBucket {
		SBucket { client, name }
	}
}

impl SBucket {
	pub async fn list(&self, options: &ListOptions) -> Result<Vec<SItem>, Error> {
		// BUILD - the aws S3 list request
		let mut builder = self.client.list_objects_v2().prefix(&options.prefix).bucket(&self.name);

		if !options.recursive {
			builder = builder.delimiter("/");
		}

		// EXECUTE - the AWS S3 request
		let resp = builder.send().await?;

		// PARSE - reponse data
		// first get the prefixes
		let mut data: Vec<SItem> = resp
			.common_prefixes()
			.unwrap_or_default()
			.into_iter()
			.map(SItem::from_prefix)
			.collect();
		// then, get the Objects
		let objects: Vec<SItem> = resp.contents().unwrap_or_default().into_iter().map(SItem::from_object).collect();
		// concatenate with objects
		data.extend(objects);

		Ok(data)
	}

	/// Upload a file or files in a directory into a this bucket at the given prefix. By default it wont be recursive.
	/// - TODO - add support for rename (when prefix has same extension as file and src_path is a file)
	/// - DECIDE - if prefix should end with '/' to denote a directory prefix rather than a file rename (with not extension)
	///            This could be done with a options.force_prefix_as_file_key or something similar
	pub async fn upload_path(&self, src_path: &Path, prefix: &str, opts: CpOptions) -> Result<(), Error> {
		// When copy only a given file
		if src_path.is_file() {
			let key = compute_dst_key(None, src_path, prefix, true)?;
			self.upload_file(src_path, &key).await?;
		}
		// When copying all file from a directory (recursive if opts.recursive)
		else if src_path.is_dir() {
			let max_depth = if opts.recursive { ::std::usize::MAX } else { 1 };
			let walker = WalkDir::new(src_path).max_depth(max_depth).into_iter();
			for entry in walker.filter_map(|e| e.ok()) {
				let file = entry.path();
				if file.is_file() {
					// When
					let key = compute_dst_key(Some(src_path), file, prefix, false)?;
					self.upload_file(file, &key).await?;
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
	async fn upload_file(&self, src_file: &Path, key: &str) -> Result<(), Error> {
		// Make sure it is a file
		if !src_file.is_file() {
			panic!("sbucket.upload_file should only get a file object. Code error.");
		}

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

		Ok(())
	}

	pub async fn download_path(&self, key: &str, dst_path: &Path, _opts: CpOptions) -> Result<(), Error> {
		let key_path = Path::new(key);
		match (is_path_file_like(key_path), is_path_file_like(dst_path)) {
			// S3 File to Path File or Dir
			(true, dst_is_file) => {
				// compute the dst_file
				let file_name = get_file_name(key_path)?;
				let dst_file = if dst_is_file {
					dst_path.to_path_buf()
				} else {
					dst_path.join(file_name)
				};
				// create parent
				if let Some(dst_dir) = dst_file.parent() {
					if !dst_dir.exists() {
						create_dir_all(dst_dir)?;
					}
				}
				// perform the copy
				self.download_file(key, &dst_file).await?;
			}
			// S3 Dir Path dir
			(false, false) => return Err(Error::NotSupportedYet("S3 Dir to Path Dir")),
			// S3 dir to file (NOT supported)
			(false, true) => return Err(Error::NotSupportedYet("S3 Dir to Path File")),
		}
		Ok(())
	}

	async fn download_file(&self, key: &str, dst_file: &Path) -> Result<(), Error> {
		println!("->> download_path {} to {}", key, dst_file.to_string_lossy());
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

		Ok(())
	}
}

// endregion: S3Bucket

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
			(Some(src_ext), Some(dst_ext)) => {
				if src_ext == dst_ext {
					true
				} else {
					false
				}
			}
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

/// Determine if a key a directory (end with '/')
fn get_file_name(path: &Path) -> Result<String, Error> {
	path
		.file_name()
		.and_then(|s| s.to_str().map(|v| v.to_string()))
		.ok_or_else(|| Error::InvalidPath(path.to_string_lossy().to_string()))
}

fn is_path_file_like(path: &Path) -> bool {
	path.extension().is_some()
}
