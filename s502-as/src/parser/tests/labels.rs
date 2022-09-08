use std::collections::HashMap;

use crate::{ast::*, error::AssemblerError, parser::ParserContext};
use codespan_reporting::files::SimpleFiles;
use logos::Logos;

use crate::lexer::Token;

#[test]
fn plain() {
    let source = "mylabel".to_string();
    let source_name = "plain test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();
    let lexer = Token::lexer(&source).spanned().peekable();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let label = parser_context.parse_label();
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
    let source_name = "global test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();
    let lexer = Token::lexer(&source).spanned().peekable();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let label = parser_context.parse_label();
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
    let source_name = "sublabel test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();
    let lexer = Token::lexer(&source).spanned().peekable();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let sublabel = parser_context.parse_label();
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
    let source_name = "no ident after global test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();
    let lexer = Token::lexer(&source).spanned().peekable();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let parse_result = parser_context.parse_label();
    assert!(parse_result.is_err());

    assert_eq!(
        parse_result.unwrap_err(),
        AssemblerError {
            message: "Unexpected token `adc`".to_string(),
            labels: vec![(
                Location {
                    span: 1..4,
                    name: "no ident after global test".to_string()
                },
                Some("Expected a label".to_string())
            )],
            help: None,
        }
    );
}

#[test]
fn no_ident_after_period() {
    let source = ".dfb".to_string();
    let source_name = "no ident after period test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();
    let lexer = Token::lexer(&source).spanned().peekable();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let parse_result = parser_context.parse_label();
    assert!(parse_result.is_err());

    assert_eq!(
        parse_result.unwrap_err(),
        AssemblerError {
            message: "Unexpected token `dfb`".to_string(),
            labels: vec![(
                Location {
                    span: 1..4,
                    name: "no ident after period test".to_string()
                },
                Some("Expected a label".to_string())
            )],
            help: None,
        }
    );
}
