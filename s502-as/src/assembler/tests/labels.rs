use crate::{ast::*, error::AssemblerError};
use logos::Logos;

use crate::lexer::Token;

use super::super::Assembler;

#[test]
fn plain() {
    let source = "mylabel".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();
    let mut assembler = Assembler { labels: Vec::new() };

    assert!(assembler.parse_label(&mut lexer).is_ok());
    assert!(assembler.labels.last().is_some());

    assert_eq!(
        assembler.labels.last().unwrap(),
        &Label {
            name: "mylabel".to_string(),
            span: 0..7,
            visibility: Visibility::Object,
            sublabels: vec![],
        }
    );
}

#[test]
fn global() {
    let source = "!yourlabel".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();
    let mut assembler = Assembler { labels: Vec::new() };

    assert!(assembler.parse_label(&mut lexer).is_ok());
    assert!(assembler.labels.last().is_some());

    assert_eq!(
        assembler.labels.last().unwrap(),
        &Label {
            name: "yourlabel".to_string(),
            span: 1..10,
            visibility: Visibility::Global,
            sublabels: vec![],
        }
    );
}

#[test]
fn no_ident_after_global() {
    let source = "!adc".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();
    let mut assembler = Assembler { labels: Vec::new() };

    let parse_result = assembler.parse_label(&mut lexer);
    assert!(parse_result.is_err());
    assert!(assembler.labels.last().is_none());

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
    let mut assembler = Assembler { labels: Vec::new() };

    let parse_result = assembler.parse_label(&mut lexer);
    assert!(parse_result.is_err());
    assert!(assembler.labels.last().is_none());

    assert_eq!(
        parse_result.unwrap_err(),
        AssemblerError {
            span: 1..4,
            message: "Unexpected token `dfb`".to_string(),
            note: Some("Expected a label".to_string()),
        }
    );
}

#[test]
fn child_with_no_parent() {
    let source = ".subl".to_string();
    let mut lexer = Token::lexer(&source).spanned().peekable();
    let mut assembler = Assembler { labels: Vec::new() };

    let parse_result = assembler.parse_label(&mut lexer);
    assert!(parse_result.is_err());
    assert!(assembler.labels.last().is_none());

    assert_eq!(
        parse_result.unwrap_err(),
        AssemblerError {
            span: 0..5,
            message: "Sublabel without top-level label".to_string(),
            note: Some("No top level label has been created for `subl` to go under".to_string()),
        }
    );
}

#[test]
fn child_after_parent() {
    let toplevel_source = "parent".to_string();
    let sublevel_source = ".sublabel".to_string();
    let mut lexer = Token::lexer(&toplevel_source).spanned().peekable();
    let mut assembler = Assembler { labels: Vec::new() };

    assert!(assembler.parse_label(&mut lexer).is_ok());

    let mut lexer = Token::lexer(&sublevel_source).spanned().peekable();
    assert!(assembler.parse_label(&mut lexer).is_ok());
    assert_eq!(assembler.labels.len(), 1);

    let sublabels = &assembler.labels.last().unwrap().sublabels;
    assert_eq!(sublabels.len(), 1);

    assert_eq!(
        sublabels[0],
        SubLabel {
            name: "sublabel".to_string(),
            span: 0..9,
        }
    );
}
