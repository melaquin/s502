//! Parser that builds a syntax tree that represents the input assembly.

#[cfg(test)]
mod tests;

use std::iter::Peekable;

use super::lexer::Token;
use crate::{ast::*, error::*};
use codespan_reporting::files::SimpleFiles;
use logos::{Logos, SpannedIter};

type SpannedLexer<'source> = Peekable<SpannedIter<'source, Token>>;

pub fn parse_program(
    name: String,
    source: String,
    files: &mut SimpleFiles<String, String>,
) -> (usize, Result<Program, Vec<AssemblerError>>) {
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let mut errors = Vec::with_capacity(8);
    let mut program = Program {
        lines: Vec::with_capacity(256),
    };

    // Assemble the program one line at a time. Stop when there are no more tokens.
    while lexer.peek().is_some() {
        match parse_line(&mut lexer) {
            // The line was empty, go on to the next one.
            Ok(None) => continue,
            Ok(Some(line)) => program.lines.push(line),
            Err(error) => errors.push(error),
        }
    }

    (
        files.add(name, source),
        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(program)
        },
    )
}

fn parse_line(lexer: &mut SpannedLexer) -> Result<Option<Line>, AssemblerError> {
    // Label appears first.
    let label = parse_label(lexer)?;

    // Skip to and past the end of the line.
    loop {
        match lexer.peek() {
            None => break,
            Some((Token::Eol, _)) => {
                lexer.next();
                break;
            }
            _ => {
                lexer.next();
            }
        }
    }

    if let None = label {
        Ok(None)
    } else {
        Ok(Some(Line { label }))
    }
}

/// Parses an optional label at the beginning of a line.
/// Matches the syntax (GLOBAL | PERIOD)? ID.
fn parse_label(lexer: &mut SpannedLexer) -> Result<Option<Label>, AssemblerError> {
    // Only want to parse a label if we see one of these four tokens, otherwise
    // there is no label to parse.
    let (mut token, mut span) = match lexer.next_if(|(token, _)| {
        matches!(token, Token::Global)
                // || matches!(token, Token::Object)
                || matches!(token, Token::Period)
                || matches!(token, Token::Ident { .. })
    }) {
        Some(next) => next,
        None => return Ok(None),
    };

    // A label  may start with either `!`, indicating that  the label should be exported,
    // or `.` indicating that it is a sublabel of the most recent top-level label.
    let (vis, sublabel) = match token {
        Token::Global => {
            let attributes = (Some(span.clone()), None);
            (token, span) = lexer.next().ok_or(AssemblerError {
                span,
                message: "Unexpected end of file, expected a label".to_string(),
                note: None,
            })?;
            attributes
        }
        Token::Period => {
            let attributes = (None, Some(span.clone()));
            (token, span) = lexer.next().ok_or(AssemblerError {
                span,
                message: "Unexpected end of file, expected a label".to_string(),
                note: None,
            })?;
            attributes
        }
        _ => (None, None),
    };

    // Expect Ident to follow the above attribute.
    let identifier = match token {
        Token::Ident(ident) => ident,
        _ => {
            return Err(AssemblerError {
                span,
                message: format!("Unexpected token {}", token),
                note: Some("Expected a label".to_string()),
            });
        }
    };

    Ok(Some(if let Some(period_span) = sublabel {
        Label::Sub(SubLabel {
            name: identifier,
            span: period_span.start..span.end,
        })
        // TODO keeping this for reference later
        // // Check that there is actually a top-level label to add this to.
        // self.labels
        //     .last_mut()
        //     .ok_or(AssemblerError {
        //         span: period_span.start..span.end,
        //         message: format!("Sublabel without top-level label"),
        //         note: Some(format!(
        //             "No top level label has been created for `{}` to go under",
        //             identifier
        //         )),
        //     })?
        //     .sublabels
        //     .push(SubLabel {
        //         name: identifier,
        //         span: period_span.start..span.end,
        //     });
    } else {
        Label::Top(TopLabel {
            name: identifier,
            span,
            visibility: if vis.is_some() {
                Visibility::Global
            } else {
                Visibility::Object
            },
            sublabels: vec![],
        })
    }))
}
