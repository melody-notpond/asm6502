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
	write_addr: bool,
	addr_start: Option<u16>,
	addr_end: Option<u16>
}

fn main() {
	let mut config = Config {
		files: Vec::new(),
		out: String::from("a.out"),
		write_addr: true,
		addr_start: None,
		addr_end: None
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
			config.addr_start = Some(0);
			config.addr_end = Some(u16::MAX);
			config.write_addr = false;

		// Start address
		} else if arg == "-s" || arg == "--start" {
			if let Some(start) = iter.next() {
				// Parse hex number
				config.addr_start = match u16::from_str_radix(&start, 16) {
					Ok(a) => Some(a),
					Err(_) => {
						eprintln!("Error: -s must be followed by valid 16 bit hex number");
						process::exit(1);
					}
				};
			} else {
				eprintln!("Error: -s must be followed by a 16 bit hex number");
				process::exit(1);
			}

		// End address
		} else if arg == "-e" || arg == "--end" {
			if let Some(end) = iter.next() {
				// Parse hex number
				config.addr_end = match u16::from_str_radix(&end, 16) {
					Ok(a) => Some(a),
					Err(_) => {
						eprintln!("Error: -e must be followed by valid 16 bit hex number");
						process::exit(1);
					}
				}
			} else {
				eprintln!("Error: -s must be followed by a 16 bit hex number");
				process::exit(1);
			}

		// Input files
		} else if !config.files.contains(&arg) {
			config.files.push(arg);
		}
	}

	// Check for files
	if config.files.len() == 0 {
		eprintln!("usage: {} [-o out] [-d] [-s addr] [-e addr] [files]", name);
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
	let start = match config.addr_start {
		Some(v) => v,
		None => final_result.start
	};
	let end = match config.addr_end {
		Some(v) => v,
		None => final_result.end
	};

	if end > start {
		// Create contents of file
		let mut contents: Vec<u8> = Vec::with_capacity((end - start) as usize + 3);

		// Write address if enabled
		if config.write_addr {
			contents.push(start as u8);
			contents.push((start >> 8) as u8);
		}

		// Write contents
		contents.extend(&final_result.bytes[start as usize..end as usize + 1]);

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
