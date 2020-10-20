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

fn check_overflow(n: u16) -> Result<ImmediateValue, String>
{
	if n < 256 {
		Ok(ImmediateValue::Literal(n as u8))
	} else {
		Err(String::from("Cannot use word as immediate value"))
	}
}

fn parse_operand(lexer: &mut Lexer, line: &mut Line) -> Result<(), String> {
	// Immediate values
	if let Some(_) = optional!(lexer, TokenValue::Hash) {
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
				_ => Err(String::from(""))
			}?
		);
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
