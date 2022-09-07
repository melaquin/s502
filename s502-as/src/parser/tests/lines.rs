use logos::Logos;

use super::super::parse_line;
use crate::lexer::Token;

#[test]
fn empty_line() {
    let source = "".to_string();
    let source_name = "empty line test".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let line = parse_line(&mut lexer, &source_name);
    assert!(line.is_ok());
    assert!(line.unwrap().is_none());
}

#[test]
fn label_line() {
    let source = "mylabel".to_string();
    let source_name = "label line test".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let line = parse_line(&mut lexer, &source_name);
    assert!(line.is_ok());
    assert!(line.unwrap().unwrap().label.is_some());
}
