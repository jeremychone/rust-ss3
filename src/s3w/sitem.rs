use aws_sdk_s3::model::{CommonPrefix, Object};

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
