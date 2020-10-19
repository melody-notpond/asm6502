use std::env;
use std::fs;
use std::process;

use asm6502::lexer;

fn main()
{
	let args: Vec<String> = env::args().collect();

	if args.len() <= 1
	{
		println!("usage: {} [files]", args[0]);
		process::exit(1);
	}

	for file in &args[1..]
	{
		let content = fs::read_to_string(file).unwrap_or_else(|_| {
			panic!("failed");
		});

		println!("Content:\n{}", content);
	}

	println!("{:?}", args);
}
