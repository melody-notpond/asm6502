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
	HighByte(String)
}

// Represents an address
#[derive(Debug)]
pub enum Address {
	Literal(u16),
	Label(String)
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
	Indirect(Address)
}

// Represents a line of 6502 assembly
#[derive(Debug)]
pub struct Line {
	pub label: String,
	pub opcode: String,
	pub addr_mode: AddressingMode
}

// Consumes a token
macro_rules! consume {
	($lexer: ident, $matchy: pat, $err: expr) => {
		if let Some(token) = $lexer.peek() {
			if let $matchy = token.value {
				$lexer.next();
				Ok(token)
			} else {
				Err(String::from($err))
			}
		} else {
			Err(String::from("End of file reached"))
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
			_ => panic!("Called unwrap_token for {} and received something else", stringify!($matchy))
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
fn check_overflow(n: u16) -> Result<ImmediateValue, String>
{
	if n < 256 {
		Ok(ImmediateValue::Literal(n as u8))
	} else {
		Err(String::from("Cannot use word as immediate value"))
	}
}

// Parses an operand
fn parse_operand(lexer: &mut Lexer, line: &mut Line) -> Result<(), String> {
	// Immediate values (lda #imm)
	if let Some(_) = optional!(lexer, TokenValue::Hash) {
		// Get operand
		let operand = match lexer.next() {
			Some(token) => token,
			None => return Err(String::from("Missing operand"))
		};

		// Match operand
		line.addr_mode = AddressingMode::Immediate(
			match operand.value {
				TokenValue::Bin(n) => check_overflow(n),
				TokenValue::Oct(n) => check_overflow(n),
				TokenValue::Dec(n) => check_overflow(n),
				TokenValue::Hex(n) => check_overflow(n),
				TokenValue::Symbol(s) => Ok(ImmediateValue::Label(s)),

				TokenValue::LT => {
					Ok(ImmediateValue::LowByte(
						unwrap_token!(consume!(lexer, TokenValue::Symbol(_), "Expected label")?, Symbol))
					)
				}

				TokenValue::GT => {
					Ok(ImmediateValue::HighByte(
						unwrap_token!(consume!(lexer, TokenValue::Symbol(_), "Expected label")?, Symbol))
					)
				}

				_ => Err(String::from(""))
			}?
		);

	// Indirect addressing
	} else if let Some(_) = optional!(lexer, TokenValue::LParen) {
		// Get address token
		let addr = match lexer.next() {
			Some(token) => token,
			None => return Err(String::from("Missing address"))
		};

		// Get address value
		let addr = match addr.value {
			TokenValue::Bin(n) => Address::Literal(n),
			TokenValue::Oct(n) => Address::Literal(n),
			TokenValue::Dec(n) => Address::Literal(n),
			TokenValue::Hex(n) => Address::Literal(n),
			TokenValue::Symbol(s) => Address::Label(s),
			_ => return Err(String::from("Expected address"))
		};

		// Indirect X addressing (lda (addr, X))
		if let Some(_) = optional!(lexer, TokenValue::Comma) {
			// Consume X register
			let x = unwrap_token!(consume!(lexer, TokenValue::Symbol(_), "Expected X register")?, Symbol);
			if x != "x" && x != "X" {
				return Err(String::from("Expected X register"));
			}

			// Consume right parenthesis
			consume!(lexer, TokenValue::RParen, "Expected right parenthesis")?;

			// Save
			line.addr_mode = AddressingMode::IndirectX(addr);

		// Indirect Y addressing and indirect addressing
		} else if let Some(_) = optional!(lexer, TokenValue::RParen) {
			// Indirect Y addressing (lda (addr), Y)
			if let Some(_) = optional!(lexer, TokenValue::Comma) {
				// Consume Y register
				let x = unwrap_token!(consume!(lexer, TokenValue::Symbol(_), "Expected Y register")?, Symbol);
				if x != "y" && x != "Y" {
					return Err(String::from("Expected Y register"));
				}

				// Save
				line.addr_mode = AddressingMode::IndirectY(addr);

			// Indirect addressing (jmp (addr))
			} else {
				line.addr_mode = AddressingMode::Indirect(addr);
			}

		// Unpaired right parenthesis
		} else {
			return Err(String::from("Expected right parenthesis or comma"));
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
			_ => return Ok(())
		};

		lexer.next();
		// X and Y index addressing
		line.addr_mode = if let Some(_) = optional!(lexer, TokenValue::Comma) {
			let reg = unwrap_token!(consume!(lexer, TokenValue::Symbol(_), "Expected X or Y register")?, Symbol);

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
				return Err(String::from("Expected X or Y register"));
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

// Parses a line of 6502 assembly
fn parse_line(lexer: &mut Lexer) -> Result<Line, String> {
	let mut line = Line {
		label: String::from(""),
		opcode: String::from(""),
		addr_mode: AddressingMode::Implicit
	};

	// First token
	let start_of_line = consume!(lexer, TokenValue::Symbol(_), "Expected opcode or label")?;

	// First token is a label
	if let Some(_) = optional!(lexer, TokenValue::Colon) {
		line.label = unwrap_token!(start_of_line, Symbol);

		// Optionally consume an opcode
		if let Some(token) = optional!(lexer, TokenValue::Symbol(_)) {
			line.opcode = unwrap_token!(token, Symbol);
		}

	// First token is an opcode
	} else {
		line.opcode = unwrap_token!(start_of_line, Symbol);
	}

	// Parse the operand
	if line.opcode != "" {
		parse_operand(lexer, &mut line)?;
	}

	// Parse newline if not at eof
	if let Some(_) = lexer.peek() {
		consume!(lexer, TokenValue::Newline, "Expected end of line")?;
	}

	// Success!
	Ok(line)
}

// Parses a 6502 assembly file
pub fn parse(lexer: &mut Lexer) -> Result<Vec<Line>, String> {
	let mut lines = Vec::new();

	// Iterate through all tokens
	loop {
		// Skip newlines
		while let Some(_) = peek!(lexer, TokenValue::Newline) { lexer.next(); }

		// Stop parsing if there's nothing left
		if let None = lexer.peek() { break; }

		// Get next line
		let line = parse_line(lexer)?;
		lines.push(line);
	}

	// Success!
	Ok(lines)
}
