use crate::{ast::*, error::AssemblerError};
use logos::Logos;

use super::super::parse_label;
use crate::lexer::Token;

#[test]
fn plain() {
    let source = "mylabel".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let label = parse_label(&mut lexer);
    assert!(label.is_ok());

    assert_eq!(
        label.unwrap().unwrap(),
        Label::Top(TopLabel {
            name: "mylabel".to_string(),
            span: 0..7,
            visibility: Visibility::Object,
            sublabels: vec![],
        })
    );
}

#[test]
fn global() {
    let source = "!yourlabel".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let label = parse_label(&mut lexer);
    assert!(label.is_ok());

    assert_eq!(
        label.unwrap().unwrap(),
        Label::Top(TopLabel {
            name: "yourlabel".to_string(),
            span: 1..10,
            visibility: Visibility::Global,
            sublabels: vec![],
        })
    );
}

#[test]
fn sublabel() {
    let source = ".sublabel".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let sublabel = parse_label(&mut lexer);
    assert!(sublabel.is_ok());

    assert_eq!(
        sublabel.unwrap().unwrap(),
        Label::Sub(SubLabel {
            name: "sublabel".to_string(),
            span: 0..9,
        })
    );
}

#[test]
fn no_ident_after_global() {
    let source = "!adc".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let parse_result = parse_label(&mut lexer);
    assert!(parse_result.is_err());

    assert_eq!(
        parse_result.unwrap_err(),
        AssemblerError {
            span: 1..4,
            message: "Unexpected token `adc`".to_string(),
            note: Some("Expected a label".to_string()),
        }
    );
}

#[test]
fn no_ident_after_period() {
    let source = ".dfb".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let parse_result = parse_label(&mut lexer);
    assert!(parse_result.is_err());

    assert_eq!(
        parse_result.unwrap_err(),
        AssemblerError {
            span: 1..4,
            message: "Unexpected token `dfb`".to_string(),
            note: Some("Expected a label".to_string()),
        }
    );
}
