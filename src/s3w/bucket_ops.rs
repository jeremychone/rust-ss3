use crate::Result;
use aws_sdk_s3::Client;

pub async fn list_buckets(client: &Client) -> Result<Vec<String>> {
	let buckets_output = client.list_buckets().send().await?;
	let buckets = buckets_output.buckets.unwrap_or_default();
	Ok(buckets.into_iter().flat_map(|b| b.name).collect())
}

pub async fn create_bucket(client: &Client, bucket_name: &str) -> Result<Option<String>> {
	let bucket_output = client.create_bucket().bucket(bucket_name).send().await?;
	let result = bucket_output.location().map(|s| s.to_string());
	Ok(result)
}

pub async fn delete_bucket(client: &Client, bucket_name: &str) -> Result<()> {
	client.delete_bucket().bucket(bucket_name).send().await?;
	Ok(())
}
