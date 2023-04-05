// -- Re-Export
pub use self::error::{Error, Result};
pub use consts::*;
pub use std::format as f;

// -- Imports
use cmd::cmd_run;
use std::process::ExitCode;

// -- Sub-Modules
mod cmd;
mod consts;
mod error;
mod prelude;
mod s3w;
mod spath;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
	match cmd_run().await {
		Ok(_) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("{e}");
			ExitCode::from(1)
		}
	}
}

#[macro_export]
macro_rules! s {
	() => {
		String::new()
	};
	($x:expr $(,)?) => {
		ToString::to_string(&$x)
	};
}
