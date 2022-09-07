//! Parser that builds a syntax tree that represents the input assembly.

#[cfg(test)]
mod tests;

use std::{collections::HashMap, fs, iter::Peekable, ops::Range};

use super::lexer::Token;
use crate::{ast::*, error::*};
use codespan_reporting::files::SimpleFiles;
use logos::{Logos, SpannedIter};

type SpannedLexer<'source> = Peekable<SpannedIter<'source, Token>>;

pub fn parse_program(
    file_name: String,
    source: String,
    files: &mut SimpleFiles<String, String>,
    include_path: &mut Vec<Include>,
    id_table: &mut HashMap<String, usize>,
) -> (usize, Result<Program, Vec<AssemblerError>>) {
    let mut lexer = Token::lexer(&source).spanned().peekable();

    let mut errors = Vec::with_capacity(8);
    let mut program = Program {
        lines: Vec::with_capacity(256),
    };

    // Assemble the program one line at a time. Stop when there are no more tokens.
    while lexer.peek().is_some() {
        // Handle includes specially, they cannot start with a label.
        if let Some((Token::Inl, _)) = lexer.peek() {
            let (_, inl_span) = lexer.next().unwrap();
            let (included_id, parse_result) =
                handle_include(&mut lexer, inl_span, files, include_path, id_table);

            // Push included file's ID before its code so the next stage knows what file it's in.
            if let Some(id) = included_id {
                program.lines.push(Line::PushInclude(id));
            }
            match parse_result {
                Ok(mut included_program) => program.lines.append(&mut included_program.lines),
                Err(mut included_errors) => errors.append(&mut included_errors),
            }
            // And the next stage needs to know when the included file ends.
            if included_id.is_some() {
                program.lines.push(Line::PopInclude);
            }
            skip_to_eol(&mut lexer);
            continue;
        }

        match parse_line(&mut lexer, &file_name) {
            // The line was empty, go on to the next one.
            Ok(None) => continue,
            Ok(Some(line)) => program.lines.push(Line::Instruction(line)),
            Err(error) => errors.push(error),
        }
    }

    (
        // Add it to the files only if it hasn't been included before to prevent duplicates,
        // otherwise return hte existing ID.
        if id_table.contains_key(&file_name) {
            id_table[&file_name]
        } else {
            files.add(file_name, source)
        },
        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(program)
        },
    )
}

/// parse the inl directive, preventing circular inclusion and parsing the included file.
fn handle_include(
    lexer: &mut SpannedLexer,
    inl_span: Range<usize>,
    files: &mut SimpleFiles<String, String>,
    include_stack: &mut Vec<Include>,
    id_table: &mut HashMap<String, usize>,
) -> (Option<usize>, Result<Program, Vec<AssemblerError>>) {
    // Expect to see a string literal after the inl directive.
    let (file_name, name_span) = match lexer.next() {
        None => {
            return (
                None,
                Err(vec![AssemblerError {
                    message: "Unexpected end of file, expected a string".to_string(),
                    labels: vec![(
                        Location {
                            span: inl_span,
                            // SAFETY Last can be unwrapped because the stack has at least one member
                            // before parse_program and subsequently handle_include are called.
                            name: include_stack.last().unwrap().included.clone(),
                        },
                        None,
                    )],
                    help: None,
                }]),
            );
        }
        Some((Token::Literal(Literal::String(file_name)), name_span)) => (file_name, name_span),
        Some((token, span)) => {
            return (
                None,
                Err(vec![AssemblerError {
                    message: format!("Unexpected token {}", token),
                    labels: vec![(
                        Location {
                            span,
                            name: include_stack.last().unwrap().included.clone(),
                        },
                        Some("Expected a string".to_string()),
                    )],
                    help: None,
                }]),
            );
        }
    };

    // Check if the file has already been parsed. If it has then it would loop back
    // to this file and cause infinite recursion.
    if let Some(include) = include_stack
        .iter()
        .find(|include| include.included == file_name)
    {
        return (
            None,
            Err(vec![AssemblerError {
                message: "Recursive include found".to_string(),
                labels: vec![
                    (
                        Location {
                            span: inl_span.start..name_span.end,
                            name: include_stack.last().unwrap().included.clone(),
                        },
                        Some(format!("Could not include {}", file_name)),
                    ),
                    (
                        Location {
                            span: include.loc.span.clone(),
                            name: include.loc.name.clone(),
                        },
                        Some(if include == include_stack.first().unwrap() {
                            "Given in assembler invocatiion".to_string()
                        } else {
                            "Already included here".to_string()
                        }),
                    ),
                ],
                help: Some(format!("Labels can be referenced before they're defined,\nso including `{}` here may not be necessary", file_name)),
            }]),
        );
    }

    // No recursion, read the source.
    let source = match fs::read_to_string(&file_name) {
        Err(error) => {
            return (
                None,
                Err(vec![AssemblerError {
                    message: format!("Could not read {}: {}", file_name, error),
                    labels: vec![(
                        Location {
                            span: name_span,
                            name: include_stack.last().unwrap().included.clone(),
                        },
                        None,
                    )],
                    help: None,
                }]),
            );
        }
        Ok(source) => source,
    };

    // Add it to the stack so it can be found in nested calls to handle_include.
    include_stack.push(Include {
        included: file_name.clone(),
        loc: Location {
            span: inl_span.start..name_span.end,
            name: include_stack.last().unwrap().included.clone(),
        },
    });

    let (file_id, parse_result) =
        parse_program(file_name.clone(), source, files, include_stack, id_table);

    // Remove it from the stack so it may be included again later.
    let _ = include_stack.pop();
    // And add it to the map if it hasn't been included before.
    if !id_table.contains_key(&file_name) {
        id_table.insert(file_name, file_id);
    }
    (Some(file_id), parse_result)
}

/// Skip past the end of the line after it is done being parsed.
/// This makes a dedicated line comment character unnecessary
/// just like the good ol' days.
fn skip_to_eol(lexer: &mut SpannedLexer) {
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
}

/// Parse a line including the label, mnemonic, and operand.
fn parse_line(
    lexer: &mut SpannedLexer,
    file_name: &String,
) -> Result<Option<Instruction>, AssemblerError> {
    // Label appears first.
    let label = parse_label(lexer, file_name)?;

    skip_to_eol(lexer);

    if let None = label {
        Ok(None)
    } else {
        Ok(Some(Instruction { label }))
    }
}

/// Parses an optional label at the beginning of a line.
/// Matches the syntax (GLOBAL | PERIOD)? ID.
fn parse_label(
    lexer: &mut SpannedLexer,
    file_name: &String,
) -> Result<Option<Label>, AssemblerError> {
    // Only want to parse a label if we see one of these four tokens, otherwise
    // there is no label to parse.
    let (mut token, mut span) = match lexer.next_if(|(token, _)| {
        matches!(token, Token::Global)
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
                message: "Unexpected end of file, expected a label".to_string(),
                labels: vec![(
                    Location {
                        span,
                        name: file_name.clone(),
                    },
                    None,
                )],
                help: None,
            })?;
            attributes
        }
        Token::Period => {
            let attributes = (None, Some(span.clone()));
            (token, span) = lexer.next().ok_or(AssemblerError {
                message: "Unexpected end of file, expected a label".to_string(),
                labels: vec![(
                    Location {
                        span,
                        name: file_name.clone(),
                    },
                    None,
                )],
                help: None,
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
                message: format!("Unexpected token {}", token),
                labels: vec![(
                    Location {
                        span,
                        name: file_name.clone(),
                    },
                    Some("Expected a label".to_string()),
                )],
                help: None,
            });
        }
    };

    Ok(Some(if let Some(period_span) = sublabel {
        Label::Sub(SubLabel {
            name: identifier,
            span: period_span.start..span.end,
        })
        // TODO keeping this for reference later when the next stage deals with this.
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

// fn parse_directive(lexer: &mut SpannedLexer) -> Result<Option<Directive>, AssemblerError> {
//     let (mut token, mut span) = match lexer.next_if(|(token, _)| {
//         matches!(token, Token::Dfb)
//             || matches!(token, Token::Dfw)
//             || matches!(token, Token::Equ)
//             || matches!(token, Token::Inl)
//             || matches!(token, Token::Hlt)
//             || matches!(token, Token::Org)
//             || matches!(token, Token::Sct)
//     }) {
//         Some(next) => next,
//         None => return Ok(None),
//     };

//     Ok(None)
// }
