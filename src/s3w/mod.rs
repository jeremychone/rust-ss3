////////////////////////////////////
// AWS S3 Wrapper API
////

mod cred;
mod s3_bucket;

// re-export
pub use self::cred::get_sbucket;
pub use self::s3_bucket::ListOptions;
