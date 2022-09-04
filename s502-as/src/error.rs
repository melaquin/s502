use std::ops::Range;

/// Errors encountered while parsing the assembly.
pub struct AssemblerError {
    pub span: Range<usize>,
    pub message: String,
    pub note: Option<String>,
}
