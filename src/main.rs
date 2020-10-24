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
	full_disc: bool
}

fn main() {
	let mut config = Config {
		files: Vec::new(),
		out: String::from("a.out"),
		full_disc: false
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
				eprintln!("Error: -o must be followed by a file name");
				process::exit(1);
			}

		// Full disc
		} else if arg == "-d" || arg == "--disc" {
			config.full_disc = true;

		// Input files
		} else if !config.files.contains(&arg) {
			config.files.push(arg);
		}
	}

	// Check for files
	if config.files.len() == 0 {
		eprintln!("usage: {} [-o out] [-d] [files]", name);
		process::exit(1);
	}

	// The final result to be turned into a binary file
	let mut final_result = AssemblerResult {
		filename: String::from("total"),
		start: u16::MAX,
		end: 0,
		bytes: [0u8; u16::MAX as usize + 1]
	};

	// Iterate over every file
	for file in config.files {
		// Read file
		let content = fs::read_to_string(&file).unwrap_or_else(|_| {
			panic!("Could not read file");
		});

		// Parse file and generate code
		let mut lexer = Lexer::new(&file, &content);
		let result = pass_1::first_pass(&mut lexer).expect("Error handling parsing or pass 1");
		let result = pass_2::second_pass(result).expect("Error handling pass 2");
		final_result.merge(&result).expect("Error merging files");
	}

	if config.full_disc {
		fs::write(config.out, final_result.bytes).expect("Error writing file");
	} else {
		let mut contents: Vec<u8> = Vec::with_capacity((final_result.end - final_result.start + 2) as usize);
		contents.push(final_result.start as u8);
		contents.push((final_result.start >> 8) as u8);
		contents.extend(&final_result.bytes[final_result.start as usize..final_result.end as usize]);
		fs::write(config.out, contents).expect("Error writing file");
	}
}
