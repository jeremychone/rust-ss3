use aws_sdk_s3::error::{GetObjectError, HeadObjectError, ListObjectsV2Error, PutObjectError};
use aws_sdk_s3::types::SdkError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Not a valid s3 url '{0}'. Should be format 's3://bucket_name[/path/to/object]'")]
	NotValidS3Url(String),

	#[error("Credential environment variable {0} not found")]
	NoCredentialEnv(String),

	#[error("Credential profile config key {0} not found")]
	NoCredentialConfig(String),

	#[error("No credentials found for profile {0}.")]
	NoCredentialsForProfile(String),

	#[error(
		"No AWS environment variable found. Specify default 'AWS_ACCESS_KEY_ID', ... environments, or specify a valid --profile profile_name."
	)]
	NoDefaultEnvCredentialsFound,

	#[error(
		"No credential found for bucket '{0}'. Provide the following (by order of precedence): 
  - Provide bucket SS3_BUCKET_... environments (will take precendence on profile env/configs)
    - SS3_BUCKET_bucket_name_KEY_ID
    - SS3_BUCKET_bucket_name_KEY_SECRET
    - SS3_BUCKET_bucket_name_REGION  
  - Provide '--profile profile_name' with the following SS3_PROFILE_... environments:
    - SS3_PROFILE_profile_name_KEY_ID
    - SS3_PROFILE_profile_name_KEY_SECRET
    - SS3_PROFILE_profile_name_REGION  
  - Provide '--profile profile_name' which should be configured in aws default config files
  - As a last fallback, use the default AWS environment variables: 
    - AWS_ACCESS_KEY_ID
    - AWS_SECRET_ACCESS_KEY
    - AWS_DEFAULT_REGION
  NOTE: '-' characters in profile and bucket names will be replaced by '_' for environment names above.		
  	"
	)]
	NoCredentialsFoundForBucket(String),

	#[error("Missing config. The credential environment variables or config must have either a REGION or ENDPOINT. Both absent.")]
	MissingConfigMustHaveEndpointOrRegion,

	#[error("Invalid command. Cause: {0}")]
	CmdInvalid(&'static str),

	#[error("File path '{0}' not found.")]
	FilePathNotFound(String),

	#[error("Not Supported - '{0}' feature is not supported.")]
	NotSupported(&'static str),

	#[error("Not Supported yet - '{0}' feature is not supported yet")]
	NotSupportedYet(&'static str),

	#[error("Cannot perform, invalid key '{0}'")]
	InvalidPath(String),

	#[error("Fail mode is on and the object '{0}' already exits")]
	ObjectExistsOverFailMode(String),

	#[error("Fail mode is on and the file '{0}' already exits")]
	FileExistsOverFailMode(String),

	#[error("This command is not valid. Cause: {0}")]
	ComamndInvalid(&'static str),

	#[error(transparent)]
	InvalidUri(#[from] http::uri::InvalidUri),

	#[error(transparent)]
	ByteStreamError(#[from] aws_smithy_http::byte_stream::Error),

	#[error(transparent)]
	AwsGetObjectError(#[from] SdkError<GetObjectError>),

	#[error(transparent)]
	AwsListObjectsV2Error(#[from] SdkError<ListObjectsV2Error>),

	#[error(transparent)]
	AwsPutObjectError(#[from] SdkError<PutObjectError>),

	#[error(transparent)]
	AwsHeadObjectError(#[from] SdkError<HeadObjectError>),

	#[error(transparent)]
	IOError(#[from] std::io::Error),
}
