use std::ops::Range;

/// Errors encountered while parsing the assembly.
#[derive(Debug, PartialEq)]
pub struct AssemblerError {
    pub span: Range<usize>,
    pub message: String,
    pub note: Option<String>,
}
