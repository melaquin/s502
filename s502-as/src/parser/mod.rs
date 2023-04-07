//! Parser that builds a syntax tree that represents the input assembly.
//! The "tree" is really a sequence of actions for the generation stage to perform.

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
    current_parent_label: Option<String>,
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
            current_parent_label: None,
        }
    }

    pub fn parse_program(mut self) -> Result<Program, Vec<AssemblerError>> {
        while self.lexer.peek().is_some() {
            // Tell the generation stage where this line starts.
            if self.program.is_empty() {
                self.program.push(Action::LineStart(0));
            } else if let Some(Action::LineEnd(line_end)) = self.program.last() {
                self.program.push(Action::LineStart(*line_end));
            }

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

    /// Parse a line including the label, mnemonic, and operand.
    fn parse_line(&mut self) -> Result<(), AssemblerError> {
        // Initialize to 0 to satisfy the compiler even though
        // parse_line is only called when lexer.peek() is Some.
        let mut line_end = 0;
        // Label appears first.
        let label = self.parse_label()?;
        // Add it to the program right away because if the instruction is an include, then
        // parse_instruction will put the included file in the program before returning.
        if let Some(label) = label {
            // So far this is where the line ends.
            line_end = label.span.end;
            // Push the label on the stack so the generation stage knows where
            // the label is in the code.
            self.program.push(Action::Label(label));
        }

        let instruction = self.parse_instruction()?;

        let included_program = if let Some(instruction) = instruction {
            // The line might actualy end here.
            line_end = instruction.span.end;

            if let Mnemonic::Inl = instruction.val.mnemonic.val {
                Some(
                    if let Value::Include(included_program) =
                        instruction.val.operand.unwrap().val.value.val
                    {
                        included_program
                    } else {
                        unreachable!()
                    },
                )
            } else {
                self.program.push(Action::Instruction(instruction));
                None
            }
        } else {
            None
        };

        let eol_end = self.skip_to_eol();

        // Or, if there was a comment, then it actually ends there.
        line_end = if eol_end != 0 { eol_end } else { line_end };
        self.program.push(Action::LineEnd(line_end));

        if let Some((included_name, mut included_program)) = included_program {
            self.program.push(Action::PushInclude(included_name));
            self.program.append(&mut included_program);
            self.program.push(Action::PopInclude);
        }

        Ok(())
    }

    /// Parses an optional label at the beginning of a line.
    /// Matches the syntax (GLOBAL | PERIOD)? ID.
    fn parse_label(&mut self) -> Result<Option<Spanned<Label>>, AssemblerError> {
        // Only want to parse a label if we see one of these four tokens, otherwise
        // there is no label to parse.
        let (mut token, mut main_span) = match self.lexer.next_if(|(token, _)| {
            matches!(token, Token::Global)
                || matches!(token, Token::Period)
                || matches!(token, Token::Ident { .. })
        }) {
            Some(next) => next,
            None => return Ok(None),
        };

        // A label  may start with either `!`, indicating that it should be exported,
        // or `.`, indicating that it is a sublabel of the most recent top-level label.
        let (visibility_span, first_period_span) = match token {
            Token::Global => {
                let attributes = (Some(main_span.clone()), None);
                (token, main_span) = self.lexer.next().ok_or(AssemblerError {
                    message: "Unexpected end of file, expected a label".to_string(),
                    labels: vec![(
                        Location {
                            span: main_span,
                            file_name: self.file_name.clone(),
                        },
                        None,
                    )],
                    help: None,
                })?;
                attributes
            }
            Token::Period => {
                let attributes = (None, Some(main_span.clone()));
                (token, main_span) = self.lexer.next().ok_or(AssemblerError {
                    message: "Unexpected end of file, expected a label".to_string(),
                    labels: vec![(
                        Location {
                            span: main_span,
                            file_name: self.file_name.clone(),
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
                            span: main_span,
                            file_name: self.file_name.clone(),
                        },
                        Some("Expected a label".to_string()),
                    )],
                    help: None,
                });
            }
        };

        // A top level label may explicitly prefix a sublabel if wanted for clarity.
        let sublabel_period = self
            .lexer
            .next_if(|(token, _)| matches!(token, Token::Period));

        let sublabel_identifier = if let Some((_, sublabel_period_span)) = sublabel_period {
            // Expect an identifier to follow.
            let (sublabel_identifier, sublabel_identifier_span) = match self
                .lexer
                .next_if(|(token, _)| matches!(token, Token::Ident { .. }))
            {
                Some(next) => next,
                None => {
                    return Err(AssemblerError {
                        message: "Expected a label after `.`".to_string(),
                        labels: vec![(
                            Location {
                                span: sublabel_period_span,
                                file_name: self.file_name.clone(),
                            },
                            None,
                        )],
                        help: None,
                    })
                }
            };

            // Don't allow nested sublabels.
            if let Some(first_period_span) = first_period_span {
                return Err(AssemblerError {
                    message: format!(
                        "Cannot nest sublabel {} under {}",
                        sublabel_identifier, identifier
                    ),
                    labels: vec![
                        (
                            Location {
                                span: sublabel_period_span.start..sublabel_identifier_span.end,
                                file_name: self.file_name.clone(),
                            },
                            Some("Nested sublabels are not allowed".to_string()),
                        ),
                        (
                            Location {
                                span: first_period_span.start..main_span.end,
                                file_name: self.file_name.clone(),
                            },
                            Some("sublabel already created here".to_string()),
                        ),
                    ],
                    help: None,
                });
            } else if let Some((_, nested_sublabel_period_span)) = self
                .lexer
                .next_if(|(token, _)| matches!(token, Token::Period))
            {
                // The parent label was explicitly specified *and* there is a second (nested) sublabel.
                // Expect an identifier follow.
                let nested_sublabel_span_end = match self
                    .lexer
                    .next_if(|(token, _)| matches!(token, Token::Ident { .. }))
                {
                    Some((_, identifier_span)) => identifier_span.end,
                    None => nested_sublabel_period_span.end,
                };

                return Err(AssemblerError {
                    message: format!("Cannot nest sublabel under {}", sublabel_identifier),
                    labels: vec![
                        (
                            Location {
                                span: nested_sublabel_period_span.start..nested_sublabel_span_end,
                                file_name: self.file_name.clone(),
                            },
                            Some("Nested sublabels are not allowed".to_string()),
                        ),
                        (
                            Location {
                                span: sublabel_period_span.start..sublabel_identifier_span.end,
                                file_name: self.file_name.clone(),
                            },
                            Some("sublabel already created here".to_string()),
                        ),
                    ],
                    help: None,
                });
            }

            Some((
                match sublabel_identifier {
                    Token::Ident(ident) => ident,
                    _ => unreachable!(),
                },
                sublabel_identifier_span,
            ))
        } else {
            None
        };

        Ok(Some(
            if let Some(sublabel_identifier) = sublabel_identifier {
                if visibility_span.is_some() {
                    return Err(AssemblerError {
                        message: String::from("Cannot specify a global visibility with a sublabel"),
                        labels: vec![(
                            Location {
                                span: visibility_span.unwrap(),
                                file_name: self.file_name.clone(),
                            },
                            None,
                        )],
                        help: None,
                    });
                }
                let label_span = main_span.start..sublabel_identifier.1.end;
                Spanned::new((
                    Label::Sub((
                        Some(Spanned::new((identifier, main_span))),
                        // NOTE Should this start with the nested sublabel period span start?
                        // Depends on if we will actually use it.
                        Spanned::new(sublabel_identifier),
                    )),
                    label_span,
                ))
            } else if let Some(period_span) = first_period_span {
                let label_span = period_span.start..main_span.end;
                Spanned::new((
                    Label::Sub((None, Spanned::new((identifier, main_span)))),
                    label_span,
                ))
            } else {
                self.current_parent_label = Some(identifier.clone());
                Spanned::new((
                    Label::Top(TopLabel {
                        name: identifier,
                        visibility: if visibility_span.is_some() {
                            Visibility::Global
                        } else {
                            Visibility::Object
                        },
                    }),
                    main_span,
                ))
            },
        ))
    }

    fn parse_instruction(&mut self) -> Result<Option<Spanned<Instruction>>, AssemblerError> {
        let parsed_mnemonic = self.parse_mnemonic();
        let mut parsed_operand = None;
        // If mnemonic is implied then don't try to parse what follows
        // an operand, return and let parse_line skip it as a comment.
        // TODO probably get rid of all these if lets and just return none if mnemonic is none
        if let Some(ref mnemonic) = parsed_mnemonic {
            if mnemonic.0.is_implied() {
                return Ok(Some(Spanned::new((
                    Instruction {
                        mnemonic: Spanned::new(mnemonic.clone()),
                        operand: None,
                    },
                    mnemonic.1.clone(),
                ))));
            }
            parsed_operand = self.parse_operand()?;
        }

        // Handle the include directive here so the nested parser can give its Items to the generation stage.
        // Other directives will be handled in that stage.
        if let Some((ref mnemonic, ref mnemonic_span)) = parsed_mnemonic {
            if let Mnemonic::Inl = mnemonic {
                if let Some(Spanned {
                    val:
                        Operand {
                            mode: OperandMode::Address,
                            modifier: None,
                            value:
                                Spanned {
                                    val: Value::String(to_include_name),
                                    span: _,
                                },
                        },
                    span: to_include_span,
                }) = parsed_operand
                {
                    // Expect the included file extension to be 65a.
                    return match Path::new(&to_include_name).extension() {
                        Some(extension) if extension == "65a" => {
                            let (included_name, parse_result) = self.handle_include(
                                to_include_name,
                                mnemonic_span.start..to_include_span.end,
                            );

                            if let Err(mut included_errors) = parse_result {
                                self.errors.append(&mut included_errors);
                                return Ok(None);
                            }

                            // None of these spans are ever actually used because parse_line will
                            // check for an Inl mnemonic and not include it in the Program. Do
                            // this because the operand is an entire included file and Locaiton
                            // cannot span multiple files.
                            Ok(Some(Spanned::new((
                                Instruction {
                                    mnemonic: Spanned::new((Mnemonic::Inl, mnemonic_span.clone())),
                                    operand: Some(Spanned::new((
                                        Operand {
                                            mode: OperandMode::Immediate,
                                            modifier: None,
                                            value: Spanned::new((
                                                Value::Include((
                                                    included_name.unwrap(),
                                                    parse_result.unwrap(),
                                                )),
                                                to_include_span.clone(),
                                            )),
                                        },
                                        to_include_span.clone(),
                                    ))),
                                },
                                mnemonic_span.start..to_include_span.end,
                            ))))
                        }
                        // Wrong or no extension, error.
                        _ => Err(AssemblerError {
                            message: format!("Could not include \"{}\"", to_include_name),
                            labels: vec![(
                                Location {
                                    span: to_include_span,
                                    file_name: self.file_name.clone(),
                                },
                                Some("File extension is expected to be `65a`".to_string()),
                            )],
                            help: None,
                        }),
                    };
                } else {
                    let mut labels = vec![];
                    if let Some(operand) = parsed_operand {
                        labels.push((
                            Location {
                                span: operand.span,
                                file_name: self.file_name.clone(),
                            },
                            // The operand cannot be a reference to a macro that is a string literal
                            // because inclusion is done during parsing and macros are not evaluated
                            // until code generation.
                            Some("Expected an unmodified string literal".to_string()),
                        ));
                    }
                    return Err(AssemblerError {
                        message: "Invalid operand to `inl`".to_string(),
                        labels: labels,
                        help: None,
                    });
                }
            }
        }

        // If we parsed a mnemonic then create an Instruction
        // with optional operand, otherwise None.
        if let Some(mnemonic) = parsed_mnemonic {
            let instruction_span = mnemonic.1.start
                ..parsed_operand
                    .as_ref()
                    .map_or(mnemonic.1.end, |operand| operand.span.end);
            Ok(Some(Spanned::new((
                Instruction {
                    mnemonic: Spanned::new(mnemonic),
                    operand: parsed_operand,
                },
                instruction_span,
            ))))
        } else {
            Ok(None)
        }
    }

    fn parse_mnemonic(&mut self) -> Option<(Mnemonic, Range<usize>)> {
        let mnemonic = self
            .lexer
            .peek()
            .map(|(token, _)| Mnemonic::try_from(token))?
            .ok()?;

        // SAFETY This unwrap is safe because the function returns if peek returned None.
        let (_, mnemonic_span) = self.lexer.next().unwrap();

        Some((mnemonic, mnemonic_span))
    }

    fn parse_operand(&mut self) -> Result<Option<Spanned<Operand>>, AssemblerError> {
        let (first_token, first_span) = match self.lexer.next_if(|(token, _)| {
            matches!(token, Token::Literal { .. })
                || matches!(token, Token::Ident { .. })
                || matches!(token, Token::A)
                || matches!(token, Token::Immediate)
                || matches!(token, Token::LParen)
                || matches!(token, Token::LAngle)
                || matches!(token, Token::RAngle)
                || matches!(token, Token::Period)
        }) {
            Some(next) => next,
            None => return Ok(None),
        };

        // Decision tree to find which address mode the operand may be.
        Ok(Some(match first_token {
            // Simple enough, A makes up an entire operand.
            Token::A => Spanned::new((
                Operand {
                    mode: OperandMode::Accumulator,
                    modifier: None,
                    value: Spanned::new((Value::Accumulator, first_span.clone())),
                },
                first_span,
            )),
            Token::Immediate => {
                // Immediate address mode, only a value follows.
                let operand_start = first_span.start;
                let (modifier, value) = self.parse_modified_value()?.ok_or(AssemblerError {
                    message: "Expected value after `#`".to_string(),
                    labels: vec![(
                        Location {
                            span: first_span,
                            file_name: self.file_name.clone(),
                        },
                        None,
                    )],
                    help: None,
                })?;
                // Get the span end before moving the value into the Operand.
                let operand_end = value.span.end;
                Spanned::new((
                    Operand {
                        mode: OperandMode::Immediate,
                        modifier,
                        value,
                    },
                    operand_start..operand_end,
                ))
            }
            Token::LAngle => {
                // Parsed modifier, expect value to follow.
                let value = self.expect_value(first_span.clone())?;
                let value_end = value.span.end;

                let modifier = Some(Spanned::new((Modifier::HighByte, first_span.clone())));

                let (peeked_token, _) = match self.lexer.peek() {
                    None => {
                        // Plain absolute or zeropage operand.
                        return Ok(Some(Spanned::new((
                            Operand {
                                mode: OperandMode::Address,
                                modifier,
                                value,
                            },
                            first_span.start..value_end,
                        ))));
                    }
                    Some(peeked_spanned_token) => peeked_spanned_token,
                };

                match peeked_token {
                    // May be abs,_ or zpg,_ indexed mode.
                    Token::Comma => {
                        let (_, comma_span) = self.lexer.next().unwrap();
                        match self.lexer.next() {
                            Some((Token::X, x_span)) => Spanned::new((
                                Operand {
                                    mode: OperandMode::XIndexed,
                                    modifier,
                                    value,
                                },
                                first_span.start..x_span.end,
                            )),
                            Some((Token::Y, y_span)) => Spanned::new((
                                Operand {
                                    mode: OperandMode::YIndexed,
                                    modifier,
                                    value,
                                },
                                first_span.start..y_span.end,
                            )),
                            _ => {
                                return Err(AssemblerError {
                                    message: "Expected `x` or `y` after `,`".to_string(),
                                    labels: vec![(
                                        Location {
                                            span: comma_span,
                                            file_name: self.file_name.clone(),
                                        },
                                        None,
                                    )],
                                    help: None,
                                })
                            }
                        }
                    }
                    // The token was something else, may be a comment or eol.
                    _ => Spanned::new((
                        Operand {
                            mode: OperandMode::Address,
                            modifier,
                            value,
                        },
                        first_span.start..value_end,
                    )),
                }
            }
            Token::RAngle => {
                // Parsed modifier, expect value to follow.
                let value = self.expect_value(first_span.clone())?;
                let value_end = value.span.end;

                let modifier = Some(Spanned::new((Modifier::LowByte, first_span.clone())));

                let (peeked_token, _) = match self.lexer.peek() {
                    None => {
                        // Plain absolute or zeropage operand.
                        return Ok(Some(Spanned::new((
                            Operand {
                                mode: OperandMode::Address,
                                modifier,
                                value,
                            },
                            first_span.start..value_end,
                        ))));
                    }
                    Some(peeked_spanned_token) => peeked_spanned_token,
                };

                match peeked_token {
                    // May be abs,_ or zpg,_ indexed mode.
                    Token::Comma => {
                        let (_, comma_span) = self.lexer.next().unwrap();
                        match self.lexer.next() {
                            Some((Token::X, x_span)) => Spanned::new((
                                Operand {
                                    mode: OperandMode::XIndexed,
                                    modifier,
                                    value,
                                },
                                first_span.start..x_span.end,
                            )),
                            Some((Token::Y, y_span)) => Spanned::new((
                                Operand {
                                    mode: OperandMode::YIndexed,
                                    modifier,
                                    value,
                                },
                                first_span.start..y_span.end,
                            )),
                            _ => {
                                return Err(AssemblerError {
                                    message: "Expected `x` or `y` after `,`".to_string(),
                                    labels: vec![(
                                        Location {
                                            span: comma_span,
                                            file_name: self.file_name.clone(),
                                        },
                                        None,
                                    )],
                                    help: None,
                                })
                            }
                        }
                    }
                    // The token was something else, may be a comment or eol.
                    _ => Spanned::new((
                        Operand {
                            mode: OperandMode::Address,
                            modifier,
                            value,
                        },
                        first_span.start..value_end,
                    )),
                }
            }
            Token::LParen => {
                // One of:
                //   indirect
                //   X-indexed, indirect
                //   indirect, Y-indexed
                // All of them start with a value.
                let (modifier, value) = self.parse_modified_value()?.ok_or(AssemblerError {
                    message: "Expected value after `(`".to_string(),
                    labels: vec![(
                        Location {
                            span: first_span.clone(),
                            file_name: self.file_name.clone(),
                        },
                        None,
                    )],
                    help: None,
                })?;
                let (peeked_token, _) = self.lexer.peek().ok_or(AssemblerError {
                    message: "Expected `)` or `,` after value".to_string(),
                    labels: vec![(
                        Location {
                            span: value.span.clone(),
                            file_name: self.file_name.clone(),
                        },
                        None,
                    )],
                    help: None,
                })?;
                match peeked_token {
                    Token::RParen => {
                        // May be indirect or indirect, Y-indexed.
                        // SAFETY This will not panic because the enclosing match statement peeks and finds Some.
                        let (_, rparen_span) = self.lexer.next().unwrap();
                        match self.lexer.peek() {
                            //ind,Y if there was a comma after rparen.
                            Some((Token::Comma, _)) => {
                                let (_, comma_span) = self.lexer.next().unwrap();
                                match self.lexer.next() {
                                    Some((Token::Y, y_span)) => Spanned::new((
                                        Operand {
                                            mode: OperandMode::IndirectY,
                                            modifier,
                                            value,
                                        },
                                        first_span.start..y_span.end,
                                    )),
                                    _ => {
                                        return Err(AssemblerError {
                                            message: "Expected `y` after `,`".to_string(),
                                            labels: vec![(
                                                Location {
                                                    span: comma_span,
                                                    file_name: self.file_name.clone(),
                                                },
                                                None,
                                            )],
                                            help: None,
                                        })
                                    }
                                }
                            }
                            // Otherwise indirect.
                            _ => Spanned::new((
                                Operand {
                                    mode: OperandMode::Indirect,
                                    modifier,
                                    value,
                                },
                                first_span.start..rparen_span.end,
                            )),
                        }
                    }
                    Token::Comma => {
                        // Must be ind,X because there was a comma after the value.
                        let (_, comma_span) = self.lexer.next().unwrap();
                        match self.lexer.next() {
                            Some((Token::X, x_span)) => match self.lexer.next() {
                                Some((Token::RParen, rparen_span)) => Spanned::new((
                                    Operand {
                                        mode: OperandMode::XIndirect,
                                        modifier,
                                        value,
                                    },
                                    first_span.start..rparen_span.end,
                                )),
                                _ => {
                                    return Err(AssemblerError {
                                        message: "Expected `)` after `x`".to_string(),
                                        labels: vec![(
                                            Location {
                                                span: x_span,
                                                file_name: self.file_name.clone(),
                                            },
                                            None,
                                        )],
                                        help: None,
                                    })
                                }
                            },
                            _ => {
                                return Err(AssemblerError {
                                    message: "Expected `x` after `,`".to_string(),
                                    labels: vec![(
                                        Location {
                                            span: comma_span,
                                            file_name: self.file_name.clone(),
                                        },
                                        None,
                                    )],
                                    help: None,
                                })
                            }
                        }
                    }
                    _ => {
                        // An lparen must have a matching rparen for ind,
                        // or comma for x,ind.
                        return Err(AssemblerError {
                            message: "Expected `)` or `,` after value".to_string(),
                            labels: vec![(
                                Location {
                                    span: value.span,
                                    file_name: self.file_name.clone(),
                                },
                                None,
                            )],
                            help: None,
                        });
                    }
                }
            }
            Token::Period => {
                // Expect an identifier to follow.
                let (sublabel_identifier, sublabel_identifier_span) = match self
                    .lexer
                    .next_if(|(token, _)| matches!(token, Token::Ident { .. }))
                {
                    Some(next) => next,
                    None => {
                        return Err(AssemblerError {
                            message: String::from("Expected a label after `.`"),
                            labels: vec![(
                                Location {
                                    span: first_span,
                                    file_name: self.file_name.clone(),
                                },
                                None,
                            )],
                            help: None,
                        })
                    }
                };

                let sublabel_identifier = match sublabel_identifier {
                    Token::Ident(identifier) => identifier,
                    _ => unreachable!(),
                };

                // first_span.end = sublabel_identifier_span.end;
                let operand_span = first_span.start..sublabel_identifier_span.end;

                let value = match &self.current_parent_label {
                    Some(parent) => Value::Reference(format!("{}.{}", parent, sublabel_identifier)),
                    None => {
                        return Err(AssemblerError {
                            message: String::from(
                                "No parent label under which to refer to a sublabel",
                            ),
                            labels: vec![(
                                Location {
                                    span: operand_span,
                                    file_name: self.file_name.clone(),
                                },
                                None,
                            )],
                            help: None,
                        })
                    }
                };

                Spanned::new((
                    Operand {
                        mode: OperandMode::Address,
                        modifier: None,
                        value: Spanned::new((value, operand_span.clone())),
                    },
                    operand_span,
                ))
            }
            _ => {
                // The first token was either a literal or identifier. One of:
                //   absolute
                //   absolute, X-indexed
                //   absolute, Y-indexed
                //   zeropage
                //   zeropage, X-indexed
                //   zeropage, Y-indexed
                let value = Spanned::new((
                    match first_token {
                        Token::Literal(Literal::Byte(byte)) => Value::Byte(byte),
                        Token::Literal(Literal::Word(word)) => Value::Word(word),
                        Token::Literal(Literal::String(string)) => Value::String(string),
                        Token::Ident(ident) => Value::Reference(ident),
                        _ => unreachable!(),
                    },
                    first_span.clone(),
                ));
                let (peeked_token, _) = match self.lexer.peek() {
                    None => {
                        // Plain absolute or zeropage operand.
                        return Ok(Some(Spanned::new((
                            Operand {
                                mode: OperandMode::Address,
                                modifier: None,
                                value,
                            },
                            first_span,
                        ))));
                    }
                    Some(peeked_spanned_token) => peeked_spanned_token,
                };
                match peeked_token {
                    // May be abs,_ or zpg,_ indexed mode.
                    Token::Comma => {
                        let (_, comma_span) = self.lexer.next().unwrap();
                        match self.lexer.next() {
                            Some((Token::X, x_span)) => Spanned::new((
                                Operand {
                                    mode: OperandMode::XIndexed,
                                    modifier: None,
                                    value,
                                },
                                first_span.start..x_span.end,
                            )),
                            Some((Token::Y, y_span)) => Spanned::new((
                                Operand {
                                    mode: OperandMode::YIndexed,
                                    modifier: None,
                                    value,
                                },
                                first_span.start..y_span.end,
                            )),
                            _ => {
                                return Err(AssemblerError {
                                    message: "Expected `x` or `y` after `,`".to_string(),
                                    labels: vec![(
                                        Location {
                                            span: comma_span,
                                            file_name: self.file_name.clone(),
                                        },
                                        None,
                                    )],
                                    help: None,
                                })
                            }
                        }
                    }
                    // The token was something else, may be a comment or eol.
                    _ => Spanned::new((
                        Operand {
                            mode: OperandMode::Address,
                            modifier: None,
                            value,
                        },
                        first_span,
                    )),
                }
            }
        }))
    }

    /// Parse a value with an optional modifier. Returns Err if there was a modifier with no value.
    fn parse_modified_value(
        &mut self,
    ) -> Result<Option<(Option<Spanned<Modifier>>, Spanned<Value>)>, AssemblerError> {
        if let Some(modifier) = self.parse_modifier() {
            // If there's a modifier then expect a value to follow.
            let modifier_span = modifier.span.clone();
            Ok(Some((Some(modifier), self.expect_value(modifier_span)?)))
        } else if let Some(value) = self.parse_value()? {
            Ok(Some((None, value)))
        } else {
            Ok(None)
        }
    }

    /// Tries to parse a modifier.
    fn parse_modifier(&mut self) -> Option<Spanned<Modifier>> {
        let (modifier_token, modifier_span) = match self
            .lexer
            .next_if(|(token, _)| matches!(token, Token::LAngle) || matches!(token, Token::RAngle))
        {
            Some(next) => next,
            None => return None,
        };

        Some(Spanned::new((
            if let Token::LAngle = modifier_token {
                Modifier::HighByte
            } else {
                Modifier::LowByte
            },
            modifier_span,
        )))
    }

    /// Tries to parse a value, returns Err if it failed.
    fn expect_value(
        &mut self,
        prefix_span: Range<usize>,
    ) -> Result<Spanned<Value>, AssemblerError> {
        if let Some(value) = self.parse_value()? {
            Ok(value)
        } else {
            // This is only ever called after a modifier was parsed.
            Err(AssemblerError {
                message: "Expected value after modifier".to_string(),
                labels: vec![(
                    Location {
                        span: prefix_span,
                        file_name: self.file_name.clone(),
                    },
                    None,
                )],
                help: None,
            })
        }
    }

    /// Tries to parse a value.
    fn parse_value(&mut self) -> Result<Option<Spanned<Value>>, AssemblerError> {
        // let next_token = self.lexer.next_if(|(token, _)| {
        //     matches!(token, Token::Literal { .. })
        //         || matches!(token, Token::Ident { .. })
        //         || matches!(token, Token::Period)
        // });

        let (value_token, mut value_span) = match self.lexer.next_if(|(token, _)| {
            matches!(token, Token::Literal { .. })
                || matches!(token, Token::Ident { .. })
                || matches!(token, Token::Period)
        }) {
            Some(next) => next,
            None => return Ok(None),
        };

        Ok(Some(Spanned::new((
            match value_token {
                Token::Literal(Literal::Byte(byte)) => Value::Byte(byte),
                Token::Literal(Literal::Word(word)) => Value::Word(word),
                Token::Literal(Literal::String(string)) => Value::String(string),
                Token::Ident(ident) => Value::Reference(ident),
                Token::Period => {
                    // Expect an identifier to follow.
                    let (sublabel_identifier, sublabel_identifier_span) = match self
                        .lexer
                        .next_if(|(token, _)| matches!(token, Token::Ident { .. }))
                    {
                        Some(next) => next,
                        None => {
                            return Err(AssemblerError {
                                message: String::from("Expected a label after `.`"),
                                labels: vec![(
                                    Location {
                                        span: value_span,
                                        file_name: self.file_name.clone(),
                                    },
                                    None,
                                )],
                                help: None,
                            })
                        }
                    };

                    let sublabel_identifier = match sublabel_identifier {
                        Token::Ident(identifier) => identifier,
                        _ => unreachable!(),
                    };
                    value_span.end = sublabel_identifier_span.end;

                    let value = match &self.current_parent_label {
                        Some(parent) => {
                            Value::Reference(format!("{}.{}", parent, sublabel_identifier))
                        }
                        None => {
                            return Err(AssemblerError {
                                message: String::from(
                                    "No parent label under which to refer to a sublabel",
                                ),
                                labels: vec![(
                                    Location {
                                        span: value_span,
                                        file_name: self.file_name.clone(),
                                    },
                                    None,
                                )],
                                help: None,
                            })
                        }
                    };

                    // Ok(Some(Spanned::new((value, value_span))))
                    value
                }
                _ => unreachable!(),
            },
            value_span,
        ))))
    }

    /// Read and parse an included file, preventing circular inclusion.
    /// Returns the included file ID (if reading was successful) and
    /// the result of parsing the included file.
    fn handle_include(
        &mut self,
        to_include_name: String,
        to_include_span: Range<usize>,
    ) -> (Option<String>, Result<Program, Vec<AssemblerError>>) {
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
                                file_name: self.include_stack.last().unwrap().included.clone(),
                            },
                            Some(format!("Could not include \"{}\"", to_include_name)),
                        ),
                        (
                            Location {
                                span: include.loc.span.clone(),
                                file_name: include.loc.file_name.clone(),
                            },
                            Some(if include == self.include_stack.first().unwrap() {
                                "Given in assembler invocatiion".to_string()
                            } else {
                                "Already included here".to_string()
                            }),
                        ),
                    ],

                    help: Some(formatdoc!(
                        "Labels can be referenced before they're defined,
                         so including `{}` here may not be necessary",
                        to_include_name
                    )),
                }]),
            );
        }

        // No recursion, read the source.
        let included_source = match fs::read_to_string(&to_include_name) {
            Err(error) => {
                return (
                    None,
                    Err(vec![AssemblerError {
                        message: format!("Could not include \"{}\": {}", to_include_name, error),
                        labels: vec![(
                            Location {
                                span: to_include_span,
                                file_name: self.include_stack.last().unwrap().included.clone(),
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
                file_name: self.include_stack.last().unwrap().included.clone(),
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
            self.id_table
                .insert(to_include_name.clone(), included_file_id);
        }
        (Some(to_include_name), parse_result)
    }

    /// Skip past the end of the line after it is done being parsed.
    /// This makes a dedicated line comment character unnecessary
    /// just like the good ol' days. Returns the index of the end of
    /// the line so the newline can be included in the listing,
    /// or 0 if no token swere skipped.
    fn skip_to_eol(&mut self) -> usize {
        // Use this in case the file ends without a newline.
        // If it returns 0 then there was nothing to skip and
        // the lexer was already at the end of the file.
        let mut last_end = 0;
        loop {
            match self.lexer.peek() {
                // No token to get the end of, so return
                // the end of the previous token.
                None => return last_end,
                Some((Token::Eol, _)) => {
                    // Include the newline because it wil be printed
                    // at the end of each line of the listing.
                    return self.lexer.next().unwrap().1.end;
                }
                _ => {
                    last_end = self.lexer.next().unwrap().1.end;
                }
            }
        }
    }
}

impl TryFrom<&Token> for Mnemonic {
    type Error = ();

    #[cfg(not(tarpaulin_include))]
    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Adc => Ok(Mnemonic::Adc),
            Token::And => Ok(Mnemonic::And),
            Token::Asl => Ok(Mnemonic::Asl),
            Token::Bcc => Ok(Mnemonic::Bcc),
            Token::Bcs => Ok(Mnemonic::Bcs),
            Token::Beq => Ok(Mnemonic::Beq),
            Token::Bit => Ok(Mnemonic::Bit),
            Token::Bmi => Ok(Mnemonic::Bmi),
            Token::Bne => Ok(Mnemonic::Bne),
            Token::Bpl => Ok(Mnemonic::Bpl),
            Token::Brk => Ok(Mnemonic::Brk),
            Token::Bvc => Ok(Mnemonic::Bvc),
            Token::Bvs => Ok(Mnemonic::Bvs),
            Token::Clc => Ok(Mnemonic::Clc),
            Token::Cld => Ok(Mnemonic::Cld),
            Token::Cli => Ok(Mnemonic::Cli),
            Token::Clv => Ok(Mnemonic::Clv),
            Token::Cmp => Ok(Mnemonic::Cmp),
            Token::Cpx => Ok(Mnemonic::Cpx),
            Token::Cpy => Ok(Mnemonic::Cpy),
            Token::Dec => Ok(Mnemonic::Dec),
            Token::Dex => Ok(Mnemonic::Dex),
            Token::Dey => Ok(Mnemonic::Dey),
            Token::Eor => Ok(Mnemonic::Eor),
            Token::Inc => Ok(Mnemonic::Inc),
            Token::Inx => Ok(Mnemonic::Inx),
            Token::Iny => Ok(Mnemonic::Iny),
            Token::Jmp => Ok(Mnemonic::Jmp),
            Token::Jsr => Ok(Mnemonic::Jsr),
            Token::Lda => Ok(Mnemonic::Lda),
            Token::Ldx => Ok(Mnemonic::Ldx),
            Token::Ldy => Ok(Mnemonic::Ldy),
            Token::Lsr => Ok(Mnemonic::Lsr),
            Token::Nop => Ok(Mnemonic::Nop),
            Token::Ora => Ok(Mnemonic::Ora),
            Token::Pha => Ok(Mnemonic::Pha),
            Token::Php => Ok(Mnemonic::Php),
            Token::Pla => Ok(Mnemonic::Pla),
            Token::Plp => Ok(Mnemonic::Plp),
            Token::Rol => Ok(Mnemonic::Rol),
            Token::Ror => Ok(Mnemonic::Ror),
            Token::Rti => Ok(Mnemonic::Rti),
            Token::Rts => Ok(Mnemonic::Rts),
            Token::Sbc => Ok(Mnemonic::Sbc),
            Token::Sec => Ok(Mnemonic::Sec),
            Token::Sed => Ok(Mnemonic::Sed),
            Token::Sei => Ok(Mnemonic::Sei),
            Token::Sta => Ok(Mnemonic::Sta),
            Token::Stx => Ok(Mnemonic::Stx),
            Token::Sty => Ok(Mnemonic::Sty),
            Token::Tax => Ok(Mnemonic::Tax),
            Token::Tay => Ok(Mnemonic::Tay),
            Token::Tsx => Ok(Mnemonic::Tsx),
            Token::Txa => Ok(Mnemonic::Txa),
            Token::Txs => Ok(Mnemonic::Txs),
            Token::Tya => Ok(Mnemonic::Tya),
            Token::Dfb => Ok(Mnemonic::Dfb),
            Token::Dfw => Ok(Mnemonic::Dfw),
            Token::Equ => Ok(Mnemonic::Equ),
            Token::Inl => Ok(Mnemonic::Inl),
            Token::Hlt => Ok(Mnemonic::Hlt),
            Token::Org => Ok(Mnemonic::Org),
            Token::Sct => Ok(Mnemonic::Sct),
            _ => Err(()),
        }
    }
}
