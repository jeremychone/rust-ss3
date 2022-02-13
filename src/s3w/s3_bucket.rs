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
}

// builders
impl SItem {
	pub fn from_object(obj: &Object) -> SItem {
		let key = obj.key().unwrap_or_default().to_string();
		SItem {
			key,
			typ: SItemType::Object,
		}
	}

	pub fn from_prefix(prefix: &CommonPrefix) -> SItem {
		let key = prefix.prefix().unwrap_or_default().to_string();
		SItem {
			key,
			typ: SItemType::Prefix,
		}
	}

	pub fn from_prefix_str(prefix: &str) -> SItem {
		SItem {
			key: prefix.to_string(),
			typ: SItemType::Prefix,
		}
	}
}
// endregion: --- S3Item

// region:    --- ListOptions
pub struct ListOptions {
	recursive: bool,
}
impl Default for ListOptions {
	fn default() -> Self {
		Self { recursive: false }
	}
}

impl ListOptions {
	pub fn new(recursive: bool) -> ListOptions {
		ListOptions { recursive }
	}
}
// endregion: --- ListOptions

// region:    ListResult
pub struct ListResult {
	pub prefixes: Vec<SItem>,
	pub objects: Vec<SItem>,
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
		let mut builder = self.client.list_objects_v2().prefix(prefix).bucket(&self.name);

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

		Ok(ListResult { prefixes, objects })
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
