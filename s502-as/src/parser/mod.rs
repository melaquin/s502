//! Parser that builds a syntax tree that represents the input assembly.

pub mod lexer;
#[cfg(test)]
mod tests;

use std::{collections::HashMap, convert::TryFrom, fs, iter::Peekable, ops::Range, path::Path};

use codespan_reporting::files::SimpleFiles;
use logos::{Logos, SpannedIter};

use crate::{ast::*, error::*};
use lexer::{Literal, Token};

pub type SpannedLexer<'source> = Peekable<SpannedIter<'source, Token>>;

pub struct ParserContext<'source, 'context> {
    file_name: String,
    lexer: SpannedLexer<'source>,
    files: &'context mut SimpleFiles<String, String>,
    include_stack: &'context mut Vec<Include>,
    id_table: &'context mut HashMap<String, usize>,
    program: Program,
    errors: Vec<AssemblerError>,
}

impl<'source, 'context> ParserContext<'source, 'context> {
    pub fn new(
        file_name: String,
        source: &'source String,
        files: &'context mut SimpleFiles<String, String>,
        include_stack: &'context mut Vec<Include>,
        id_table: &'context mut HashMap<String, usize>,
    ) -> Self {
        Self {
            file_name,
            lexer: Token::lexer(source).spanned().peekable(),
            files,
            include_stack,
            id_table,
            program: Program::with_capacity(256),
            errors: Vec::with_capacity(8),
        }
    }

    pub fn parse_program(mut self) -> Result<Program, Vec<AssemblerError>> {
        while self.lexer.peek().is_some() {
            match self.parse_line() {
                // The line was empty, go on to the next one.
                Ok(()) => continue,
                Err(error) => self.errors.push(error),
            }
        }

        if self.errors.len() > 0 {
            Err(self.errors)
        } else {
            Ok(self.program)
        }
    }

    /// Read and parse an included file, preventing circular inclusion.
    /// Returns the included file ID (if reading was successful) and
    /// the result of parsing the included file.
    fn handle_include(
        &mut self,
        to_include_name: String,
        to_include_span: Range<usize>,
    ) -> (Option<usize>, Result<Program, Vec<AssemblerError>>) {
        // Check if the file has already been parsed. If it has then it would loop back
        // to this file and cause infinite recursion.
        if let Some(include) = self
            .include_stack
            .iter()
            .find(|include| include.included == to_include_name)
        {
            return (
                None,
                Err(vec![AssemblerError {
                    message: "Recursive include found".to_string(),
                    labels: vec![
                        (
                            Location {
                                span: to_include_span,
                                // SAFETY Last can be unwrapped because the stack has at least one member
                                // before parse_program and subsequently handle_include are called.
                                name: self.include_stack.last().unwrap().included.clone(),
                            },
                            Some(format!("Could not include {}", to_include_name)),
                        ),
                        (
                            Location {
                                span: include.loc.span.clone(),
                                name: include.loc.name.clone(),
                            },
                            Some(if include == self.include_stack.first().unwrap() {
                                "Given in assembler invocatiion".to_string()
                            } else {
                                "Already included here".to_string()
                            }),
                        ),
                    ],
                    help: Some(format!("Labels can be referenced before they're defined,\nso including `{}` here may not be necessary", to_include_name)),
                }])
            );
        }

        // No recursion, read the source.
        let included_source = match fs::read_to_string(&to_include_name) {
            Err(error) => {
                return (
                    None,
                    Err(vec![AssemblerError {
                        message: format!("Could not read {}: {}", to_include_name, error),
                        labels: vec![(
                            Location {
                                span: to_include_span,
                                name: self.include_stack.last().unwrap().included.clone(),
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
        self.include_stack.push(Include {
            included: to_include_name.clone(),
            loc: Location {
                span: to_include_span.start..to_include_span.end,
                name: self.include_stack.last().unwrap().included.clone(),
            },
        });

        let included_context = ParserContext::new(
            to_include_name.clone(),
            &included_source,
            self.files,
            self.include_stack,
            self.id_table,
        );

        let parse_result = included_context.parse_program();
        let included_file_id = if self.id_table.contains_key(&self.file_name) {
            self.id_table[&to_include_name]
        } else {
            self.files.add(to_include_name.clone(), included_source)
        };

        // Remove it from the stack so it may be included again later.
        let _ = self.include_stack.pop();
        // And add it to the map if it hasn't been included before.
        if !self.id_table.contains_key(&to_include_name) {
            self.id_table.insert(to_include_name, included_file_id);
        }
        (Some(included_file_id), parse_result)
    }

    /// Skip past the end of the line after it is done being parsed.
    /// This makes a dedicated line comment character unnecessary
    /// just like the good ol' days.
    fn skip_to_eol(&mut self) {
        loop {
            match self.lexer.peek() {
                None => break,
                Some((Token::Eol, _)) => {
                    self.lexer.next();
                    break;
                }
                _ => {
                    self.lexer.next();
                }
            }
        }
    }

    /// Parse a line including the label, mnemonic, and operand.
    fn parse_line(&mut self) -> Result<(), AssemblerError> {
        // Label appears first.
        let label = self.parse_label()?;
        // Add it to the program right away because if the action is an include, then
        // parse_instruction will put the included file in the program before returning.
        if let Some(label) = label {
            self.program.push(Item::Label(label));
        }

        let instruction = self.parse_instruction()?;

        if let Some(instruction) = instruction {
            self.program.push(Item::Instruction(instruction));
        }

        self.skip_to_eol();

        Ok(())
    }

    /// Parses an optional label at the beginning of a line.
    /// Matches the syntax (GLOBAL | PERIOD)? ID.
    fn parse_label(&mut self) -> Result<Option<Label>, AssemblerError> {
        // Only want to parse a label if we see one of these four tokens, otherwise
        // there is no label to parse.
        let (mut token, mut span) = match self.lexer.next_if(|(token, _)| {
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
                (token, span) = self.lexer.next().ok_or(AssemblerError {
                    message: "Unexpected end of file, expected a label".to_string(),
                    labels: vec![(
                        Location {
                            span,
                            name: self.file_name.clone(),
                        },
                        None,
                    )],
                    help: None,
                })?;
                attributes
            }
            Token::Period => {
                let attributes = (None, Some(span.clone()));
                (token, span) = self.lexer.next().ok_or(AssemblerError {
                    message: "Unexpected end of file, expected a label".to_string(),
                    labels: vec![(
                        Location {
                            span,
                            name: self.file_name.clone(),
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
                            name: self.file_name.clone(),
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
            // TODO keeping this for reference later when the generation stage deals with this.
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

    fn parse_instruction(&mut self) -> Result<Option<Instruction>, AssemblerError> {
        let parsed_mnemonic = self.parse_mnemonic();
        // If mnemonic is implied then don't try to parse what follows
        // an operand, return and let parse_line skip it as a comment.
        if let Some(ref mnemonic) = parsed_mnemonic {
            if mnemonic.0.is_implied() {
                return Ok(Some(Instruction {
                    mnemonic: Spanned::new(mnemonic.clone()),
                    operand: None,
                }));
            }
        }
        let parsed_operand = self.parse_operand()?;

        // Handle the include directive here so the nested parser can give its Items to the generation stage.
        // Other directives will be handled in that stage.
        if let Some((ref mnemonic, ref mnemonic_span)) = parsed_mnemonic {
            if let Mnemonic::Inl = mnemonic {
                if let Some((Operand::Literal(Literal::String(to_include_name)), to_include_span)) =
                    parsed_operand
                {
                    // Expect the included file extension to be 65a.
                    return match Path::new(&to_include_name).extension() {
                        Some(extension) if extension == "65a" => {
                            let (included_id, parse_result) = self.handle_include(
                                to_include_name,
                                mnemonic_span.start..to_include_span.end,
                            );
                            // Push included file's ID before its code so the generation stage knows what file it's in.
                            if let Some(id) = included_id {
                                self.program.push(Item::PushInclude(id));
                            }
                            match parse_result {
                                Ok(mut included_program) => {
                                    self.program.append(&mut included_program)
                                }
                                Err(mut included_errors) => {
                                    self.errors.append(&mut included_errors)
                                }
                            }
                            // And the generation stage needs to know when the included file ends.
                            if included_id.is_some() {
                                self.program.push(Item::PopInclude);
                            }
                            Ok(None)
                        }
                        // Wrong or no extension, error.
                        _ => Err(AssemblerError {
                            message: format!("Could not include {}", to_include_name),
                            labels: vec![(
                                Location {
                                    span: to_include_span,
                                    name: self.file_name.clone(),
                                },
                                Some("File extension is expected to be `65a`".to_string()),
                            )],
                            help: None,
                        }),
                    };
                }
            }
        }

        // If we parsed a mnemonic then create an Instruction
        // with optional operand, otherwise None.
        if let Some(mnemonic) = parsed_mnemonic {
            Ok(Some(Instruction {
                mnemonic: Spanned::new(mnemonic),
                operand: parsed_operand.map(Spanned::new),
            }))
        } else {
            Ok(None)
        }
    }

    fn parse_mnemonic(&mut self) -> Option<(Mnemonic, Range<usize>)> {
        let mnemonic_result = self
            .lexer
            .peek()
            .map(|(token, _)| Mnemonic::try_from(token))?;

        let mnemonic = match mnemonic_result {
            Ok(mnemonic) => mnemonic,
            Err(()) => return None,
        };

        // SAFETY This unwrap is safe because the function returns if peeking returned None.
        let (_, mnemonic_span) = self.lexer.next().unwrap();

        Some((mnemonic, mnemonic_span))
    }

    fn parse_operand(&mut self) -> Result<Option<(Operand, Range<usize>)>, AssemblerError> {
        let (token, span) = match self
            .lexer
            .next_if(|(token, _)| matches!(token, Token::Literal { .. }))
        {
            Some(next) => next,
            None => return Ok(None),
        };

        if let Token::Literal(literal) = token {
            Ok(Some((Operand::Literal(literal), span)))
        } else {
            Ok(None)
        }
    }
}

impl TryFrom<&Token> for Mnemonic {
    type Error = ();

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Dfb => Ok(Mnemonic::Dfb),
            Token::Dfw => Ok(Mnemonic::Dfw),
            Token::Equ => Ok(Mnemonic::Equ),
            Token::Inl => Ok(Mnemonic::Inl),
            Token::Hlt => Ok(Mnemonic::Hlt),
            Token::Org => Ok(Mnemonic::Org),
            Token::Sct => Ok(Mnemonic::Sct),
            // Token:: => Ok(Mnemonic::),
            _ => Err(()),
        }
    }
}
