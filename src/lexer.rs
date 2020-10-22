//
// src
// lexer.rs: Splits the file into tokens.
//
// Created by jenra.
// Created on October 18 2020.
//

// Represents the value of the token.
#[derive(Debug, PartialEq)]
pub enum TokenValue {
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
	String(String),
}

// Represents a token.
#[derive(Debug)]
pub struct Token {
	// Where the token was generated
	pub pos: usize,
	pub lino: u32,
	pub charpos: u32,

	// The type of the token
	pub value: TokenValue,
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
	filename: String,

	// The state of the lexer
	state: LexerState,

	// The string being parsed
	string: String,
}

impl Lexer {
	// Creates a new lexer
	pub fn new(filename: &str, string: &str) -> Lexer {
		let mut string = String::from(string);
		string.push(' ');
		Lexer {
			filename: String::from(filename),
			state: LexerState {
				pos: 0,
				lino: 1,
				charpos: 0,
			},
			string: string,
		}
	}

	// Returns the next token without updating the iterator
	pub fn peek(&mut self) -> Option<Token> {
		let state = self.state;
		let token = self.next();
		self.state = state;
		token
	}

	// Skips whitespace
	fn skip_whitespace(&mut self) {
		let mut in_comment = false;

		for c in self.string[self.state.pos..].char_indices() {
			// Comments end with a newline
			if in_comment && c.1 == '\n' {
				self.state.pos += 1;
				self.state.charpos = 0;
				self.state.lino += 1;
				in_comment = false

			// Skip whitespace and comments
			} else if c.1 == ' ' || c.1 == '\t' || in_comment {
				self.state.pos += 1;
				self.state.charpos += 1;

			// Mark semicolons as the start of a comment
			} else if c.1 == ';' {
				in_comment = true;
				self.state.pos += 1;
				self.state.charpos += 1;

			// Stop skipping if there's no more comments or whitespace
			} else {
				break;
			}
		}
	}

	pub fn get_lino(&self) -> u32 {
		self.state.lino
	}

	pub fn get_filename(&self) -> &String {
		&self.filename
	}

	pub fn save(&self) -> LexerState {
		self.state
	}

	pub fn recall(&mut self, state: LexerState) {
		self.state = state;
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
			value: TokenValue::None,
		};

		// Iterate over the characters of the string
		for c in self.string[self.state.pos..].char_indices() {
			match &mut token.value {
				// No type has been assigned to the token
				TokenValue::None => {
					// Error token (unknown character)
					if c.0 != 0 {
						token.value = TokenValue::Err(format!(
							"Invalid token '{}'",
							&self.string[self.state.pos..self.state.pos + c.0]
						));
						self.state.pos += c.0;
						break;

					// Symbol characters and newline
					} else if c.1 == '(' {
						token.value = TokenValue::LParen;
					} else if c.1 == ')' {
						token.value = TokenValue::RParen;
					} else if c.1 == ':' {
						token.value = TokenValue::Colon;
					} else if c.1 == ',' {
						token.value = TokenValue::Comma;
					} else if c.1 == '\n' {
						token.value = TokenValue::Newline;

						// Update lines
						self.state.charpos = 0;
						self.state.lino += 1;
					} else if c.1 == '<' {
						token.value = TokenValue::LT;
					} else if c.1 == '>' {
						token.value = TokenValue::GT;
					} else if c.1 == '.' {
						token.value = TokenValue::Dot;
					} else if c.1 == '#' {
						token.value = TokenValue::Hash;

					// Symbols
					} else if ('a' <= c.1 && c.1 <= 'z') || ('A' <= c.1 && c.1 <= 'Z') || c.1 == '_'
					{
						token.value = TokenValue::Symbol(String::from(""));

					// Number literals
					} else if c.1 == '%' {
						token.value = TokenValue::Bin(0);
					} else if c.1 == '0' {
						token.value = TokenValue::Oct(0);
					} else if '1' <= c.1 && c.1 <= '9' {
						token.value = TokenValue::Dec(0);
					} else if c.1 == '$' {
						token.value = TokenValue::Hex(0);

					// Strings
					} else if c.1 == '"' {
						token.value = TokenValue::String(String::new());
					}
				}

				TokenValue::Symbol(s) => {
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

				TokenValue::Bin(v) => {
					if !(c.1 == '0' || c.1 == '1') {
						// Parse
						let string = &self.string[self.state.pos + 1..self.state.pos + c.0];
						let parsed = u16::from_str_radix(string, 2);

						// Check for overflow
						match parsed {
							Ok(n) => *v = n,
							Err(_) => {
								token.value = TokenValue::Err(format!(
									"'{}' is an invalid 16 bit integer",
									string
								));
							}
						}

						// Exit the loop
						self.state.pos += c.0;
						break;
					}
				}

				TokenValue::Oct(v) => {
					if !('0' <= c.1 && c.1 <= '7') {
						if c.0 == 1 {
							token.value = TokenValue::Dec(0);
						} else {
							// Parse
							let string = &self.string[self.state.pos + 1..self.state.pos + c.0];
							let parsed = u16::from_str_radix(string, 8);

							// Check for overflow
							match parsed {
								Ok(n) => *v = n,
								Err(_) => {
									token.value = TokenValue::Err(format!(
										"'{}' is an invalid 16 bit integer",
										string
									));
								}
							}

							// Exit the loop
							self.state.pos += c.0;
							break;
						}

						self.state.pos += c.0;
						break;
					}
				}

				TokenValue::Dec(v) => {
					if !('0' <= c.1 && c.1 <= '9') {
						// Parse
						let string = &self.string[self.state.pos..self.state.pos + c.0];
						let parsed = u16::from_str_radix(string, 10);

						// Check for overflow
						match parsed {
							Ok(n) => *v = n,
							Err(_) => {
								token.value = TokenValue::Err(format!(
									"'{}' is an invalid 16 bit integer",
									string
								));
							}
						}

						// Exit the loop
						self.state.pos += c.0;
						break;
					}
				}

				TokenValue::Hex(v) => {
					if !(('0' <= c.1 && c.1 <= '9')
						|| ('a' <= c.1 && c.1 <= 'f')
						|| ('A' <= c.1 && c.1 <= 'F'))
					{
						// Parse
						let string = &self.string[self.state.pos + 1..self.state.pos + c.0];
						let parsed = u16::from_str_radix(string, 16);

						// Check for overflow
						match parsed {
							Ok(n) => *v = n,
							Err(_) => {
								token.value = TokenValue::Err(format!(
									"'{}' is an invalid 16 bit integer",
									string
								));
							}
						}

						// Exit the loop
						self.state.pos += c.0;
						break;
					}
				}

				TokenValue::String(s) => {
					if c.1 == '"' {
						self.state.pos += c.0 + 1;
						break;
					} else if c.0 == self.string.len() - self.state.pos - 1 {
						token.value = TokenValue::Err(String::from(
							&self.string[self.state.pos..self.state.pos + c.0],
						));
						self.state.pos += c.0;
						break;
					} else {
						s.push(c.1);
					}
				}

				// Type of the token is only one character
				_ => {
					self.state.pos += c.0;
					break;
				}
			}

			// Update char position if not newline
			if token.value != TokenValue::Newline {
				self.state.charpos += 1;
			}
		}

		if token.value == TokenValue::None {
			None
		} else {
			Some(token)
		}
	}
}
