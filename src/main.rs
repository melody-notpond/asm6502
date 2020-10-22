use std::env;
use std::fs;
use std::process;

use asm6502::lexer::Lexer;
use asm6502::pass_1;
use asm6502::pass_2;
use asm6502::pass_2::AssemblerResult;

struct Config {
	files: Vec<String>,
	out: String,
}

fn main() {
	let mut config = Config {
		files: Vec::new(),
		out: String::from("a.out")
	};

	// Set up config
	let mut iter = env::args();
	let name = iter.next().unwrap();
	while let Some(arg) = iter.next() {
		// Output files
		if arg == "-o" || arg == "--output" {
			if let Some(out) = iter.next() {
				config.out = out;
			} else {
				eprintln!("Error: -o must be followed by a file");
				process::exit(1);
			}

		// Input files
		} else if !config.files.contains(&arg) {
			config.files.push(arg);
		}
	}

	// Check for files
	if config.files.len() == 0 {
		eprintln!("usage: {} [files]", name);
		process::exit(1);
	}

	// The final result to be turned into a binary file
	let mut final_result = AssemblerResult {
		filename: String::from("total"),
		start: u16::MAX,
		end: 0,
		bytes: [0u8; u16::MAX as usize + 1]
	};

	for file in config.files {
		let content = fs::read_to_string(&file).unwrap_or_else(|_| {
			panic!("Could not read file");
		});

		println!("Content:\n{}", content);
		let mut lexer = Lexer::new(&file, &content);
		let result = pass_1::first_pass(&mut lexer).expect("Error handling parsing or pass 1");
		let result = pass_2::second_pass(result).expect("Error handling pass 2");
		final_result.merge(&result).expect("Error merging files");
	}

	for byte in &final_result.bytes[final_result.start as usize..final_result.end as usize] {
		print!("{:02X} ", *byte);
	}
	println!();
}
