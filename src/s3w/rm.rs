use crate::Result;
use crate::s3w::SBucket;

impl SBucket {
	pub async fn delete_object(&self, key: &str) -> Result<()> {
		let builder = self.client.delete_object().bucket(&self.name).key(key);

		builder.send().await?;

		Ok(())
	}
}
