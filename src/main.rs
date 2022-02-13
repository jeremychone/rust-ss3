// #![allow(unused)] // silence unused warnings while exploring (to comment out)
use cmd::cmd_run;

mod cmd;
mod error;
mod s3w;
mod spath;

pub use error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	match cmd_run().await {
		Ok(_) => println!("✔ All good and well"),
		Err(e) => {
			println!("Error:\n  {}", e)
		}
	};
}
