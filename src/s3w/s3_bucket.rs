use aws_sdk_s3::model::Object;
use aws_sdk_s3::Client;
use regex::Regex;

use crate::Error;

// region:    S3Object
pub struct S3Object {
	pub key: String,
}

impl S3Object {
	fn from_object(obj: &Object) -> S3Object {
		let key = obj.key().unwrap_or_default().to_string();
		let size = obj.size;
		S3Object { key }
	}
}
// endregion: S3Object

pub struct S3Bucket {
	client: Client,
	name: String,
}

impl S3Bucket {
	pub fn from_client_and_name(client: Client, name: String) -> S3Bucket {
		S3Bucket { client, name }
	}
}

impl S3Bucket {
	pub async fn list(&self) -> Result<Vec<S3Object>, Error> {
		let resp = self.client.list_objects_v2().bucket(&self.name).send().await?;

		let data: Vec<S3Object> = resp.contents().unwrap_or_default().into_iter().map(S3Object::from_object).collect();

		Ok(data)
	}
}
