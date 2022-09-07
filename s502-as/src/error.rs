use crate::ast::Location;

/// Errors encountered while parsing the assembly.
#[derive(Debug, PartialEq)]
pub struct AssemblerError {
    /// The main error message.
    pub message: String,
    /// Notes that describe the error and point to code excerpts.
    /// The first will be the primary label and the rest secondary.
    pub labels: Vec<(Location, Option<String>)>,
    /// Help message if necessary.
    pub help: Option<String>,
}
