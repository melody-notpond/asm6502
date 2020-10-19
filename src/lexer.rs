//
// src
// lexer.rs: Splits the file into tokens.
//
// Created by jenra.
// Created on October 18 2020.
//

// Represents the type of the token.
#[derive(Debug, PartialEq)]
pub enum TokenType {
	// No token
	None,
	Err(String),

	// Parentheses
	LParen,
	RParen,

	// Miscellaneous characters
	Colon,
	Comma,
	Newline,
	LT,
	GT,
	Dot,
	Hash,

	// Symbol (labels, opcodes, pragmas, etc)
	Symbol(String),

	// Values
	Bin(u16),
	Oct(u16),
	Dec(u16),
	Hex(u16),
	String(Vec<u8>),
	Char(u8),
}

// Represents a token.
#[derive(Debug)]
pub struct Token {
	// Where the token was generated
	pub pos: usize,
	pub lino: u32,
	pub charpos: u32,

	// The type of the token
	pub token_type: TokenType,
}

// Represents a lexer state.
#[derive(Copy, Clone)]
pub struct LexerState {
	pos: usize,
	lino: u32,
	charpos: u32,
}

// Represents a lexer.
pub struct Lexer {
	// The state of the lexer
	state: LexerState,

	// The string being parsed
	string: String,
}

impl Lexer {
	// Creates a new lexer
	pub fn new(string: &str) -> Lexer {
		let mut string = String::from(string);
		string.push(' ');
		Lexer {
			state: LexerState {
				pos: 0,
				lino: 1,
				charpos: 0,
			},
			string: string,
		}
	}

	fn skip_whitespace(&mut self) {
		for c in self.string[self.state.pos..].char_indices() {
			if c.1 == ' ' || c.1 == '\t' {
				self.state.pos += 1;
				self.state.charpos += 1;
			} else {
				break;
			}
		}
	}

	pub fn save(&self) -> LexerState {
		self.state
	}

	pub fn recall(&mut self, state: LexerState) {
		self.state = state
	}
}

impl Iterator for Lexer {
	type Item = Token;

	fn next(&mut self) -> Option<Token> {
		// Skip whitespace
		self.skip_whitespace();

		// The token we will eventually return
		let mut token = Token {
			pos: self.state.pos,
			lino: self.state.lino,
			charpos: self.state.charpos,
			token_type: TokenType::None,
		};

		// Iterate over the characters of the string
		for c in self.string[self.state.pos..].char_indices() {
			match &mut token.token_type {
				// No type has been assigned to the token
				TokenType::None => {
					// Error token (unknown character)
					if c.0 != 0 {
						token.token_type = TokenType::Err(String::from(
							&self.string[self.state.pos..self.state.pos + c.0],
						));
						self.state.pos += c.0;
						break;

					// Symbol characters and newline
					} else if c.1 == '(' {
						token.token_type = TokenType::LParen;
					} else if c.1 == ')' {
						token.token_type = TokenType::RParen;
					} else if c.1 == ':' {
						token.token_type = TokenType::Colon;
					} else if c.1 == ',' {
						token.token_type = TokenType::Comma;
					} else if c.1 == '\n' {
						token.token_type = TokenType::Newline;

						// Update lines
						self.state.charpos = 0;
						self.state.lino += 1;
					} else if c.1 == '<' {
						token.token_type = TokenType::LT;
					} else if c.1 == '>' {
						token.token_type = TokenType::GT;
					} else if c.1 == '.' {
						token.token_type = TokenType::Dot;
					} else if c.1 == '#' {
						token.token_type = TokenType::Hash;

					// Symbols
					} else if ('a' <= c.1 && c.1 <= 'z') || ('A' <= c.1 && c.1 <= 'Z') || c.1 == '_'
					{
						token.token_type = TokenType::Symbol(String::from(""));

					// Number literals
					} else if c.1 == '%' {
						token.token_type = TokenType::Bin(0);
					} else if c.1 == '0' {
						token.token_type = TokenType::Oct(0);
					} else if '1' <= c.1 && c.1 <= '9' {
						token.token_type = TokenType::Dec(0);
					} else if c.1 == '$' {
						token.token_type = TokenType::Hex(0);
					}
				}

				TokenType::Symbol(s) => {
					if !(('a' <= c.1 && c.1 <= 'z')
					  || ('A' <= c.1 && c.1 <= 'Z')
					  || ('0' <= c.1 && c.1 <= '9')
					  || c.1 == '_')
					{
						s.push_str(&self.string[self.state.pos..self.state.pos + c.0]);
						self.state.pos += c.0;
						break;
					}
				}

				TokenType::Bin(v) => {
					if !(c.1 == '0' || c.1 == '1') {
						let string = &self.string[self.state.pos + 1..self.state.pos + c.0];
						*v = u16::from_str_radix(string, 2).unwrap();
						self.state.pos += c.0;
						break;
					}
				}

				TokenType::Oct(v) => {
					if !('0' <= c.1 && c.1 <= '7') {
						let string = &self.string[self.state.pos + 1..self.state.pos + c.0];
						*v = u16::from_str_radix(string, 8).unwrap();
						self.state.pos += c.0;
						break;
					}
				}

				TokenType::Dec(v) => {
					if !('0' <= c.1 && c.1 <= '9') {
						let string = &self.string[self.state.pos..self.state.pos + c.0];
						*v = u16::from_str_radix(string, 10).unwrap();
						self.state.pos += c.0;
						break;
					}
				}

				TokenType::Hex(v) => {
					if !(('0' <= c.1 && c.1 <= '9') || ('a' <= c.1 && c.1 <= 'f') || ('A' <= c.1 && c.1 <= 'F')) {
						let string = &self.string[self.state.pos + 1..self.state.pos + c.0];
						*v = u16::from_str_radix(string, 16).unwrap();
						self.state.pos += c.0;
						break;
					}
				}

				// Type of the token is only one character
				_ => {
					self.state.pos += c.0;
					break;
				}
			}

			// Update char position if not newline
			if token.token_type != TokenType::Newline {
				self.state.charpos += 1;
			}
		}

		if token.token_type == TokenType::None {
			None
		} else {
			Some(token)
		}
	}
}
