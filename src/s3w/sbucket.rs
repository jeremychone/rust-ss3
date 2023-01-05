use aws_sdk_s3::Client;
use std::collections::HashSet;

pub struct SBucketConfig {
	pub default_ignore_upload_names: Option<HashSet<String>>,
}

pub struct SBucket {
	pub client: Client,
	pub name: String,
	pub default_ignore_upload_names: Option<HashSet<String>>,
}

impl SBucket {
	/// Constructor
	pub fn from_client_and_name(client: Client, name: String, config: Option<SBucketConfig>) -> SBucket {
		SBucket {
			client,
			name,
			default_ignore_upload_names: config.and_then(|d| d.default_ignore_upload_names),
		}
	}
}

impl SBucket {
	pub async fn exists(&self, key: &str) -> bool {
		let builder = self.client.head_object().key(key).bucket(&self.name);
		let resp = builder.send().await;
		resp.is_ok()
	}

	pub fn s3_url(&self, key: &str) -> String {
		format!("s3://{}/{key}", self.name)
	}
}
