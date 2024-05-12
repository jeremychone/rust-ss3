use crate::Result;
use std::io::{self, Write};

pub fn prompt(msg: &str) -> Result<String> {
	print!("{}", msg);
	// flush
	io::stdout().flush().expect("Cannot flush stdout ???");

	let mut buff: String = String::new();
	io::stdin().read_line(&mut buff).expect("Cannot read stdin ???");
	let buff = buff.trim();

	Ok(buff.to_string())
}
