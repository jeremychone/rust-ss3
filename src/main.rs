// #![allow(unused)] // silence unused warnings while exploring (to comment out)
use cmd::cmd_run;

mod cmd;
mod consts;
mod error;
mod prelude;
mod s3w;
mod spath;

pub use consts::*;
pub use error::Error;
pub use std::format as f;
use std::process::ExitCode;

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
