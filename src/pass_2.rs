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

pub struct SecondPassResult {
	pub start: u16,
	pub end: u16,
	pub bytes: [u8; 65536]
}

pub fn second_pass(first_pass: FirstPassResult) -> Result<SecondPassResult, ParseError> {
	let mut start = 2u16.pow(16);
	let mut end = 0u16;
	let mut bytes = [0u8; 65536];

	for line in first_pass.lines {
		bytes[line.addr as usize] = line.opcode;
		let last;

		if line.addr < start {
			start = line.addr;
		}

		match line.arg {
			InstructionArg::NoArgs => {
				last = line.addr
			}

			InstructionArg::ByteArg(n) => {
				last = line.addr + 1;
				bytes[last as usize] = n;
			}

			InstructionArg::WordArg(n) => {
				last = line.addr + 2;
				bytes[line.addr as usize + 1] = n as u8;
				bytes[last as usize] = (n >> 8) as u8;
			}

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

			InstructionArg::ByteLabelLowArg(label) => {
				last = line.addr + 1;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					bytes[last as usize] = *v as u8;
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

			InstructionArg::ByteLabelHighArg(label) => {
				last = line.addr + 1;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					bytes[last as usize] = (*v >> 8) as u8;
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

			InstructionArg::WordLabelArg(label) => {
				last = line.addr + 2;

				if let Some(v) = first_pass.symbol_table.get(&label) {
					bytes[line.addr as usize + 1] = *v as u8;
					bytes[last as usize] = (*v >> 8) as u8;
				} else {
					return ParseError::new(line.lino, &format!("Undeclared label '{}' used as value", label));
				}
			}

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

		if end <= last {
			end = last + 1;
		}
	}

	Ok(SecondPassResult {
		start,
		end,
		bytes
	})
}
