use std::collections::HashMap;

use codespan_reporting::files::SimpleFiles;

use crate::{ast::*, parser::ParserContext};

#[test]
fn safe_include() {
    let source = "inl \"test_inputs/safe_include.65a\"".to_string();
    let source_name = "include labels test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
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

    let included_result = parser_context.parse_instruction();

    assert!(included_result.is_ok());
    assert_eq!(
        included_result.unwrap(),
        Some(Spanned::new((
            Instruction {
                mnemonic: Spanned::new((Mnemonic::Inl, 0..3)),
                operand: Some(Spanned::new((
                    Operand {
                        mode: OperandMode::Immediate,
                        modifier: None,
                        value: Spanned::new((
                            Value::Include((
                                String::from("test_inputs/safe_include.65a"),
                                vec![
                                    Action::LineStart(0),
                                    Action::Label(Spanned::new((
                                        Label::Top(TopLabel {
                                            name: String::from("mylabel"),
                                            visibility: Visibility::Object
                                        }),
                                        0..7
                                    ))),
                                    Action::LineEnd(7)
                                ]
                            )),
                            4..34
                        ))
                    },
                    4..34
                )))
            },
            0..34
        )))
    );
}

#[test]
fn bad_string_include() {
    let source = "inl <\"test_inputs/safe_include.65a\"".to_string();
    let source_name = "include labels test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
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

    let included_result = parser_context.parse_instruction();

    assert!(included_result.is_err());
    let error_labels = included_result.unwrap_err().labels;
    assert_eq!(
        error_labels.get(0).unwrap().1,
        Some(String::from("Expected an unmodified string literal"))
    );
}

#[test]
fn recursive_include() {
    let source = "inl \"test_inputs/recursive_include.65a\"".to_string();
    let source_name = "include labels test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
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

    let (included_file_id, included_result) =
        parser_context.handle_include(String::from("test_inputs/recursive_include.65a"), 4..34);

    assert!(included_file_id.is_some());
    assert!(included_result.is_err());
    let errors = included_result.unwrap_err();
    let error = errors.get(0).unwrap();
    assert_eq!(error.message, String::from("Recursive include found"));
}

#[test]
fn bad_extension_include() {
    let source = "inl \"test_inputs/bad_ext_include.txt\"".to_string();
    let source_name = "include labels test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
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

    let included_result = parser_context.parse_instruction();

    assert!(included_result.is_err());
    assert_eq!(
        included_result.unwrap_err().labels.get(0).unwrap().1,
        Some(String::from("File extension is expected to be `65a`"))
    );
}

#[test]
fn file_dne_include() {
    let source = "inl \"test_inputs/file_dne_include.65a\"".to_string();
    let source_name = "include labels test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
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

    let (included_file_id, included_result) =
        parser_context.handle_include(String::from("test_inputs/file_dne_include.65a"), 4..33);

    assert!(included_file_id.is_some());
    assert!(included_result.is_err());
    let errors = included_result.unwrap_err();
    let error = errors.get(0).unwrap();
    assert!(error.message.contains("No such file or directory"));
}
