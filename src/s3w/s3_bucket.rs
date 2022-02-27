use crate::Error;
use aws_sdk_s3::model::{CommonPrefix, Object};
use aws_sdk_s3::Client;

// region:    --- S3Item
#[derive(Debug)]
pub enum SItemType {
	Object,
	Prefix,
}

#[derive(Debug)]
pub struct SItem {
	pub typ: SItemType,
	pub key: String,
	pub size: i64,
}

// builders
impl SItem {
	pub fn from_object(obj: &Object) -> SItem {
		let key = obj.key().unwrap_or_default().to_string();
		let size = obj.size();
		SItem {
			key,
			typ: SItemType::Object,
			size,
		}
	}

	pub fn from_prefix(prefix: &CommonPrefix) -> SItem {
		let key = prefix.prefix().unwrap_or_default().to_string();
		SItem {
			key,
			typ: SItemType::Prefix,
			size: 0,
		}
	}

	pub fn from_prefix_str(prefix: &str) -> SItem {
		SItem {
			key: prefix.to_string(),
			typ: SItemType::Prefix,
			size: 0,
		}
	}
}
// endregion: --- S3Item

// region:    --- ListOptions
pub enum ListInfo {
	WithInfo,
	InfoOnly,
}

pub struct ListOptions {
	pub recursive: bool,
	pub continuation_token: Option<String>,
	pub info: Option<ListInfo>,
}
impl Default for ListOptions {
	fn default() -> Self {
		Self {
			recursive: false,
			continuation_token: None,
			info: None,
		}
	}
}

impl ListOptions {
	pub fn new(recursive: bool) -> ListOptions {
		ListOptions {
			recursive,
			..Default::default()
		}
	}
}
// endregion: --- ListOptions

// region:    ListResult
pub struct ListResult {
	pub prefixes: Vec<SItem>,
	pub objects: Vec<SItem>,
	pub next_continuation_token: Option<String>,
}
// endregion: --- ListResult

// region:    S3Bucket
pub struct SBucket {
	pub(super) client: Client,
	pub(super) name: String,
}

impl SBucket {
	pub fn from_client_and_name(client: Client, name: String) -> SBucket {
		SBucket { client, name }
	}
}

impl SBucket {
	pub async fn list(&self, prefix: &str, options: &ListOptions) -> Result<ListResult, Error> {
		// BUILD - the aws S3 list request
		let mut builder = self
			.client
			.list_objects_v2()
			.prefix(prefix)
			.bucket(&self.name)
			.set_continuation_token(options.continuation_token.clone());

		if !options.recursive {
			builder = builder.delimiter("/");
		}

		// EXECUTE - the AWS S3 request
		let resp = builder.send().await?;

		// get the prefixes
		let prefixes: Vec<SItem> = resp
			.common_prefixes()
			.unwrap_or_default()
			.into_iter()
			.map(SItem::from_prefix)
			.collect();

		// get the objects
		let objects: Vec<SItem> = resp.contents().unwrap_or_default().into_iter().map(SItem::from_object).collect();
		let next_continuation_token = resp.next_continuation_token().map(|t| t.to_string());

		Ok(ListResult {
			prefixes,
			objects,
			next_continuation_token,
		})
	}

	pub async fn exists(&self, key: &str) -> bool {
		let builder = self.client.head_object().key(key).bucket(&self.name);
		let resp = builder.send().await;
		resp.is_ok()
	}

	pub fn s3_url(&self, key: &str) -> String {
		format!("s3://{}/{key}", self.name)
	}
}

// endregion: S3Bucket
