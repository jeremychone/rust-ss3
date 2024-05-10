use super::sitem::SItem;
use super::SBucket;
use crate::{Error, Result};

impl SBucket {
	pub async fn get_sitem(&self, key: &str) -> Result<SItem> {
		let builder = self.client.list_objects_v2().prefix(key).bucket(&self.name).set_max_keys(Some(1));
		let res = builder.send().await?;

		let obj = res
			.contents()
			.first()
			.ok_or_else(|| Error::S3ObjectNotFound { key: key.to_string() })?;

		Ok(SItem::from_object(obj))
	}
}
