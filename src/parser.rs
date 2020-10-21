//
// src
// parser.rs: Parses.
//
// Created by jenra.
// Created on October 19 2020.
//

use crate::lexer::Lexer;
use crate::lexer::TokenValue;

// Represents an immediate value
#[derive(Debug)]
pub enum ImmediateValue {
	Literal(u8),
	Label(String),
	LowByte(String),
	HighByte(String),
}

// Represents an address
#[derive(Debug)]
pub enum Address {
	Literal(u16),
	Label(String),
}

// Represents an addressing mode
#[derive(Debug)]
pub enum AddressingMode {
	Implicit,
	Immediate(ImmediateValue),
	ZeroPage(Address),
	ZeroPageX(Address),
	ZeroPageY(Address),
	Absolute(Address),
	AbsoluteX(Address),
	AbsoluteY(Address),
	IndirectX(Address),
	IndirectY(Address),
	Indirect(Address),
}

#[derive(Debug)]
pub struct Instruction {
	pub opcode: String,
	pub addr_mode: AddressingMode,
}

#[derive(Debug)]
pub enum Pragma {
	Byte(u8),
	Bytes(Vec<u8>),
	Word(Address),
	Origin(Address),
	Define(String, Address),
	Include(String),
}

#[derive(Debug)]
pub enum LineValue {
	None,
	Instruction(Instruction),
	Pragma(Pragma),
}

// Represents a line of 6502 assembly
#[derive(Debug)]
pub struct Line {
	pub lino: u32,
	pub label: String,
	pub value: LineValue,
}

#[derive(Debug)]
pub struct ParseError {
	pub lino: u32,
	pub message: String,
}

impl ParseError {
	pub fn new<T>(lexer: &Lexer, message: &str) -> Result<T, ParseError> {
		Err(ParseError {
			lino: lexer.get_lino(),
			message: String::from(message),
		})
	}
}

// Consumes a token
macro_rules! consume {
	($lexer: ident, $matchy: pat, $err: expr) => {
		if let Some(token) = $lexer.peek() {
			if let $matchy = token.value {
				$lexer.next();
				Ok(token)
			} else {
				ParseError::new($lexer, $err)
			}
		} else {
			ParseError::new($lexer, "Unexpected EOF")
		}
	};
}

// Looks ahead to see if a token matches the specified pattern
macro_rules! peek {
	($lexer: ident, $matchy: pat) => {
		if let Some(token) = $lexer.peek() {
			if let $matchy = token.value {
				Some(token)
			} else {
				None
			}
		} else {
			None
		}
	};
}

// Unwraps the value of a token
macro_rules! unwrap_token {
	($token: expr, $matchy: ident) => {
		match $token.value {
			TokenValue::$matchy(x) => x,
			_ => panic!(
				"Called unwrap_token for {} and received something else",
				stringify!($matchy)
			),
		}
	};
}

// Optionally consumes a token
macro_rules! optional {
	($lexer: ident, $matchy: pat) => {
		if let Some(token) = $lexer.peek() {
			if let $matchy = token.value {
				$lexer.next();
				Some(token)
			} else {
				None
			}
		} else {
			None
		}
	};
}

// Checks if the literal is under 256 and returns an ImmediateValue if it is, an error if not
fn check_overflow(lexer: &Lexer, n: u16) -> Result<u8, ParseError> {
	if n < 256 {
		Ok(n as u8)
	} else {
		ParseError::new(lexer, "Cannot use word as immediate value")
	}
}

// Parses an operand
fn parse_operand(lexer: &mut Lexer, instr: &mut Instruction) -> Result<(), ParseError> {
	// Immediate values (lda #imm)
	if let Some(_) = optional!(lexer, TokenValue::Hash) {
		// Get operand
		let operand = match lexer.next() {
			Some(token) => token,
			None => return ParseError::new(lexer, "Missing operand"),
		};

		// Match operand
		instr.addr_mode = AddressingMode::Immediate(match operand.value {
			TokenValue::Bin(n) => Ok(ImmediateValue::Literal(check_overflow(lexer, n)?)),
			TokenValue::Oct(n) => Ok(ImmediateValue::Literal(check_overflow(lexer, n)?)),
			TokenValue::Dec(n) => Ok(ImmediateValue::Literal(check_overflow(lexer, n)?)),
			TokenValue::Hex(n) => Ok(ImmediateValue::Literal(check_overflow(lexer, n)?)),
			TokenValue::Symbol(s) => Ok(ImmediateValue::Label(s)),

			TokenValue::LT => Ok(ImmediateValue::LowByte(unwrap_token!(
				consume!(lexer, TokenValue::Symbol(_), "Expected label")?,
				Symbol
			))),

			TokenValue::GT => Ok(ImmediateValue::HighByte(unwrap_token!(
				consume!(lexer, TokenValue::Symbol(_), "Expected label")?,
				Symbol
			))),

			_ => ParseError::new(lexer, "Expected literal, label, '<', or '>'"),
		}?);

	// Indirect addressing
	} else if let Some(_) = optional!(lexer, TokenValue::LParen) {
		// Get address token
		let addr = match lexer.next() {
			Some(token) => token,
			None => return ParseError::new(lexer, "Missing address"),
		};

		// Get address value
		let addr = match addr.value {
			TokenValue::Bin(n) => Address::Literal(n),
			TokenValue::Oct(n) => Address::Literal(n),
			TokenValue::Dec(n) => Address::Literal(n),
			TokenValue::Hex(n) => Address::Literal(n),
			TokenValue::Symbol(s) => Address::Label(s),
			_ => return ParseError::new(lexer, "Expected address"),
		};

		// Indirect X addressing (lda (addr, X))
		if let Some(_) = optional!(lexer, TokenValue::Comma) {
			// Consume X register
			let x = unwrap_token!(
				consume!(lexer, TokenValue::Symbol(_), "Expected X register")?,
				Symbol
			);
			if x != "x" && x != "X" {
				return ParseError::new(lexer, "Expected X register");
			}

			// Consume right parenthesis
			consume!(lexer, TokenValue::RParen, "Expected right parenthesis")?;

			// Save
			instr.addr_mode = AddressingMode::IndirectX(addr);

		// Indirect Y addressing and indirect addressing
		} else if let Some(_) = optional!(lexer, TokenValue::RParen) {
			// Indirect Y addressing (lda (addr), Y)
			if let Some(_) = optional!(lexer, TokenValue::Comma) {
				// Consume Y register
				let x = unwrap_token!(
					consume!(lexer, TokenValue::Symbol(_), "Expected Y register")?,
					Symbol
				);
				if x != "y" && x != "Y" {
					return ParseError::new(lexer, "Expected Y register");
				}

				// Save
				instr.addr_mode = AddressingMode::IndirectY(addr);

			// Indirect addressing (jmp (addr))
			} else {
				instr.addr_mode = AddressingMode::Indirect(addr);
			}

		// Unpaired right parenthesis
		} else {
			return ParseError::new(lexer, "Expected right parenthesis or comma");
		}

	// Everything else
	} else if let Some(token) = lexer.peek() {
		// Get address
		let addr = match token.value {
			TokenValue::Bin(n) => Address::Literal(n),
			TokenValue::Oct(n) => Address::Literal(n),
			TokenValue::Dec(n) => Address::Literal(n),
			TokenValue::Hex(n) => Address::Literal(n),
			TokenValue::Symbol(s) => Address::Label(s),
			_ => return Ok(()),
		};
		lexer.next();

		// X and Y index addressing
		instr.addr_mode = if let Some(_) = optional!(lexer, TokenValue::Comma) {
			let reg = unwrap_token!(
				consume!(lexer, TokenValue::Symbol(_), "Expected X or Y register")?,
				Symbol
			);

			// X indexing addressing
			if reg == "x" || reg == "X" {
				// Zero page (lda $00, x)
				if let Address::Literal(0..=255) = addr {
					AddressingMode::ZeroPageX(addr)

				// Absolute (lda $1234, x; lda label, x)
				} else {
					AddressingMode::AbsoluteX(addr)
				}

			// Y indexing addressing
			} else if reg == "y" || reg == "Y" {
				// Zero page (lda $00, y)
				if let Address::Literal(0..=255) = addr {
					AddressingMode::ZeroPageY(addr)

				// Absolute (lda $1234, y; lda label, y)
				} else {
					AddressingMode::AbsoluteY(addr)
				}

			// Error
			} else {
				return ParseError::new(lexer, "Expected X or Y register");
			}
		} else {
			// Zero page addressing (lda $00)
			if let Address::Literal(0..=255) = addr {
				AddressingMode::ZeroPage(addr)

			// Absolute addressing (lda $1234; lda label)
			} else {
				AddressingMode::Absolute(addr)
			}
		}
	}

	Ok(())
}

// Parse a pragma
fn parse_pragma(lexer: &mut Lexer) -> Result<Pragma, ParseError> {
	// Get the name of the pragma
	match unwrap_token!(
		consume!(lexer, TokenValue::Symbol(_), "Expected pragma after dot")?,
		Symbol
	)
	.as_str()
	{
		"byte" => {
			// Consume a byte
			if let Some(token) = lexer.next() {
				Ok(Pragma::Byte(match token.value {
					TokenValue::Bin(n) => check_overflow(lexer, n)?,
					TokenValue::Oct(n) => check_overflow(lexer, n)?,
					TokenValue::Dec(n) => check_overflow(lexer, n)?,
					TokenValue::Hex(n) => check_overflow(lexer, n)?,
					_ => return ParseError::new(lexer, "Expected byte after .byte"),
				}))
			} else {
				return ParseError::new(lexer, "Expected byte after .byte");
			}
		}

		"bytes" => {
			// Vector of bytes
			let mut bytes: Vec<u8> = Vec::new();

			// Get next token
			while let Some(token) = lexer.peek() {
				if let TokenValue::Newline = token.value {
					break;
				}
				lexer.next();

				match token.value {
					// Push bytes to vector
					TokenValue::Bin(n) => bytes.push(check_overflow(lexer, n)?),
					TokenValue::Oct(n) => bytes.push(check_overflow(lexer, n)?),
					TokenValue::Dec(n) => bytes.push(check_overflow(lexer, n)?),
					TokenValue::Hex(n) => bytes.push(check_overflow(lexer, n)?),
					TokenValue::String(s) => {
						for b in s.bytes() {
							bytes.push(b);
						}
					}

					// Not a byte
					_ => return ParseError::new(lexer, "Invalid byte"),
				}

				// Commas
				optional!(lexer, TokenValue::Comma);
			}

			Ok(Pragma::Bytes(bytes))
		}

		"word" => {
			// Consume a word
			if let Some(token) = lexer.next() {
				Ok(Pragma::Word(match token.value {
					TokenValue::Bin(n) => Address::Literal(n),
					TokenValue::Oct(n) => Address::Literal(n),
					TokenValue::Dec(n) => Address::Literal(n),
					TokenValue::Hex(n) => Address::Literal(n),
					TokenValue::Symbol(s) => Address::Label(s),
					_ => return ParseError::new(lexer, "Expected address or label after .word"),
				}))
			} else {
				ParseError::new(lexer, "Expected address or label after .word")
			}
		}

		"origin" => {
			// Consume an address
			if let Some(token) = lexer.next() {
				Ok(Pragma::Origin(match token.value {
					TokenValue::Bin(n) => Address::Literal(n),
					TokenValue::Oct(n) => Address::Literal(n),
					TokenValue::Dec(n) => Address::Literal(n),
					TokenValue::Hex(n) => Address::Literal(n),
					TokenValue::Symbol(s) => Address::Label(s),
					_ => return ParseError::new(lexer, "Expected address or label after .origin"),
				}))
			} else {
				ParseError::new(lexer, "Expected address or label after .origin")
			}
		}

		"define" => {
			// Consume label
			let label = unwrap_token!(
				consume!(lexer, TokenValue::Symbol(_), "Expected label after .define")?,
				Symbol
			);

			// Consume address
			if let Some(token) = lexer.next() {
				Ok(Pragma::Define(
					label,
					match token.value {
						TokenValue::Bin(n) => Address::Literal(n),
						TokenValue::Oct(n) => Address::Literal(n),
						TokenValue::Dec(n) => Address::Literal(n),
						TokenValue::Hex(n) => Address::Literal(n),
						TokenValue::Symbol(s) => Address::Label(s),
						_ => {
							return ParseError::new(
								lexer,
								"Expected address or label after .define [label]",
							)
						}
					},
				))
			} else {
				ParseError::new(lexer, "Expected address or label after .define [label]")
			}
		}

		"include" => {
			// Consume include path as string
			if let Some(token) = optional!(lexer, TokenValue::String(_)) {
				Ok(Pragma::Include(unwrap_token!(token, String)))
			} else {
				ParseError::new(lexer, "Expected string with include path")
			}
		}

		// Invalid pragma
		_ => ParseError::new(lexer, "Invalid pragma"),
	}
}

// Parses everything in the line after the label
fn parse_post_label(lexer: &mut Lexer) -> Result<LineValue, ParseError> {
	// Parse instruction
	if let Some(token) = optional!(lexer, TokenValue::Symbol(_)) {
		let mut instr = Instruction {
			opcode: unwrap_token!(token, Symbol),
			addr_mode: AddressingMode::Implicit,
		};

		// Parse operand
		parse_operand(lexer, &mut instr)?;
		Ok(LineValue::Instruction(instr))

	// Parse pragma
	} else if let Some(_) = optional!(lexer, TokenValue::Dot) {
		Ok(LineValue::Pragma(parse_pragma(lexer)?))

	// Nothing after label
	} else {
		Ok(LineValue::None)
	}
}

// Parses a line of 6502 assembly
pub fn parse_line(lexer: &mut Lexer) -> Result<Option<Line>, ParseError> {
	// Skip newlines
	while let Some(_) = peek!(lexer, TokenValue::Newline) {
		lexer.next();
	}

	// The line to eventually return
	let mut line = Line {
		lino: lexer.get_lino(),
		label: String::from(""),
		value: LineValue::None,
	};

	// Stop parsing if there's nothing left
	if let None = lexer.peek() {
		return Ok(None);
	}

	// First token
	let state = lexer.save();
	let label = optional!(lexer, TokenValue::Symbol(_));

	// First token is a label
	if let Some(label) = label {
		if let Some(_) = optional!(lexer, TokenValue::Colon) {
			line.label = unwrap_token!(label, Symbol);
		} else {
			lexer.recall(state);
		}
	}

	// Parse everything after the label
	line.value = parse_post_label(lexer)?;

	// Parse newline if not at eof
	if let Some(_) = lexer.peek() {
		consume!(lexer, TokenValue::Newline, "Expected end of line")?;
	}

	// Success!
	Ok(Some(line))
}

// Parses a 6502 assembly file
pub fn parse(lexer: &mut Lexer) -> Result<Vec<Line>, ParseError> {
	let mut lines = Vec::new();

	// Iterate through all tokens
	loop {
		// Get next line
		if let Some(line) = parse_line(lexer)? {
			lines.push(line);
		} else {
			break;
		}
	}

	// Success!
	Ok(lines)
}
