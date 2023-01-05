#![allow(unused)] // silence unused warnings while exploring (to comment out)
use cmd::cmd_run;

mod cmd;
mod consts;
mod error;
mod prelude;
mod s3w;
mod spath;

pub use consts::*;
pub use error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	match cmd_run().await {
		Ok(_) => (),
		Err(e) => {
			println!("Error:\n  {}", e)
		}
	};
}

pub use std::format as f;

#[macro_export]
macro_rules! s {
	() => {
		String::new()
	};
	($x:expr $(,)?) => {
		ToString::to_string(&$x)
	};
}
