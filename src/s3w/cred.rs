use crate::{Error, DEFAULT_UPLOAD_IGNORE_FILES};
use aws_config::profile::profile_file::ProfileFiles;
use aws_config::profile::Profile;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::{Client, Credentials, Endpoint, Region};
use aws_types::credentials::SharedCredentialsProvider;
use aws_types::os_shim_internal::{Env, Fs};
use http::Uri;
use std::collections::HashSet;
use std::env;
use std::str::FromStr;

use super::s3_bucket::SBucket;
use super::SBucketConfig;

// Default AWS environement names (used as last fallback)
const AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
const AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";

#[derive(Debug)]
struct AwsCred {
	key_id: String,
	key_secret: String,
	region: Option<String>,
	endpoint: Option<String>,
}

enum CredKey {
	Id,
	Secret,
	Region,
	Endpoint,
}

impl CredKey {
	fn env_part(&self) -> &'static str {
		match self {
			CredKey::Id => "KEY_ID",
			CredKey::Secret => "KEY_SECRET",
			CredKey::Region => "REGION",
			CredKey::Endpoint => "ENDPOINT",
		}
	}
}

enum EnvType {
	Profile,
	Bucket,
}

impl EnvType {
	fn env_part(&self) -> &'static str {
		match self {
			EnvType::Profile => "SS3_PROFILE",
			EnvType::Bucket => "SS3_BUCKET",
		}
	}
}

pub async fn get_sbucket(profile: Option<&str>, bucket: &str) -> Result<SBucket, Error> {
	let client = new_s3_client(profile, bucket).await?;
	let default_ignore_files = HashSet::from_iter(DEFAULT_UPLOAD_IGNORE_FILES.map(String::from));
	let config = SBucketConfig {
		default_ignore_upload_names: Some(default_ignore_files),
	};
	let sbucket = SBucket::from_client_and_name(client, bucket.to_string(), Some(config));

	Ok(sbucket)
}

async fn new_s3_client(profile: Option<&str>, bucket: &str) -> Result<Client, Error> {
	let cred = load_aws_cred(profile, bucket).await?;
	let client = client_from_cred(cred)?;
	Ok(client)
}

fn client_from_cred(aws_cred: AwsCred) -> Result<Client, Error> {
	let AwsCred {
		key_id,
		key_secret,
		region,
		endpoint,
	} = aws_cred;

	let cred = Credentials::new(key_id, key_secret, None, None, "loaded-from-config-or-env");

	if let (None, None) = (&region, &endpoint) {
		return Err(Error::MissingConfigMustHaveEndpointOrRegion);
	}

	let mut builder = Builder::new().credentials_provider(SharedCredentialsProvider::new(cred));

	if let Some(endpoint) = endpoint {
		builder = builder.endpoint_resolver(Endpoint::immutable(Uri::from_str(&endpoint).unwrap()));
		// WORKAROUND - Right now the aws-sdk throw a NoRegion on .send if not region even if we have a endpoint
		builder = builder.region(Region::new("endpoint-region"));
	}

	if let Some(region) = region {
		builder = builder.region(Region::new(region));
	}

	let config = builder.build();
	let client = Client::from_conf(config);
	Ok(client)
}

/// Load the AwsCred from
/// - First check if SS3_BUCKET_... envs
/// - If not, if Profile,
///    - first try the SS3_PROFILE_... envs,
///    - then try standard aws config files
///    - if still not found, error
/// - if no profile,
///    - try SS3_BUCKET_... envs
///    - try the default AWS env keys
///    - if still not found, error
async fn load_aws_cred(profile: Option<&str>, bucket: &str) -> Result<AwsCred, Error> {
	// first, try to get it from the SS3_BUCKET_bucket_name_KEY_ID, ... environments
	let mut cred_result = load_aws_cred_from_ss3_bucket_env(bucket).await;

	// if not found
	if cred_result.is_err() {
		// if we have a profile defined
		if let Some(profile) = profile {
			// try to get it from the SS3_PROFILE_profile_name_KEY_ID, ... environments
			cred_result = load_aws_cred_from_ss3_profile_env(profile).await;

			// then, try to get it frmo the aws config files
			if cred_result.is_err() {
				cred_result = load_aws_cred_from_aws_profile_configs(profile).await;
			}
		}
	}

	// if still not found, try the default AWS env
	if cred_result.is_err() {
		cred_result = load_aws_cred_from_default_aws_env().await;
	}

	cred_result.map_err(|_| Error::NoCredentialsFoundForBucket(bucket.to_string()))
}

/// Attempt to create AwsCred from SS3 BUCKET environment variables
/// - `SS3_BUCKET_bucket_name_KEY_ID`
/// - `SS3_BUCKET_bucket_name_KEY_SECRET`
/// - `SS3_BUCKET_bucket_name_REGION`
async fn load_aws_cred_from_ss3_bucket_env(bucket: &str) -> Result<AwsCred, Error> {
	let key_id = get_env(&get_env_name(EnvType::Bucket, CredKey::Id, bucket))?;
	let key_secret = get_env(&get_env_name(EnvType::Bucket, CredKey::Secret, bucket))?;
	let region = get_env(&get_env_name(EnvType::Bucket, CredKey::Region, bucket)).ok();
	let endpoint = get_env(&get_env_name(EnvType::Bucket, CredKey::Endpoint, bucket)).ok();

	Ok(AwsCred {
		key_id,
		key_secret,
		region,
		endpoint,
	})
}

/// Attempt to create AwsCred from SS3 PROFILE environment variables
/// - `SS3_PROFILE_profile_name_KEY_ID`
/// - `SS3_PROFILE_profile_name_KEY_SECRET`
/// - `SS3_PROFILE_profile_name_REGION`
async fn load_aws_cred_from_ss3_profile_env(profile: &str) -> Result<AwsCred, Error> {
	let key_id = get_env(&get_env_name(EnvType::Profile, CredKey::Id, profile))?;
	let key_secret = get_env(&get_env_name(EnvType::Profile, CredKey::Secret, profile))?;
	let region = get_env(&get_env_name(EnvType::Profile, CredKey::Region, profile)).ok();
	let endpoint = get_env(&get_env_name(EnvType::Profile, CredKey::Endpoint, profile)).ok();

	Ok(AwsCred {
		key_id,
		key_secret,
		region,
		endpoint,
	})
}

async fn load_aws_cred_from_aws_profile_configs(profile_str: &str) -> Result<AwsCred, Error> {
	let (fs, ev) = (Fs::real(), Env::default());
	let profiles = aws_config::profile::load(&fs, &ev, &ProfileFiles::default()).await;
	if let Ok(profiles) = profiles {
		if let Some(profile) = profiles.get_profile(profile_str) {
			let key_id = get_profile_value(profile, "aws_access_key_id")?;
			let key_secret = get_profile_value(profile, "aws_secret_access_key")?;
			let region = get_profile_value(profile, "region").ok();

			return Ok(AwsCred {
				key_id,
				key_secret,
				region,
				endpoint: None, // because aws configs only
			});
		}
	}

	Err(Error::NoCredentialsForProfile(profile_str.to_string()))
}

async fn load_aws_cred_from_default_aws_env() -> Result<AwsCred, Error> {
	let key_id = get_env(AWS_ACCESS_KEY_ID)?;
	let key_secret = get_env(AWS_SECRET_ACCESS_KEY)?;
	let region = get_env(AWS_DEFAULT_REGION).ok();

	Ok(AwsCred {
		key_id,
		key_secret,
		region,
		endpoint: None,
	})
}

// region:    Utils
fn get_env_name(typ: EnvType, key: CredKey, name: &str) -> String {
	let name = name.replace('-', "_");
	format!("{}_{}_{}", typ.env_part(), name, key.env_part())
}

fn get_profile_value(profile: &Profile, key: &str) -> Result<String, Error> {
	match profile.get(key) {
		Some(value) => Ok(value.to_string()),
		None => Err(Error::NoCredentialConfig(key.to_string())),
	}
}

fn get_env(name: &str) -> Result<String, Error> {
	match env::var(name) {
		Ok(v) => Ok(v),
		Err(_) => Err(Error::NoCredentialEnv(name.to_string())),
	}
}
// endregion: Utils
