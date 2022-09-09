use std::collections::HashMap;

use codespan_reporting::files::SimpleFiles;

use crate::{
    ast::{Include, Location},
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

    let parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let program_result = parser_context.parse_program();

    assert!(program_result.is_ok());
    assert!(program_result.unwrap().is_empty());
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

    let parser_context = ParserContext::new(
        source_name.clone(),
        &source_1,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let program_result_1 = parser_context.parse_program();

    let parser_context = ParserContext::new(
        source_name.clone(),
        &source_2,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let program_result_2 = parser_context.parse_program();

    assert!(program_result_1.is_ok());
    assert!(program_result_2.is_ok());
}
