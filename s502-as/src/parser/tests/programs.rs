use std::collections::HashMap;

use codespan_reporting::files::SimpleFiles;

use crate::ast::{Include, Location};

use super::super::parse_program;

#[test]
fn empty_program() {
    let source = "".to_string();
    let source_name = "empty_program".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..0,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let (_, program_result) = parse_program(
        source_name,
        source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

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
            span: 0..0,
            name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let (id_1, program_result_1) = parse_program(
        source_name,
        source_1,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let file_name = "empty_program".to_string();
    let (id_2, program_result_2) = parse_program(
        file_name,
        source_2,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    assert!(program_result_1.is_ok());
    assert!(program_result_2.is_ok());
    assert!(id_1 != id_2)
}
