use crate::utils;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::CreateBucketError;
use aws_sdk_s3::operation::delete_bucket::DeleteBucketError;
use aws_sdk_s3::operation::delete_object::DeleteObjectError;
use aws_sdk_s3::operation::get_object::GetObjectError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use aws_sdk_s3::operation::list_buckets::ListBucketsError;
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Error;
use aws_sdk_s3::operation::put_object::PutObjectError;
use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum Error {
	#[from(String, &String, &str)]
	Custom(String),

	// -- Generic
	Static(&'static str),

	// -- Get
	#[display("Cannot find S3 object at key '{key}'")]
	S3ObjectNotFound { key: String },

	// -- Clean
	#[display("Invalid clean url. Must be valid `local file path` and then `s3 url/base path` (was '{url_1}' and then '{url_2}`) ")]
	CleanInvalidArguments { url_1: String, url_2: String },

	// -- Uncategorized
	#[display("Not a valid s3 url '{_0}'. Should be format 's3://bucket_name[/path/to/object]'")]
	NotValidS3Url(String),

	#[display("Credential environment variable {_0} not found")]
	NoCredentialEnv(String),

	#[display("Credential profile config key {_0} not found")]
	NoCredentialConfig(String),

	#[display("No credentials found for profile {_0}.")]
	NoCredentialsForProfile(String),

	#[display(
		"No AWS environment variable found. Specify default 'AWS_ACCESS_KEY_ID', ... environments, or specify a valid --profile profile_name."
	)]
	NoDefaultEnvCredentialsFound,

	#[display(
		"No credential found for bucket '{_0:?}'. Provide the following (by order of precedence): 
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

	#[display("Missing config. The credential environment variables or config must have either a REGION or ENDPOINT. Both absent.")]
	MissingConfigMustHaveEndpointOrRegion,

	#[display("Invalid command. Cause: {_0}")]
	CmdInvalid(&'static str),

	#[display("File path '{_0}' not found.")]
	FilePathNotFound(String),

	#[display("Not Supported - '{_0}' feature is not supported.")]
	NotSupported(&'static str),

	#[display("Not Supported yet - '{_0}' feature is not supported yet")]
	NotSupportedYet(&'static str),

	#[display("Cannot perform, invalid key '{_0}'")]
	InvalidPath(String),

	#[display("Fail mode is on and the object '{_0}' already exits")]
	ObjectExistsOverFailMode(String),

	#[display("Fail mode is on and the file '{_0}' already exits")]
	FileExistsOverFailMode(String),

	#[display("This command is not valid. Cause: {_0}")]
	ComamndInvalid(&'static str),

	// -- Utils
	#[from]
	Md5(utils::md5::Error),

	// -- Externals
	#[from]
	InvalidUri(http::uri::InvalidUri),

	#[from]
	SimpleFs(simple_fs::Error),

	// aws_sdk_s3::primitives::ByteStreamError
	#[from]
	ByteStream(aws_sdk_s3::primitives::ByteStreamError),

	#[display("AWS SDK ERROR:\n       Code: {code}\n    Message: {message}")]
	AwsSdkErrorWrapper { code: String, message: String },

	#[from]
	IO(std::io::Error),
}

// region:    --- Custom

impl Error {
	pub fn custom_from_err(err: impl std::error::Error) -> Self {
		Self::Custom(err.to_string())
	}

	pub fn custom(val: impl Into<String>) -> Self {
		Self::Custom(val.into())
	}
}

// endregion: --- Custom

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate

macro_rules! impl_from_sdk_error {
	($($ie:ident),*) => {
		$(
impl From<SdkError<$ie>> for Error {
	fn from(val: SdkError<$ie>) -> Self {
		let se = val.into_service_error();
		let em = se.meta();
		let code = em.code().unwrap_or("NO_CODE").to_string();
		let message = em.message().unwrap_or_default().to_string();
		Error::AwsSdkErrorWrapper { code, message }
	}
}
		)*
	};
}

impl_from_sdk_error!(
	ListBucketsError,
	CreateBucketError,
	DeleteBucketError,
	GetObjectError,
	DeleteObjectError,
	PutObjectError,
	HeadObjectError,
	ListObjectsV2Error
);

// For better CLI error reporting.
// Note: Might do the same for the other AwsError types.
// impl From<SdkError<ListBucketsError>> for Error {
// 	fn from(val: SdkError<ListBucketsError>) -> Self {
// 		let se = val.into_service_error();
// 		let code = se.code().unwrap_or_default().to_string();
// 		let message = se.message().unwrap_or_default().to_string();
// 		Error::AwsServiceError(code, message)
// 	}
// }

// impl From<SdkError<CreateBucketError>> for Error {
// 	fn from(val: SdkError<CreateBucketError>) -> Self {
// 		let se = val.into_service_error();
// 		let code = se.code().unwrap_or_default().to_string();
// 		let message = se.message().unwrap_or_default().to_string();
// 		Error::AwsCreateBucket(code, message)
// 	}
// }
