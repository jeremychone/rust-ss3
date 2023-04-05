use crate::s3w::SBucket;
use crate::s3w::SBucketConfig;
use crate::{Error, Result, DEFAULT_UPLOAD_IGNORE_FILES};
use aws_config::profile::profile_file::ProfileFiles;
use aws_config::profile::Profile;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client;
use aws_types::os_shim_internal::{Env, Fs};
use std::collections::HashSet;
use std::env;

// Default AWS environement names (used as last fallback)
const AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
const AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";
const AWS_ENDPOINT: &str = "AWS_ENDPOINT";

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

pub struct RegionProfile {
	pub region: Option<String>,
	pub profile: Option<String>,
}

pub async fn get_sbucket(reg_pro: RegionProfile, bucket: &str) -> Result<SBucket> {
	let client = new_s3_client(reg_pro, Some(bucket)).await?;
	let default_ignore_files = HashSet::from_iter(DEFAULT_UPLOAD_IGNORE_FILES.map(String::from));
	let config = SBucketConfig {
		default_ignore_upload_names: Some(default_ignore_files),
	};
	let sbucket = SBucket::from_client_and_name(client, bucket.to_string(), Some(config));

	Ok(sbucket)
}

pub async fn new_s3_client(reg_pro: RegionProfile, bucket: Option<&str>) -> Result<Client> {
	let cred = load_aws_cred(reg_pro, bucket).await?;
	let client = client_from_cred(cred)?;
	Ok(client)
}

fn client_from_cred(aws_cred: AwsCred) -> Result<Client> {
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

	let mut builder = Builder::new().credentials_provider(cred);

	if let Some(endpoint) = endpoint {
		builder = builder.endpoint_url(endpoint);
		// WORKAROUND - Right now, the aws-sdk-s3 (v0.24) throws a NoRegion on .send if not region even if we have a endpoint.
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
async fn load_aws_cred(reg_pro: RegionProfile, bucket: Option<&str>) -> Result<AwsCred> {
	let mut cred_result: Option<AwsCred> = None;

	// TODO: Need to determine if we need to check if we have a profile first before doing the bucket load.

	// -- Try to get it from the bucket env
	if let Some(bucket) = bucket {
		// first, try to get it from the SS3_BUCKET_bucket_name_KEY_ID, ... environments
		cred_result = load_aws_cred_from_ss3_bucket_env(bucket).await.ok();
	}

	// -- If not bucket env, then, go by profile if specified.
	if cred_result.is_none() {
		// if we have a profile defined
		if let Some(profile) = &reg_pro.profile {
			// try to get it from the SS3_PROFILE_profile_name_KEY_ID, ... environments
			cred_result = load_aws_cred_from_ss3_profile_env(profile).await.ok();

			// if not found in SS3_PROFILE... envs, try to get it from the aws config files
			if cred_result.is_none() {
				cred_result = load_aws_cred_from_aws_profile_configs(profile).await.ok();
			}
		}
	}

	// -- Last fall back standard aws envs
	if cred_result.is_none() {
		cred_result = load_aws_cred_from_default_aws_env().await.ok();
	}

	let mut cred = cred_result.ok_or_else(|| Error::NoCredentialsFoundForBucket(bucket.map(|s| s.to_string())))?;

	// -- If reg_pro as a region, override the one found (arg take precendence)
	if reg_pro.region.is_some() {
		cred.region = reg_pro.region
	}

	Ok(cred)
}

/// Attempt to create AwsCred from SS3 BUCKET environment variables
/// - `SS3_BUCKET_bucket_name_KEY_ID`
/// - `SS3_BUCKET_bucket_name_KEY_SECRET`
/// - `SS3_BUCKET_bucket_name_REGION`
/// - `SS3_BUCKET_bucket_name_ENDPOINT`
async fn load_aws_cred_from_ss3_bucket_env(bucket: &str) -> Result<AwsCred> {
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
/// - `SS3_PROFILE_profile_name_ENDPOINT`
async fn load_aws_cred_from_ss3_profile_env(profile: &str) -> Result<AwsCred> {
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

async fn load_aws_cred_from_aws_profile_configs(profile_str: &str) -> Result<AwsCred> {
	let (fs, ev) = (Fs::real(), Env::default());
	let profiles = aws_config::profile::load(&fs, &ev, &ProfileFiles::default(), None).await;
	if let Ok(profiles) = profiles {
		if let Some(profile) = profiles.get_profile(profile_str) {
			let key_id = get_profile_value(profile, "aws_access_key_id")?;
			let key_secret = get_profile_value(profile, "aws_secret_access_key")?;
			let region = get_profile_value(profile, "region").ok();
			let endpoint = get_profile_value(profile, "endpoint").ok();

			return Ok(AwsCred {
				key_id,
				key_secret,
				region,
				endpoint, // because aws configs only
			});
		}
	}

	Err(Error::NoCredentialsForProfile(profile_str.to_string()))
}

async fn load_aws_cred_from_default_aws_env() -> Result<AwsCred> {
	let key_id = get_env(AWS_ACCESS_KEY_ID)?;
	let key_secret = get_env(AWS_SECRET_ACCESS_KEY)?;
	let region = get_env(AWS_DEFAULT_REGION).ok();
	let endpoint = get_env(AWS_ENDPOINT).ok();

	Ok(AwsCred {
		key_id,
		key_secret,
		region,
		endpoint,
	})
}

// region:    Utils
fn get_env_name(typ: EnvType, key: CredKey, name: &str) -> String {
	let name = name.replace('-', "_");
	format!("{}_{}_{}", typ.env_part(), name, key.env_part())
}

fn get_profile_value(profile: &Profile, key: &str) -> Result<String> {
	match profile.get(key) {
		Some(value) => Ok(value.to_string()),
		None => Err(Error::NoCredentialConfig(key.to_string())),
	}
}

fn get_env(name: &str) -> Result<String> {
	match env::var(name) {
		Ok(v) => Ok(v),
		Err(_) => Err(Error::NoCredentialEnv(name.to_string())),
	}
}
// endregion: Utils
