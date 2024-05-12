use aws_sdk_s3::types::{CommonPrefix, Object};

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
	pub etag: Option<String>,
	pub size: i64,
}

/// Constructors
impl SItem {
	pub fn from_object(obj: &Object) -> SItem {
		let key = obj.key().unwrap_or_default().to_string();
		let size = obj.size();
		let etag = obj
			.e_tag()
			.map(|etag| etag.strip_prefix('"').unwrap_or(etag).strip_suffix('"').unwrap_or(etag).to_string());
		SItem {
			key,
			typ: SItemType::Object,
			size: size.unwrap_or(-1),
			etag,
		}
	}

	pub fn from_prefix(prefix: &CommonPrefix) -> SItem {
		let key = prefix.prefix().unwrap_or_default().to_string();
		SItem {
			key,
			typ: SItemType::Prefix,
			size: 0,
			etag: None,
		}
	}

	pub fn from_prefix_str(prefix: &str) -> SItem {
		SItem {
			key: prefix.to_string(),
			typ: SItemType::Prefix,
			size: 0,
			etag: None,
		}
	}
}

// endregion: --- S3Item
