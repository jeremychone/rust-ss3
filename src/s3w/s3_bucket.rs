use crate::Error;
use aws_sdk_s3::model::{CommonPrefix, Object};
use aws_sdk_s3::{ByteStream, Client};
use http::Uri;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

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
pub struct UploadOptions {
	recursive: bool,
}
impl Default for UploadOptions {
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
	/// - TODO - add support for directory
	/// - TODO - add support for recursive (default should be not recursive)
	/// - TODO - add support for rename (when prefix has same extension as file and src_path is a file)
	/// - DECIDE - if prefix should end with '/' to denote a directory prefix rather than a file rename (with not extension)
	///            This could be done with a options.force_prefix_as_file_key or something similar
	pub async fn upload_path(&self, src_path: &Path, prefix: &str, opts: UploadOptions) -> Result<(), Error> {
		// for now, do not support non file source
		if src_path.is_file() {
			let key = extract_dst_key(None, src_path, prefix)?;
			println!("Uploading  {} to s3://{}/{}", src_path.display(), self.name, key);
			self.upload_file(src_path, &key).await?;
		} else {
			return Err(Error::NotSupportedYet("Copy to s3 from directory"));
		}

		Ok(())
	}

	/// Lower level function that upload a single file to a fully resolved key
	async fn upload_file(&self, src_file: &Path, key: &str) -> Result<(), Error> {
		// Make sure it is a file
		if !src_file.is_file() {
			return Err(Error::NotSupportedYet("Copy from directory to s3"));
		}

		// BUILD - the src file info
		let mime_type = mime_guess::from_path(src_file).first_or_octet_stream().to_string();
		let body = ByteStream::from_path(&src_file).await?;

		// BUILD - aws s3 put request
		let builder = self
			.client
			.put_object()
			.key(key)
			.bucket(&self.name)
			.body(body)
			.content_type(mime_type);

		// EXECUTE - aws request
		let resp = builder.send().await?;

		Ok(())
	}
}

// endregion: S3Bucket

fn extract_dst_key(_base_dir: Option<&Path>, src_file: &Path, dst_prefix: &str) -> Result<String, Error> {
	let file_name = src_file
		.file_name()
		.and_then(|s| s.to_str())
		.ok_or_else(|| Error::FilePathNotFound(src_file.display().to_string()))?;

	// TODO - needs to find better way to build path
	let key = Path::new(dst_prefix).join(file_name);
	// TODO - Should throw an error if not 'valid' standard string
	let key = key.display().to_string();

	Ok(key)
}
