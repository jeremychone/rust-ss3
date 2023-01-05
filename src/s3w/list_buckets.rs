use crate::prelude::*;
use aws_sdk_s3::Client;

pub async fn list_buckets(client: &Client) -> Result<Vec<String>> {
	let buckets_output = client.list_buckets().send().await?;
	let buckets = buckets_output.buckets.unwrap_or_default();
	Ok(buckets.into_iter().flat_map(|b| b.name).collect())
}
