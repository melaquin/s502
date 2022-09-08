use std::collections::HashMap;

use codespan_reporting::files::SimpleFiles;
use logos::Logos;

use crate::{
    ast::{Include, Location},
    lexer::Token,
    parser::ParserContext,
};

#[test]
fn empty_program() {
    let source = "".to_string();
    let source_name = "empty_program".to_string();
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

    let parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let program_result = parser_context.parse_program();

    assert!(program_result.is_ok());
    assert!(program_result.unwrap().lines.is_empty());
}

#[test]
fn two_programs() {
    let source_1 = "".to_string();
    let source_2 = "".to_string();
    let source_name = "empty_program".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();
    let lexer = Token::lexer(&source_1).spanned().peekable();

    let parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let program_result_1 = parser_context.parse_program();

    let lexer = Token::lexer(&source_2).spanned().peekable();

    let parser_context = ParserContext::new(
        source_name.clone(),
        lexer,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let program_result_2 = parser_context.parse_program();

    assert!(program_result_1.is_ok());
    assert!(program_result_2.is_ok());
}
