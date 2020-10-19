// 
// src
// lexer.rs: Splits the file into tokens.
// 
// Created by jenra.
// Created on October 18 2020.
// 

// Represents the type of the token.
#[derive(Debug)]
pub enum TokenType
{
	// No token
	None,

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
	Char(u8)
}

// Represents a token.
#[derive(Debug)]
pub struct Token
{
	// Where the token was generated
	pub pos: usize,
	pub lino: u32,
	pub charpos: u32,

	// The type of the token
	pub token_type: TokenType
}

// Represents a lexer state.
#[derive(Copy, Clone)]
pub struct LexerState
{
	pos: usize,
	lino: u32,
	charpos: u32
}

// Represents a lexer.
pub struct Lexer<'a>
{
	// The state of the lexer
	state: LexerState,

	// The list of tokens
	tokens: Vec<Token>,
	token_pos: usize,

	// The string being parsed
	string: &'a String
}

impl<'a> Lexer<'a>
{
	// Creates a new lexer
	pub fn new(string: &'a String) -> Lexer<'a>
	{
		Lexer {
			state: LexerState {
				pos: 0,
				lino: 1,
				charpos: 0
			},
			tokens: Vec::new(),
			token_pos: 0,
			string: &string
		}
	}

	pub fn next(&mut self) -> &Token
	{
		// If there's a next token on the list, yield it
		if self.token_pos < self.tokens.len()
		{
			&self.tokens[self.token_pos]
		} else
		{
			// The token we will eventually return
			let mut token = Token {
				pos: self.state.pos,
				lino: self.state.lino,
				charpos: self.state.charpos,
				token_type: TokenType::None
			};

			// Iterate over the characters of the string
			let mut token_pos = self.token_pos;
			for c in self.string[self.token_pos..].char_indices()
			{
				match token.token_type
				{
					// No type has been assigned to the token
					TokenType::None => {
						// Error token (unknown character)
						if self.token_pos != c.0
						{
							token_pos = c.0;
							break;

						// Symbol characters and newline
						} else if c.1 == '('
						{
							token.token_type = TokenType::LParen;
						} else if c.1 == ')'
						{
							token.token_type = TokenType::RParen;
						} else if c.1 == ':'
						{
							token.token_type = TokenType::Colon;
						} else if c.1 == ','
						{
							token.token_type = TokenType::Comma;
						} else if c.1 == '\n'
						{
							token.token_type = TokenType::Newline;
						} else if c.1 == '<'
						{
							token.token_type = TokenType::LT;
						} else if c.1 == '>'
						{
							token.token_type = TokenType::GT;
						} else if c.1 == '.'
						{
							token.token_type = TokenType::Dot;
						} else if c.1 == '#'
						{
							token.token_type = TokenType::Hash;
						}
					},

					// Type of the token is only one character
					_ => {
						token_pos = c.0;
						break;
					}
				}
			}

			// Save the token and return it
			self.tokens.push(token);
			self.token_pos = token_pos;
			let token = self.tokens.last().unwrap();
			token
		}
	}

	pub fn save(&self) -> LexerState
	{
		self.state
	}

	pub fn recall(&mut self, state: LexerState)
	{
		self.state = state
	}
}
