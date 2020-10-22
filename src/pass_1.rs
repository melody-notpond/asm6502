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
	Address,
	AddressingMode,
	ImmediateValue,
	LineValue,
	ParseError,
	Pragma
};

// The value of an argument of an instruction
#[derive(Debug)]
pub enum InstructionArg {
	NoArgs,
	ByteArg(u8),
	ByteLabelArg(String),
	RelativeLabelArg(String),
	ByteLabelLowArg(String),
	ByteLabelHighArg(String),
	WordArg(u16),
	WordLabelArg(String)
}

// An annotated line of assembly
#[derive(Debug)]
pub struct AnnotatedLine {
	pub addr: u16,
	pub opcode: u8,
	pub arg: InstructionArg
}

// The result of the first pass
#[derive(Debug)]
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

// Opcodes that end with c=01
macro_rules! opcode_c_01 {
	($opcode: literal, $line: ident, $addr: ident, $instr: ident, $lexer: ident) => {{
		// Set opcode
		$line.opcode = $opcode;
		$addr += 1;

		// Match the addressing mode
		match $instr.addr_mode {
			// lda ($addr, x)
			AddressingMode::IndirectX(a) => {
				$line.opcode |= 0b000_000_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// lda $zp
			AddressingMode::ZeroPage(a) => {
				$line.opcode |= 0b000_001_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// lda #imm
			AddressingMode::Immediate(i) => {
				$line.opcode |= 0b000_010_00;

				$line.arg = match i {
					ImmediateValue::Literal(n) => InstructionArg::ByteArg(n),
					ImmediateValue::Label(label) => InstructionArg::ByteLabelArg(label),
					ImmediateValue::LowByte(label) => InstructionArg::ByteLabelLowArg(label),
					ImmediateValue::HighByte(label) => InstructionArg::ByteLabelHighArg(label),
				};

				$addr += 1;
			}

			// lda $addr
			AddressingMode::Absolute(a) => {
				$line.opcode |= 0b000_011_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// lda ($addr), y
			AddressingMode::IndirectY(a) => {
				$line.opcode |= 0b000_100_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// lda $zp, x
			AddressingMode::ZeroPageX(a) => {
				$line.opcode |= 0b000_101_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// lda $addr, y
			AddressingMode::ZeroPageY(a) => {
				$line.opcode |= 0b000_110_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			AddressingMode::AbsoluteY(a) => {
				$line.opcode |= 0b000_110_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// lda $addr, x
			AddressingMode::AbsoluteX(a) => {
				$line.opcode |= 0b000_111_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// Invalid argument
			_ => return ParseError::new(&$lexer, &format!("Invalid argument for opcode '{}'", $instr.opcode))
		}
	}};
}

// Opcodes that end with c=01
macro_rules! opcode_c_10 {
	($opcode: literal, $line: ident, $addr: ident, $instr: ident, $lexer: ident, $zpx: ident, $absx: ident, $imm: literal, $a: literal, $absxincl: literal) => {{
		// Set opcode
		$line.opcode = $opcode;
		$addr += 1;

		// Match the addressing mode
		match $instr.addr_mode {
			// ldx #imm
			AddressingMode::Immediate(i) if $imm => {
				$line.opcode |= 0b000_000_00;

				$line.arg = match i {
					ImmediateValue::Literal(n) => InstructionArg::ByteArg(n),
					ImmediateValue::Label(label) => InstructionArg::ByteLabelArg(label),
					ImmediateValue::LowByte(label) => InstructionArg::ByteLabelLowArg(label),
					ImmediateValue::HighByte(label) => InstructionArg::ByteLabelHighArg(label),
				};

				$addr += 1;
			}

			// asl $zp
			AddressingMode::ZeroPage(a) => {
				$line.opcode |= 0b000_001_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// asl
			AddressingMode::Implicit if $a => {
				$line.opcode |= 0b000_010_00;
			}

			// asl $addr
			AddressingMode::Absolute(a) => {
				$line.opcode |= 0b000_011_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// asl $zp, x
			AddressingMode::$zpx(a) => {
				$line.opcode |= 0b000_101_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// asl $addr, x
			AddressingMode::$absx(a) if $absxincl => {
				$line.opcode |= 0b000_111_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// Invalid argument
			_ => return ParseError::new(&$lexer, &format!("Invalid argument for opcode '{}'", $instr.opcode))
		}
	}};
}

// Opcodes that end with c=00
macro_rules! opcode_c_00 {
	($opcode: literal, $line: ident, $addr: ident, $instr: ident, $lexer: ident, $imm: literal, $zpx: literal, $absx: literal) => {{
		// Set opcode
		$line.opcode = $opcode;
		$addr += 1;

		// Match the addressing mode
		match $instr.addr_mode {
			// ldy #imm
			AddressingMode::Immediate(i) if $imm => {
				$line.opcode |= 0b000_000_00;

				$line.arg = match i {
					ImmediateValue::Literal(n) => InstructionArg::ByteArg(n),
					ImmediateValue::Label(label) => InstructionArg::ByteLabelArg(label),
					ImmediateValue::LowByte(label) => InstructionArg::ByteLabelLowArg(label),
					ImmediateValue::HighByte(label) => InstructionArg::ByteLabelHighArg(label),
				};

				$addr += 1;
			}

			// ldy $zp
			AddressingMode::ZeroPage(a) => {
				$line.opcode |= 0b000_001_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// ldy $addr
			AddressingMode::Absolute(a) => {
				$line.opcode |= 0b000_011_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// ldy $zp, x
			AddressingMode::ZeroPageX(a) if $zpx => {
				$line.opcode |= 0b000_101_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::ByteLabelArg(label)
				};

				$addr += 1;
			}

			// ldy $addr, x
			AddressingMode::AbsoluteX(a) if $absx => {
				$line.opcode |= 0b000_111_00;

				$line.arg = match a {
					Address::Literal(n) => InstructionArg::WordArg(n),
					Address::Label(label) => InstructionArg::WordLabelArg(label)
				};

				$addr += 2;
			}

			// Invalid argument
			_ => return ParseError::new(&$lexer, &format!("Invalid argument for opcode '{}'", $instr.opcode))
		}
	}};
}

// Branching opcodes
macro_rules! opcode_branch {
	($opcode: literal, $line: ident, $addr: ident, $instr: ident, $lexer: ident) => {{
		// Set opcode
		$line.opcode = $opcode;
		$addr += 2;

		// Match the addressing mode
		match $instr.addr_mode {
			AddressingMode::ZeroPage(a) => {
				$line.arg = match a {
					Address::Literal(n) => InstructionArg::ByteArg(parser::check_overflow(&$lexer, n)?),
					Address::Label(label) => InstructionArg::RelativeLabelArg(label)
				};
			}

			AddressingMode::Absolute(a) => {
				$line.arg = match a {
					Address::Literal(_) => return ParseError::new(&$lexer, "Branching out of bounds"),
					Address::Label(label) => InstructionArg::RelativeLabelArg(label)
				};
			}

			// Invalid argument
			_ => return ParseError::new(&$lexer, &format!("Invalid argument for opcode '{}'", $instr.opcode))
		}
	}};
}

// Opcodes that are only a single byte
macro_rules! opcode_implicit {
	($opcode: literal, $line: ident, $addr: ident, $instr: ident, $lexer: ident) => {{
		// Set opcode
		$line.opcode = $opcode;
		$addr += 1;

		// Match the addressing mode
		match $instr.addr_mode {
			AddressingMode::Implicit => {}

			// Invalid argument
			_ => return ParseError::new(&$lexer, &format!("Invalid argument for opcode '{}'", $instr.opcode))
		}
	}};
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
					let mut line = AnnotatedLine {
						addr: addr,
						opcode: 0b000_000_00,
						arg: InstructionArg::NoArgs
					};

					// Match the opcode (aaa_bbb_cc)
					match instr.opcode.to_lowercase().as_str() {
						// c=01
						"ora" => opcode_c_01!(0b000_000_01, line, addr, instr, lexer),
						"and" => opcode_c_01!(0b000_001_01, line, addr, instr, lexer),
						"eor" => opcode_c_01!(0b000_010_01, line, addr, instr, lexer),
						"adc" => opcode_c_01!(0b000_011_01, line, addr, instr, lexer),
						"sta" => opcode_c_01!(0b000_100_01, line, addr, instr, lexer),
						"lda" => opcode_c_01!(0b000_101_01, line, addr, instr, lexer),
						"cmp" => opcode_c_01!(0b000_110_01, line, addr, instr, lexer),
						"sbc" => opcode_c_01!(0b000_111_01, line, addr, instr, lexer),

						// c=10
						"asl" => opcode_c_10!(0b000_000_10, line, addr, instr, lexer, ZeroPageX, AbsoluteX, false, true , true ),
						"rol" => opcode_c_10!(0b000_001_10, line, addr, instr, lexer, ZeroPageX, AbsoluteX, false, true , true ),
						"lsr" => opcode_c_10!(0b000_010_10, line, addr, instr, lexer, ZeroPageX, AbsoluteX, false, true , true ),
						"ror" => opcode_c_10!(0b000_011_10, line, addr, instr, lexer, ZeroPageX, AbsoluteX, false, true , true ),
						"stx" => opcode_c_10!(0b000_100_10, line, addr, instr, lexer, ZeroPageY, AbsoluteY, false, false, false),
						"ldx" => opcode_c_10!(0b000_101_10, line, addr, instr, lexer, ZeroPageY, AbsoluteY, true , false, true ),
						"dec" => opcode_c_10!(0b000_110_10, line, addr, instr, lexer, ZeroPageX, AbsoluteX, false, false, true ),
						"inc" => opcode_c_10!(0b000_111_10, line, addr, instr, lexer, ZeroPageX, AbsoluteX, false, false, true ),

						// c=00
						"sty" => opcode_c_00!(0b000_100_00, line, addr, instr, lexer, false, true , false),
						"ldy" => opcode_c_00!(0b000_101_00, line, addr, instr, lexer, true , true , true ),
						"cpy" => opcode_c_00!(0b000_110_00, line, addr, instr, lexer, true , false, false),
						"cpx" => opcode_c_00!(0b000_111_00, line, addr, instr, lexer, true , false, false),

						// Bit
						"bit" => {
							line.opcode = 0b001_000_00;

							match instr.addr_mode {
								// bit $zp
								AddressingMode::ZeroPage(a) => {
									line.opcode |= 0b000_001_00;

									line.arg = match a {
										Address::Literal(a) => InstructionArg::ByteArg(parser::check_overflow(lexer, a)?),
										Address::Label(label) => InstructionArg::ByteLabelArg(label)
									};

									addr += 1;
								}

								// bit $addr
								AddressingMode::Absolute(a) => {
									line.opcode |= 0b000_011_00;

									line.arg = match a {
										Address::Literal(a) => InstructionArg::WordArg(a),
										Address::Label(label) => InstructionArg::WordLabelArg(label)
									};

									addr += 2;
								}

								_ => return ParseError::new(&lexer, &format!("Invalid argument for opcode '{}'", instr.opcode))
							}
						}

						// Jump
						"jmp" => {
							match instr.addr_mode {
								// jmp $addr
								AddressingMode::Absolute(a) => {
									line.opcode = 0x4C;

									line.arg = match a {
										Address::Literal(a) => InstructionArg::WordArg(a),
										Address::Label(label) => InstructionArg::WordLabelArg(label)
									};

									addr += 2;
								}

								// jmp ($addr)
								AddressingMode::Indirect(a) => {
									line.opcode = 0x6C;

									line.arg = match a {
										Address::Literal(a) => InstructionArg::WordArg(a),
										Address::Label(label) => InstructionArg::WordLabelArg(label)
									};

									addr += 2;
								}

								_ => return ParseError::new(&lexer, &format!("Invalid argument for opcode '{}'", instr.opcode))
							}
						}

						// Software interrupt
						"brk" => {
							line.opcode = 0x00;
							addr += 2;

							match instr.addr_mode {
								AddressingMode::Implicit => {}
								_ => return ParseError::new(&lexer, &format!("Invalid argument for opcode '{}'", instr.opcode))
							}
						}

						// Jump to subroutine
						"jsr" => {
							line.opcode = 0x20;

							match instr.addr_mode {
								AddressingMode::Absolute(a) => {
									line.arg = match a {
										Address::Literal(a) => InstructionArg::WordArg(a),
										Address::Label(label) => InstructionArg::WordLabelArg(label)
									}
								}

								_ => return ParseError::new(&lexer, &format!("Invalid argument for opcode '{}'", instr.opcode))
							}

							addr += 3;
						}

						// Branching instructions
						"bpl" => opcode_branch!(0x10, line, addr, instr, lexer),
						"bmi" => opcode_branch!(0x30, line, addr, instr, lexer),
						"bvc" => opcode_branch!(0x50, line, addr, instr, lexer),
						"bvs" => opcode_branch!(0x70, line, addr, instr, lexer),
						"bcc" => opcode_branch!(0x90, line, addr, instr, lexer),
						"bcs" => opcode_branch!(0xB0, line, addr, instr, lexer),
						"bne" => opcode_branch!(0xD0, line, addr, instr, lexer),
						"beq" => opcode_branch!(0xF0, line, addr, instr, lexer),

						// Opcodes that are a single byte
						// Returns
						"rti" => opcode_implicit!(0x40, line, addr, instr, lexer),
						"rts" => opcode_implicit!(0x60, line, addr, instr, lexer),

						// Stack operations
						"php" => opcode_implicit!(0x08, line, addr, instr, lexer),
						"plp" => opcode_implicit!(0x28, line, addr, instr, lexer),
						"pha" => opcode_implicit!(0x48, line, addr, instr, lexer),
						"pla" => opcode_implicit!(0x68, line, addr, instr, lexer),

						// Flag setting/clearing
						"clc" => opcode_implicit!(0x18, line, addr, instr, lexer),
						"sec" => opcode_implicit!(0x38, line, addr, instr, lexer),
						"cli" => opcode_implicit!(0x58, line, addr, instr, lexer),
						"sei" => opcode_implicit!(0x78, line, addr, instr, lexer),
						"clv" => opcode_implicit!(0xB8, line, addr, instr, lexer),
						"cld" => opcode_implicit!(0xD8, line, addr, instr, lexer),
						"sed" => opcode_implicit!(0xF8, line, addr, instr, lexer),

						// Incrementing/decrementing
						"inx" => opcode_implicit!(0xE8, line, addr, instr, lexer),
						"dex" => opcode_implicit!(0xCA, line, addr, instr, lexer),
						"iny" => opcode_implicit!(0xC8, line, addr, instr, lexer),
						"dey" => opcode_implicit!(0x88, line, addr, instr, lexer),

						// Transfering registers
						"tax" => opcode_implicit!(0xAA, line, addr, instr, lexer),
						"txa" => opcode_implicit!(0x8A, line, addr, instr, lexer),
						"tay" => opcode_implicit!(0xA8, line, addr, instr, lexer),
						"tya" => opcode_implicit!(0x98, line, addr, instr, lexer),
						"tsx" => opcode_implicit!(0xBA, line, addr, instr, lexer),
						"txs" => opcode_implicit!(0x9A, line, addr, instr, lexer),

						// Invalid opcode
						_ => return ParseError::new(lexer, &format!("Invalid opcode '{}'", instr.opcode))
					}

					lines.push(line);
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

				// Do nothing
				LineValue::None => {}
			}
		}

	}

	// Success!
	Ok(FirstPassResult {
		lines, symbol_table
	})
}
