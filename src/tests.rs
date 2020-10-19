//
// src
// test.rs: Implements tests.
//
// Created by jenra.
// Created on October 18 2020.
//

use crate::lexer::*;

#[test]
fn lexer_misc_chars() {
	let string = String::from(".< >()\n#~");
	let mut lexer = Lexer::new(&string);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Dot);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::LT);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::GT);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::LParen);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::RParen);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Newline);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Hash);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Err(String::from("~")));
	assert!(if let None = lexer.next() { true } else { false });
}

#[test]
fn lexer_symbols() {
	let string = String::from("hewwo HEWWO _underscore");
	let mut lexer = Lexer::new(&string);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Symbol(String::from("hewwo")));
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Symbol(String::from("HEWWO")));
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Symbol(String::from("_underscore")));
	assert!(if let None = lexer.next() { true } else { false });
}

#[test]
fn lexer_numbers() {
	let string = String::from("%00101010 052 42 $2a $2A");
	let mut lexer = Lexer::new(&string);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Bin(42));
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Oct(42));
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Dec(42));
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Hex(42));
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Hex(42));
	assert!(if let None = lexer.next() { true } else { false });
}
