use crate::Error;
use crate::Result;
use core::fmt;
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SPath {
	S3(S3Url),
	File(PathBuf),
}

impl SPath {
	pub fn from_str(path: &str) -> Result<SPath> {
		if path.starts_with("s3://") {
			Ok(SPath::S3(S3Url::from_url(path)?))
		} else {
			Ok(SPath::File(Path::new(path).to_path_buf()))
		}
	}
}

impl fmt::Display for SPath {
	// This trait requires `fmt` with this exact signature.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			SPath::S3(s3_url) => write!(f, "{}", s3_url),
			SPath::File(path) => write!(f, "{}", path.display()),
		}
	}
}

// region:    S3Url
#[derive(Debug)]
pub struct S3Url {
	bucket: String,
	key: String,
}
impl fmt::Display for S3Url {
	// This trait requires `fmt` with this exact signature.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "s3://{}/{}", self.bucket, self.key)
	}
}

impl S3Url {
	pub fn bucket(&self) -> &str {
		&self.bucket
	}
	pub fn key(&self) -> &str {
		&self.key
	}
}

/// Builders
impl S3Url {
	pub fn from_url(url: &str) -> Result<Self> {
		let rx = Regex::new(r"s3://([^:/\s]+)(.*)").expect("Invalid S3Url parsing regex");

		let caps = rx
			.captures(url)
			.map(|caps| caps.iter().filter_map(|cap| cap.map(|cap| cap.as_str())).collect::<Vec<_>>());

		if let Some(caps) = caps {
			if caps.len() == 3 {
				return Ok(S3Url {
					bucket: caps[1].to_string(),
					key: {
						let prefix = caps[2];
						prefix.strip_prefix('/').unwrap_or(prefix).to_string()
					},
				});
			}
		}

		Err(Error::NotValidS3Url(url.to_string()))
	}
}
// endregion: S3Url
