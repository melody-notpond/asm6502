// 
// src
// lexer.rs: Splits the file into tokens.
// 
// Created by jenra.
// Created on October 18 2020.
// 

// Represents the type of the token.
enum TokenType
{
	// Symbol (labels, opcodes, pragmas, etc)
	Symbol(String),

	// Values
	Byte(u8),
	Word(u16),
	String(Vec<u8>),
	Char(u8),

	// Parentheses
	LParen,
	RParen,

	// Miscellaneous characters
	Colon,
	Comma,
	Newline,
	LT,
	GT,
	Dot
}

// Represents a token.
pub struct Token
{
	// Where the token was generated
	pos: usize,
	lino: u32,
	charpos: u32,

	// The type of the token
	t: TokenType
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
	tokens: Vec<&'a Token>,
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
}

impl<'a> Iterator for Lexer<'a>
{
	type Item = &'a Token;

	fn next(&mut self) -> Option<&'a Token>
	{
		// If there's a next token on the list, yield it
		if self.token_pos < self.tokens.len()
		{
			Some(&self.tokens[self.token_pos])
		} else
		{
			// TODO
			None
		}
	}
}
