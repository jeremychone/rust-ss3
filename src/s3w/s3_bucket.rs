use crate::Error;
use aws_sdk_s3::model::{CommonPrefix, Object};
use aws_sdk_s3::Client;

// region:    S3Object
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
// endregion: S3Object

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
		let mut builder = self.client.list_objects_v2().prefix(&options.prefix).bucket(&self.name);

		if !options.recursive {
			builder = builder.delimiter("/");
		}

		let resp = builder.send().await?;
		let mut data: Vec<SItem> = resp
			.common_prefixes()
			.unwrap_or_default()
			.into_iter()
			.map(SItem::from_prefix)
			.collect();
		let objects: Vec<SItem> = resp.contents().unwrap_or_default().into_iter().map(SItem::from_object).collect();

		data.extend(objects);

		Ok(data)
	}
}

// endregion: S3Bucket
