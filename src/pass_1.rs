//
// src
// pass_1.rs: Implements the first pass, which generates code and creates a symbol table for labels.
//
// Created by jenra.
// Created on October 21 2020.
//

use std::collections::HashMap;

use crate::lexer::Lexer;
use crate::parser;
use crate::parser::{
	Address, LineValue, ParseError, Pragma
};

// The value of an argument of an instruction
pub enum InstructionArg {
	NoArgs,
	ByteArg(u8),
	ByteLabelArg(String),
	WordArg(u16),
	WordLabelArg(String)
}

// An annotated line of assembly
pub struct AnnotatedLine {
	pub addr: u16,
	pub opcode: u8,
	pub arg: InstructionArg
}

// The result of the first pass
pub struct FirstPassResult {
	pub lines: Vec<AnnotatedLine>,
	pub symbol_table: HashMap<String, u16>
}

// Adds a symbol to the symbol table
fn add_symbol(symbol_table: &mut HashMap<String, u16>, key: String, value: u16) {
	if symbol_table.contains_key(&key) {
		panic!("Repeated symbol");
	} else {
		symbol_table.insert(key, value);
	}
}

// Performs the first pass on the code
pub fn first_pass(lexer: &mut Lexer) -> Result<FirstPassResult, ParseError> {
	let mut symbol_table = HashMap::new();
	let mut lines = Vec::new();
	let mut addr = 0u16;

	// Iterate over every line
	while let Some(line) = parser::parse_line(lexer)? {
		// Set labels to the current address
		if line.label != "" {
			add_symbol(&mut symbol_table, line.label, addr);
		} else {
			match line.value {
				// Deal with instructions
				LineValue::Instruction(instr) => {
	
				}

				// Deal with pragmas
				LineValue::Pragma(pragma) => {
					match pragma {
						// Push one byte
						Pragma::Byte(byte) => {
							lines.push(AnnotatedLine {
								addr: addr,
								opcode: byte,
								arg: InstructionArg::NoArgs
							});
							addr += 1;
						}
	
						// Push a collection of bytes
						Pragma::Bytes(bytes) => {
							for byte in bytes {
								lines.push(AnnotatedLine {
									addr: addr,
									opcode: byte,
									arg: InstructionArg::NoArgs
								});
								addr += 1;
							}
						}
	
						// Push a word
						Pragma::Word(word) => {
							let word = match word {
								Address::Label(label) => {
									// Labels must be already set to set the origin
									match symbol_table.get(&label) {
										Some(w) => *w,
										None => return ParseError::new(lexer, &format!("Setting origin to value of undefined label {}", label))
									}
								}
	
								// Literal address
								Address::Literal(w) => w
							};
	
							// Push low byte
							lines.push(AnnotatedLine {
								addr: addr,
								opcode: word as u8,
								arg: InstructionArg::NoArgs
							});
							addr += 1;
	
							// Push high byte
							lines.push(AnnotatedLine {
								addr: addr,
								opcode: (word >> 8) as u8,
								arg: InstructionArg::NoArgs
							});
							addr += 1;
						}
	
						// Set the origin
						Pragma::Origin(a) => {
							match a {
								Address::Label(label) => {
									// Labels must be already set to set the origin
									addr = match symbol_table.get(&label) {
										Some(a) => *a,
										None => return ParseError::new(lexer, &format!("Setting origin to value of undefined label {}", label))
									}
								}
	
								// Literal address
								Address::Literal(a) => addr = a
							}
						}

						// Define a label with a given address
						Pragma::Define(label, addr) => {
							match addr {
								// Set label to the value of another label
								Address::Label(s) => {
									let v = match symbol_table.get(&s) {
										Some(v) => *v,
										None => return ParseError::new(lexer, &format!("Setting label {} to value of undefined label {}", label, s))
									};
									add_symbol(&mut symbol_table, label, v);
								}
				
								// Set label to a value
								Address::Literal(n) => add_symbol(&mut symbol_table, label, n)
							}
						}

						// Include a file (TODO)
						Pragma::Include(_) => todo!()
					}
				}

				LineValue::None => unreachable!("None case already handled")
			}
		}

	}

	Ok(FirstPassResult {
		lines, symbol_table
	})
}
