use std::ops::Range;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::{self, Config};

/// Errors encountered while parsing the assembly.
pub struct AssemblerError {
    pub span: Range<usize>,
    pub message: String,
    pub note: Option<String>,
}
// #[derive(Debug)]
// pub enum AssemblerError {
//     /// An illegal lexeme was found.
//     Lexeme(Range<usize>, String),
// }

/// Handler for `AssemblerError`s that will keep and print them.
pub struct ErrorHandler<'name, 'source> {
    /// The files that are being parsed.
    pub files: SimpleFiles<&'name str, &'source String>,
    /// Messages detailing errors and warnings that were found while parsing.
    pub errors: Vec<Diagnostic<usize>>,
    /// Standard error writer for reporting.
    writer: StandardStream,
    /// Terminal Writer cnfiguration.
    config: Config,
}

impl<'name, 'source> ErrorHandler<'name, 'source> {
    /// Create a new error handler from filenames and their corresponding source code.
    ///
    /// Returns an error handler and file IDs corresponding to the given source code.
    pub fn new<S>(sources: S) -> (Self, Vec<usize>)
    where
        S: Iterator<Item = (&'name str, &'source String)>,
    {
        let mut files = SimpleFiles::new();
        let mut ids = vec![];
        sources.for_each(|(name, source)| {
            ids.push(files.add(name, source));
        });

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = Config::default();

        (
            Self {
                files,
                errors: vec![],
                writer,
                config,
            },
            ids,
        )
    }

    pub fn push_file(&mut self, name: &'name str, source: &'source String) -> usize {
        self.files.add(name, source)
    }

    /// Report an error that is not tied to any source file. This is to be used
    /// independently of any `ErrorHandler` instance.
    pub fn report_lone(message: String) {
        let diagnostic = Diagnostic::error().with_message(message);

        let files = SimpleFiles::<&'static str, &'static str>::new();
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = Config::default();
        let _ = term::emit(&mut writer.lock(), &config, &files, &diagnostic);
    }

    // pub fn err(&mut self, error: AssemblerError) {
    //     self.errors.push(
    //         Diagnostic::error()
    //             .with_message(error.message)
    //             .with_labels(vec![Label::primary(error.file_id, error.span)]),
    //     );
    // }

    /// Immediately report a diagnostic message.
    fn report_single(&self, diagnostic: Diagnostic<usize>) {
        let _ = term::emit(
            &mut self.writer.lock(),
            &self.config,
            &self.files,
            &diagnostic,
        );
    }

    // /// Report all errors that were found while parsing.
    // pub fn report_parse_messages(
    //     &self,
    //     failures: Vec<(usize, ParseError<usize, Token, AssemblerError>)>,
    // ) {
    //     failures
    //         .into_iter()
    //         .for_each(|(file_id, error)| match error {
    //             ParseError::UnrecognizedToken {
    //                 token: (start, token, end),
    //                 expected,
    //             } => {
    //                 let expected_tokens = expected.join(" or ");

    //                 let diagnostic = Diagnostic::error()
    //                     .with_message(format!("unexpected {}", token))
    //                     .with_labels(vec![Label::primary(file_id, start..end)])
    //                     .with_notes(vec![format!("expected {}", expected_tokens)]);

    //                 self.report_single(diagnostic);
    //             }
    //             ParseError::UnrecognizedEOF { location, expected } => {
    //                 let expected = expected.join(" or ");

    //                 let diagnostic = Diagnostic::error()
    //                     .with_message(format!("unexpected end of file"))
    //                     .with_labels(vec![Label::primary(file_id, location..location)])
    //                     .with_notes(vec![format!("expected {}", expected)]);

    //                 self.report_single(diagnostic);
    //             }
    //             ParseError::InvalidToken { location } => {
    //                 println!("invalid token at {location}");
    //             }
    //             ParseError::ExtraToken { token } => {
    //                 println!("extra token `{}`", token.1);
    //             }
    //             ParseError::User { error } => match error {
    //                 AssemblerError::Lexeme(span, lexeme) => {
    //                     let diagnostic = Diagnostic::error()
    //                         .with_message(format!("unrecognized `{lexeme}`"))
    //                         .with_labels(vec![Label::primary(file_id, span)]);

    //                     self.report_single(diagnostic);
    //                 }
    //             },
    //         });
    // }
}
