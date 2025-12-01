use md5::Context;
use std::io::Read as _;
use std::path::Path;
use std::{fs, io};

pub fn compute_md5(file_path: impl AsRef<Path>) -> Result<String, Error> {
	let file_path = file_path.as_ref();
	let file = fs::File::open(file_path).map_err(|err| Error::fail_md5(file_path, err))?;
	let mut context = Context::new();
	let mut buffer = [0; 2048]; // buffer size: 4KB

	let mut reader = io::BufReader::new(file);
	loop {
		let bytes_read = reader.read(&mut buffer).map_err(|err| Error::fail_md5(file_path, err))?;
		if bytes_read == 0 {
			break;
		}
		context.consume(&buffer[..bytes_read]);
	}

	let result = context.finalize();
	Ok(format!("{:x}", result))
}

// region:    --- Error

#[derive(Debug)]
pub enum Error {
	// -- Externals
	FailMd5 { path: String, cause: io::Error }, // as example
}

impl Error {
	pub fn fail_md5(path: impl AsRef<Path>, cause: std::io::Error) -> Error {
		let path = path.as_ref();
		Error::FailMd5 {
			path: path.display().to_string(),
			cause,
		}
	}
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate

// endregion: --- Error
