use std::collections::HashMap;

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

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

pub fn report_errors(
    errors: Vec<AssemblerError>,
    id_table: &HashMap<String, usize>,
    files: &SimpleFiles<String, String>,
) {
    let stderr_writer = StandardStream::stderr(ColorChoice::Always);
    let codespan_config = Config::default();

    for error in errors {
        // Create labels from notes.
        let mut labels = vec![];
        let mut notes = error.labels.into_iter();

        // The first will be primary.
        if let Some((loc, message)) = notes.next() {
            let mut label = Label::primary(id_table[&loc.file_name], loc.span);
            if let Some(message) = message {
                label = label.with_message(message);
            }
            labels.push(label);
        }

        // And the rest secondary.
        notes.for_each(|(loc, message)| {
            let mut label = Label::secondary(id_table[&loc.file_name], loc.span);
            if let Some(message) = message {
                label = label.with_message(message);
            }
            labels.push(label);
        });

        // Then create diagnostic message from it.
        let diagnostic = Diagnostic::error()
            .with_message(error.message)
            .with_labels(labels);

        let _ = emit(
            &mut stderr_writer.lock(),
            &codespan_config,
            files,
            &diagnostic,
        );

        // And create a second help diagnostic if one was given.
        if let Some(note) = error.help {
            let diagnostic = Diagnostic::help().with_message(note);

            let _ = emit(
                &mut stderr_writer.lock(),
                &codespan_config,
                files,
                &diagnostic,
            );
        }
    }
}
