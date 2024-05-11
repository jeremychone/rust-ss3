pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // For early dev.

use crate::s3w::bucket_ops::create_bucket;
use crate::s3w::cred::client_from_cred;
use crate::s3w::{get_sbucket_from_cred, ListOptions, SBucket};

const TEST_BUCKET: &str = "unit-test-bucket";

pub async fn new_test_ss3_bucket() -> Result<SBucket> {
	let cred = crate::s3w::cred::AwsCred {
		key_id: "minio".to_string(),
		key_secret: "miniominio".to_string(),
		region: None,
		endpoint: Some("http://127.0.0.1:9000".to_string()),
	};

	let client = client_from_cred(cred.clone())?;
	let res = create_bucket(&client, TEST_BUCKET).await;
	if let Err(err) = res {
		match err {
			crate::Error::AwsSdkErrorWrapper { code, message } => {
				if code != "BucketAlreadyOwnedByYou" {
					return Err(crate::Error::AwsSdkErrorWrapper { code, message }.into());
				}
			}
			other => return Err(other.into()),
		}
		// println!("Error while new_test_ss3_bucket create_bucket: {:?}", err);
	}

	let sbucket = get_sbucket_from_cred(cred, TEST_BUCKET).await?;

	Ok(sbucket)
}

pub async fn delete_s3_folder(sbucket: &SBucket, s3_key: &str) -> Result<()> {
	for obj in sbucket.list(s3_key, &ListOptions::new(true)).await?.objects {
		sbucket.delete_object(&obj.key).await?;
	}
	Ok(())
}
