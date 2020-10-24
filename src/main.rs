use std::env;
use std::fs;
use std::io::ErrorKind;
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
		let content = fs::read_to_string(&file).unwrap_or_else(|e| {
			match e.kind() {
				ErrorKind::NotFound => eprintln!("File {} not found", &file),
				ErrorKind::PermissionDenied => eprintln!("Insufficient permissions to read {}", &file),
				_ => eprintln!("Error occured whilst reading from {}: {}", &file, e)
			}
			process::exit(1);
		});

		// Parse file and generate ir
		let mut lexer = Lexer::new(&file, &content);
		let result = pass_1::first_pass(&mut lexer)
			.unwrap_or_else(|e| {
				eprintln!("Error on {}:{}: {}", e.filename, e.lino, e.message);
				process::exit(1);
			}
		);

		// Generate code
		let result = pass_2::second_pass(result)
			.unwrap_or_else(|e| {
				eprintln!("Error on {}:{}: {}", e.filename, e.lino, e.message);
				process::exit(1);
			}
		);

		// Merge code
		final_result.merge(&result).unwrap_or_else(|e| {
				eprintln!("Error: {}", e);
				process::exit(1);
			}
		);
	}

	let out = config.out;
	if config.full_disc {
		// Write disc to file
		fs::write(&out, final_result.bytes).unwrap_or_else(|e| {
			match e.kind() {
				ErrorKind::PermissionDenied => eprintln!("Insufficient permissions to write {}", &out),
				_ => eprintln!("Error occured whilst writing to {}: {}", &out, e)
			}
			process::exit(1);
		});
	} else if final_result.end > final_result.start {
		// Create contents of file
		let mut contents: Vec<u8> = Vec::with_capacity((final_result.end - final_result.start) as usize + 2);

		contents.push(final_result.start as u8);
		contents.push((final_result.start >> 8) as u8);
		contents.extend(&final_result.bytes[final_result.start as usize..final_result.end as usize]);

		// Write code to file
		fs::write(&out, contents).unwrap_or_else(|e| {
			match e.kind() {
				ErrorKind::PermissionDenied => eprintln!("Insufficient permissions to write {}", &out),
				_ => eprintln!("Error occured whilst writing to {}: {}", &out, e)
			}
			process::exit(1);
		});
	} else {
		// Write empty file
		fs::write(&out, [0u8, 0]).unwrap_or_else(|e| {
			match e.kind() {
				ErrorKind::PermissionDenied => eprintln!("Insufficient permissions to write {}", &out),
				_ => eprintln!("Error occured whilst writing to {}: {}", &out, e)
			}
			process::exit(1);
		});
	}
}
