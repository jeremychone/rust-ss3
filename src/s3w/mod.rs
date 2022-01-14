////////////////////////////////////
// AWS S3 Wrapper API
////
use self::s3_bucket::SBucket;
use crate::Error;
use aws_config::profile::Profile;
use aws_sdk_s3::{Client, Credentials, Region};
use aws_types::credentials::SharedCredentialsProvider;
use aws_types::os_shim_internal::{Env, Fs};
use std::env;

mod cred;
mod s3_bucket;

// re-export
pub use self::cred::get_sbucket;
pub use self::s3_bucket::ListOptions;
