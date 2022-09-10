use std::collections::HashMap;

use codespan_reporting::files::SimpleFiles;

use crate::{ast::*, parser::ParserContext};

#[test]
fn empty_line() {
    let source = "".to_string();
    let source_name = "empty line test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let line = parser_context.parse_line();
    assert!(line.is_ok());
    assert!(parser_context.program.is_empty());
}

#[test]
fn label_line() {
    let source = "mylabel".to_string();
    let source_name = "label line test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let line = parser_context.parse_line();
    assert!(line.is_ok());
    assert_eq!(
        parser_context.program,
        vec![Action::Label(Spanned::new((
            Label::Top(TopLabel {
                name: "mylabel".to_string(),
                visibility: Visibility::Object,
                sublabels: vec![],
            }),
            0..7
        )))]
    );
}
