use codespan_reporting::files::SimpleFiles;

use super::super::parse_program;

#[test]
fn empty_program() {
    let source = "".to_string();
    let mut files = SimpleFiles::<String, String>::new();

    let (_, program_result) = parse_program("empty_program".to_string(), source, &mut files);

    assert!(program_result.is_ok());
    assert!(program_result.unwrap().lines.is_empty());
}

#[test]
fn two_programs() {
    let source_1 = "".to_string();
    let source_2 = "".to_string();
    let mut files = SimpleFiles::<String, String>::new();

    let (id_1, program_result_1) = parse_program("empty_program".to_string(), source_1, &mut files);
    let (id_2, program_result_2) = parse_program("empty_program".to_string(), source_2, &mut files);

    assert!(program_result_1.is_ok());
    assert!(program_result_2.is_ok());
    assert!(id_1 != id_2)
}