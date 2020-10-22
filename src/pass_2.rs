// 
// src
// pass_2.rs: Implements the second pass, which condenses code into a bytestring and converts labels.
// 
// Created by jenra.
// Created on October 22 2020.
// 

use crate::parser::ParseError;
use crate::pass_1::{
	FirstPassResult,
	InstructionArg
};

// Represents the result from the second pass
#[derive(Debug)]
pub struct SecondPassResult {
	pub start: u16,
	pub end: u16,
	pub bytes: [u8; 65536]
}

// Performs the second pass on the code
pub fn second_pass(first_pass: FirstPassResult) -> Result<SecondPassResult, ParseError> {
	let mut start = u16::MAX;
	let mut end = 0u16;
	let mut bytes = [0u8; 65536];

	// Iterate over the lines of code
	for line in first_pass.lines {
		// Set the opcode
		bytes[line.addr as usize] = line.opcode;

		// Set the start index if necessary
		if line.addr < start {
			start = line.addr;
		}

		// Match the argument
		let last;
		match line.arg {
			// No arguments
			InstructionArg::NoArgs => {
				last = line.addr
			}

			// 1 byte argument
			InstructionArg::ByteArg(n) => {
				last = line.addr + 1;
				bytes[last as usize] = n;
			}

			// 1 word argument
			InstructionArg::WordArg(n) => {
				last = line.addr + 2;
				bytes[line.addr as usize + 1] = n as u8;
				bytes[last as usize] = (n >> 8) as u8;
			}

			// Decode byte label argument
			InstructionArg::ByteLabelArg(label) => {
				last = line.addr + 1;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					if *v < 256 {
						bytes[last as usize] = *v as u8;
					} else {
						return ParseError::new(line.lino, &format!("Expected byte, found word {}", label));
					}
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

			// Decode low byte of label argument
			InstructionArg::ByteLabelLowArg(label) => {
				last = line.addr + 1;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					bytes[last as usize] = *v as u8;
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

			// Decode high byte of label argument
			InstructionArg::ByteLabelHighArg(label) => {
				last = line.addr + 1;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					bytes[last as usize] = (*v >> 8) as u8;
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

			// Decode word label argument
			InstructionArg::WordLabelArg(label) => {
				last = line.addr + 2;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					bytes[line.addr as usize + 1] = *v as u8;
					bytes[last as usize] = (*v >> 8) as u8;
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

			// Decode relative label argument
			InstructionArg::RelativeLabelArg(label) => {
				last = line.addr + 1;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					let diff = *v as i32 - line.addr as i32 + 2;
					if -128 <= diff && diff <= 127 {
						bytes[last as usize] = diff as u8;
					} else {
						return ParseError::new(line.lino, &format!("Label '{}' is too far away", label));
					}
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}
		}

		// Set the end index
		if end <= last {
			end = last + 1;
		}
	}

	// Success!
	Ok(SecondPassResult {
		start,
		end,
		bytes
	})
}
