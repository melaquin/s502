//! Parser that builds a syntax tree that represents the input assembly.

#[cfg(test)]
mod tests;

use std::{collections::HashMap, convert::TryFrom, fs, iter::Peekable, ops::Range};

use codespan_reporting::files::SimpleFiles;
use logos::{Logos, SpannedIter};

use crate::{
    ast::*,
    error::*,
    lexer::{Literal, Token},
};

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
        lexer: SpannedLexer<'source>,
        files: &'context mut SimpleFiles<String, String>,
        include_stack: &'context mut Vec<Include>,
        id_table: &'context mut HashMap<String, usize>,
    ) -> Self {
        Self {
            file_name,
            lexer,
            files,
            include_stack,
            id_table,
            program: Program {
                lines: Vec::with_capacity(256),
            },
            errors: Vec::with_capacity(8),
        }
    }

    pub fn parse_program(mut self) -> Result<Program, Vec<AssemblerError>> {
        while self.lexer.peek().is_some() {
            match self.parse_line() {
                // The line was empty, go on to the next one.
                Ok(None) => continue,
                Ok(Some(line)) => self.program.lines.push(line),
                Err(error) => self.errors.push(error),
            }
        }

        // Add it to the files only if it hasn't been included before to prevent duplicates,
        // otherwise return hte existing ID.
        // if context.id_table.contains_key(&context.file_name) {
        //     context.id_table[&context.file_name]
        // } else {
        //     context.files.add(context.file_name, source)
        // },
        if self.errors.len() > 0 {
            Err(self.errors)
        } else {
            Ok(self.program)
        }
    }

    // pub fn parse_program(
    //     file_name: String,
    //     source: String,
    //     files: &mut SimpleFiles<String, String>,
    //     include_path: &mut Vec<Include>,
    //     id_table: &mut HashMap<String, usize>,
    // ) -> (usize, Result<Program, Vec<AssemblerError>>) {
    //     let mut lexer = Token::lexer(&source).spanned().peekable();

    //     let mut errors = Vec::with_capacity(8);
    //     let mut program = Program {
    //         lines: Vec::with_capacity(256),
    //     };

    //     // Assemble the program one line at a time. Stop when there are no more tokens.
    //     while lexer.peek().is_some() {
    //         // TODO move this to parse_mnemonic
    //         // // Handle includes specially, they cannot start with a label.
    //         // if let Some((Token::Inl, _)) = lexer.peek() {
    //         //     let (_, inl_span) = lexer.next().unwrap();
    //         //     let (included_id, parse_result) =
    //         //         handle_include(&mut lexer, inl_span, files, include_path, id_table);

    //         //     // Push included file's ID before its code so the next stage knows what file it's in.
    //         //     if let Some(id) = included_id {
    //         //         program.lines.push(OldLine::PushInclude(id));
    //         //     }
    //         //     match parse_result {
    //         //         Ok(mut included_program) => program.lines.append(&mut included_program.lines),
    //         //         Err(mut included_errors) => errors.append(&mut included_errors),
    //         //     }
    //         //     // And the next stage needs to know when the included file ends.
    //         //     if included_id.is_some() {
    //         //         program.lines.push(OldLine::PopInclude);
    //         //     }
    //         //     skip_to_eol(&mut lexer);
    //         //     continue;
    //         // }

    //         match parse_line(&mut lexer, &file_name) {
    //             // The line was empty, go on to the next one.
    //             Ok(None) => continue,
    //             Ok(Some(line)) => program.lines.push(line),
    //             Err(error) => errors.push(error),
    //         }
    //     }

    //     (
    //         // Add it to the files only if it hasn't been included before to prevent duplicates,
    //         // otherwise return hte existing ID.
    //         if id_table.contains_key(&file_name) {
    //             id_table[&file_name]
    //         } else {
    //             files.add(file_name, source)
    //         },
    //         if errors.len() > 0 {
    //             Err(errors)
    //         } else {
    //             Ok(program)
    //         },
    //     )
    // }

    /// parse the inl directive, preventing circular inclusion and parsing the included file.
    fn handle_include(
        &mut self,
        to_include_name: String,
        to_include_span: Range<usize>,
    ) -> (Option<usize>, Result<Program, Vec<AssemblerError>>) {
        // Expect to see a string literal after the inl directive.
        // let (to_include_name, to_include_span) = match context.lexer.next() {
        //     None => {
        //         return (
        //             None,
        //             Err(vec![AssemblerError {
        //                 message: "Unexpected end of file, expected a string".to_string(),
        //                 labels: vec![(
        //                     Location {
        //                         span: to_include_span,
        //                         // SAFETY Last can be unwrapped because the stack has at least one member
        //                         // before parse_program and subsequently handle_include are called.
        //                         name: context.include_stack.last().unwrap().included.clone(),
        //                     },
        //                     None,
        //                 )],
        //                 help: None,
        //             }]),
        //         );
        //     }
        //     Some((Token::Literal(Literal::String(file_name)), name_span)) => (file_name, name_span),
        //     Some((token, span)) => {
        //         return (
        //             None,
        //             Err(vec![AssemblerError {
        //                 message: format!("Unexpected token {}", token),
        //                 labels: vec![(
        //                     Location {
        //                         span,
        //                         name: context.include_stack.last().unwrap().included.clone(),
        //                     },
        //                     Some("Expected a string".to_string()),
        //                 )],
        //                 help: None,
        //             }]),
        //         );
        //     }
        // };

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

        let included_lexer = Token::lexer(&included_source).spanned().peekable();

        let included_context = ParserContext::new(
            to_include_name.clone(),
            included_lexer,
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
    fn parse_line(&mut self) -> Result<Option<Line>, AssemblerError> {
        // Label appears first.
        let label = self.parse_label()?;
        let action = self.parse_action()?;

        self.skip_to_eol();

        if let None = label {
            Ok(None)
        } else {
            Ok(Some(Line {
                label,
                action: None,
            }))
        }
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

    fn parse_action(&mut self) -> Result<Option<Action>, AssemblerError> {
        let parsed_mnemonic = self.parse_mnemonic();
        let parsed_operand = self.parse_operand();

        // handle include directive specially because it's handled by the parser, not the assembler stage.
        if let Some((mnemonic, mnemonic_span)) = parsed_mnemonic {
            if let Mnemonic::Inl = mnemonic {
                if let Ok(Some((Operand::Literal(Literal::String(to_include)), to_include_span))) =
                    parsed_operand
                {
                    let parse_result =
                        self.handle_include(to_include, mnemonic_span.start..to_include_span.end);
                    // TODO put result into context
                    // let included__id = if self.id_table.contains_key(&self.file_name) {
                    //     self.id_table[&self.file_name]
                    // } else {
                    //     self.files.add(self.file_name.clone(), source)
                    // };
                    // // Push included file's ID before its code so the next stage knows what file it's in.
                    // if let Some(id) = included_id {
                    //     program.lines.push(OldLine::PushInclude(id));
                    // }
                    // match parse_result {
                    //     Ok(mut included_program) => {
                    //         program.lines.append(&mut included_program.lines)
                    //     }
                    //     Err(mut included_errors) => errors.append(&mut included_errors),
                    // }
                    // // And the next stage needs to know when the included file ends.
                    // if included_id.is_some() {
                    //     program.lines.push(OldLine::PopInclude);
                    // }
                    // skip_to_eol(&mut lexer);
                    // continue;
                }
            }
        }

        Ok(None)
    }

    fn parse_mnemonic(&mut self) -> Option<(Mnemonic, Range<usize>)> {
        let mnemonic_result = match self.lexer.peek() {
            None => return None,
            Some((token, _)) => Mnemonic::try_from(token),
        };

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
