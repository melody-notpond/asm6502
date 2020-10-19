//
// src
// test.rs: [description].
//
// Created by jenra.
// Created on October 18 2020.
//

use crate::lexer::*;

#[test]
fn lexer_symbols() {
	let string = String::from(".< >()\n#~");
	let mut lexer = Lexer::new(&string);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Dot);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::LT);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::GT);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::LParen);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::RParen);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Newline);
	assert_eq!(lexer.next().unwrap().token_type, TokenType::Hash);
	assert_eq!(
		lexer.next().unwrap().token_type,
		TokenType::Err(String::from("~"))
	);
	assert!(if let None = lexer.next() { true } else { false });
}
