use aws_sdk_s3::error::{GetObjectError, HeadObjectError, ListBucketsError, ListObjectsV2Error, PutObjectError};
use aws_sdk_s3::types::SdkError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("Static error: {0}")]
	Static(&'static str),

	#[error("Generic error: {0}")]
	Generic(String),

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
		"No credential found for bucket '{0:?}'. Provide the following (by order of precedence): 
  - Provide bucket SS3_BUCKET_... environments (will take precendence on profile env/configs)
    - SS3_BUCKET_bucket_name_KEY_ID
    - SS3_BUCKET_bucket_name_KEY_SECRET
    - SS3_BUCKET_bucket_name_REGION
    - SS3_BUCKET_bucket_name_ENDPOINT (optional)  
  - Provide '--profile profile_name' with the following SS3_PROFILE_... environments:
    - SS3_PROFILE_profile_name_KEY_ID
    - SS3_PROFILE_profile_name_KEY_SECRET
    - SS3_PROFILE_profile_name_REGION  
    - SS3_PROFILE_profile_name_ENDPOINT (optional)
  - Provide '--profile profile_name' which should be configured in aws default config files
  - As a last fallback, use the default AWS environment variables: 
    - AWS_ACCESS_KEY_ID
    - AWS_SECRET_ACCESS_KEY
    - AWS_DEFAULT_REGION
    - AWS_ENDPOINT (optional)
  NOTE: '-' characters in profile and bucket names will be replaced by '_' for environment names above.		
  	"
	)]
	NoCredentialsFoundForBucket(Option<String>),

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
	ByteStream(#[from] aws_smithy_http::byte_stream::error::Error),

	#[error(transparent)]
	InvalidEndpoint(#[from] aws_config::endpoint::error::InvalidEndpointError),

	#[error("AWS Service Error. Code: {0}, Message: {1}")]
	AwsServiceError(String, String), // code, message

	#[error(transparent)]
	AwsGetObject(#[from] SdkError<GetObjectError>),

	#[error(transparent)]
	AwsListObjectsV2(#[from] SdkError<ListObjectsV2Error>),

	#[error(transparent)]
	AwsPutObject(#[from] SdkError<PutObjectError>),

	#[error(transparent)]
	AwsHeadObject(#[from] SdkError<HeadObjectError>),

	#[error(transparent)]
	IO(#[from] std::io::Error),
}

/// For better CLI error reporting.
/// Note: Might do the same for the other AwsError types.
impl From<SdkError<ListBucketsError>> for Error {
	fn from(val: SdkError<ListBucketsError>) -> Self {
		let se = val.into_service_error();
		let code = se.code().unwrap_or_default().to_string();
		let message = se.message().unwrap_or_default().to_string();
		Error::AwsServiceError(code, message)
	}
}
