//! Parser that builds a syntax tree that represents the input assembly.

use std::iter::Peekable;

use super::lexer::Token;
use crate::{ast::*, error::*};
use codespan_reporting::files::SimpleFiles;
use logos::{Logos, SpannedIter};

type SpannedLexer<'source> = Peekable<SpannedIter<'source, Token>>;

pub struct Assembler {
    labels: Vec<Label>,
}

impl Assembler {
    pub fn parse_file(
        name: String,
        source: String,
        files: &mut SimpleFiles<String, String>,
    ) -> (usize, Vec<AssemblerError>) {
        let mut assembler = Self {
            labels: Vec::with_capacity(16),
        };
        let mut lexer = Token::lexer(&source).spanned().peekable();

        let mut errors = Vec::with_capacity(8);

        'line_loop: loop {
            match assembler.parse_line(&mut lexer) {
                Ok(false) => break,
                Ok(true) => {}
                Err(error) => {
                    errors.push(error);
                    loop {
                        match lexer.peek() {
                            None => break 'line_loop,
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
            }
        }

        (files.add(name, source), errors)
    }

    fn parse_line(&mut self, lexer: &mut SpannedLexer) -> Result<bool, AssemblerError> {
        self.parse_label(lexer).map(|_| false)
        // Ok(false)
    }

    fn parse_label(&mut self, lexer: &mut SpannedLexer) -> Result<(), AssemblerError> {
        // Only want to parse a label if we see one of these four tokens, otherwise
        // there is no label to parse.
        let (mut token, mut span) = match lexer.next_if(|(token, _)| {
            matches!(token, Token::Global)
                // || matches!(token, Token::Object)
                || matches!(token, Token::Period)
                || matches!(token, Token::Ident { .. })
        }) {
            Some(next) => next,
            None => return Ok(()),
        };

        let (vis, sublabel) = match token {
            Token::Global => {
                let attributes = (Some(span.clone()), None);
                (token, span) = lexer.next().ok_or(AssemblerError {
                    span,
                    message: "unexpected end of file, expected a label".to_string(),
                    note: None,
                })?;
                attributes
            }
            Token::Period => {
                let attributes = (None, Some(span.clone()));
                (token, span) = lexer.next().ok_or(AssemblerError {
                    span,
                    message: "unexpected end of file, expected a label".to_string(),
                    note: None,
                })?;
                attributes
            }
            _ => (None, None),
        };

        let identifier = match token {
            Token::Ident(ident) => ident,
            _ => {
                return Err(AssemblerError {
                    span,
                    message: format!("unexpected token {}", token),
                    note: Some("expected a label".to_string()),
                });
            }
        };

        if let Some(period_span) = sublabel {
            self.labels
                .last_mut()
                .ok_or(AssemblerError {
                    span: period_span.start..span.end,
                    message: format!(
                        "No top level label has been created for sublabel to go under."
                    ),
                    note: None,
                })?
                .sublabels
                .push(SubLabel {
                    name: identifier,
                    span,
                });
        } else {
            self.labels.push(Label {
                name: identifier,
                span,
                visibility: if vis.is_some() {
                    Visibility::Global
                } else {
                    Visibility::Object
                },
                sublabels: vec![],
            });
        }
        Ok(())
    }
}
