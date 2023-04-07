pub mod binary;

use std::{collections::HashMap, iter::Peekable, ops::Range, vec};

use codespan_reporting::files::SimpleFiles;

use crate::{ast::*, error::*};

pub type Object = Vec<Section>;

pub struct Section {
    pub name: String,
    pub data: [u8; 65536],
    // The offset into the current section where to put the next byte of code.
    pub origin: usize,
    pub lowest_origin: Option<usize>,
    pub highest_origin: usize,
    /// The ranges of memory that have had code put into them. This is used to warn
    /// the programmer when they overwrite code, and it is only used in binary mode.
    pub used_ranges: Vec<Range<usize>>,
    pub labels: Vec<SectionLabel>,
    pub references: Vec<Reference>,
}

pub struct SectionLabel {
    pub name: String,
    pub visibility: Visibility,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Reference {
    /// The symbol being referenced.
    pub name: String,
    /// Where in the section to place the reference.
    pub offset: usize,
    /// Which byte of the referenced symbol to place.
    pub modifier: Option<Spanned<Modifier>>,
    // Whether the instruction making hte reference is a branch.
    pub branch: bool,
    /// Where in the source code the reference is.
    pub location: Location,
}

#[derive(Debug)]
enum ActualValue {
    Accumulator,
    Byte(u8),
    Word(u16),
    String(String),
    /// The value is a reference to a label.
    Reference(Reference),
}

#[derive(Debug)]
pub enum Macro {
    Byte(u8, Location),
    Word(u16, Location),
    String(String, Location),
}

pub struct Listing {
    location: Option<(usize, usize, usize)>,
    code: String,
}

pub struct GeneratorContext<'context> {
    /// The program to assemble.
    program: Peekable<vec::IntoIter<Action>>,
    /// Whether or not to assemble in binary mode.
    binary: bool,
    /// The map from file names to ID numbers.
    id_table: &'context HashMap<String, usize>,
    /// The source files. This is used for getting excerpts for listings.
    files: &'context SimpleFiles<String, String>,
    /// The object being built.
    object: Vec<Section>,
    /// The index into object the current section being assembled is at.
    current_section: usize,
    /// Where in the source code the current line starts. Used for creating listings.
    current_line_source_start: usize,
    current_line_source_end: usize,
    /// Where in the sections the current line starts. Used for creating listings.
    current_line_section_start: usize,
    current_line_section_end: usize,
    /// The file include stack. The last element is the current file.
    include_stack: Vec<(String, usize)>,
    /// The most recent parent label, used for filling in implied parent of sublabels.
    last_parent_label: Option<(Spanned<String>, String)>,
    /// Macros defined during generation.
    macros: HashMap<String, Macro>,
    /// A label appeared on this line so a macro may be created.
    macro_valid: bool,
    /// Errors found during code generation.
    errors: Vec<AssemblerError>,
    // old_listing: Vec<String>,
    listing: Vec<Listing>,
}

impl<'context> GeneratorContext<'context> {
    /// Create a new context for assembling a program.
    pub fn new(
        program: Program,
        binary: bool,
        top_file_name: String,
        id_table: &'context HashMap<String, usize>,
        files: &'context SimpleFiles<String, String>,
    ) -> Self {
        Self {
            program: program.into_iter().peekable(),
            binary,
            id_table,
            files,
            object: Object::with_capacity(2),
            current_section: 0,
            current_line_source_start: 0,
            current_line_source_end: 0,
            current_line_section_start: 0,
            current_line_section_end: 0,
            // Start with the top level file.
            include_stack: vec![(top_file_name, 0)],
            last_parent_label: None,
            macros: HashMap::with_capacity(32),
            macro_valid: false,
            errors: Vec::with_capacity(4),
            listing: Vec::with_capacity(512),
        }
    }

    pub fn generate_code(mut self) -> Result<(Object, Vec<Listing>), Vec<AssemblerError>> {
        // We need at least one section for hte firstl ine start action, so just give it the default name.
        self.object.push(Section {
            name: String::from("default"),
            data: [0; 65536],
            origin: 0,
            lowest_origin: None,
            highest_origin: 0,
            used_ranges: Vec::with_capacity(8),
            labels: Vec::with_capacity(64),
            references: Vec::with_capacity(128),
        });

        while self.program.peek().is_some() {
            let action = self.program.next().unwrap();
            match action {
                Action::LineStart(start_index) => {
                    self.current_line_source_start = start_index;
                    self.current_line_section_start = self.object[self.current_section].origin;
                }
                Action::LineEnd(line_end) => {
                    self.current_line_source_end = line_end;
                    self.create_listing_line(line_end);
                    self.macro_valid = false;
                }
                Action::PushInclude(included_name) => {
                    self.listing.push(Listing {
                        location: None,
                        code: format!("**** START INCLUDED FILE `{}`", included_name),
                    });
                    self.include_stack
                        .push((included_name, self.current_line_source_end));
                }
                Action::PopInclude => {
                    let (file_name, continue_source_index) = self.include_stack.pop().unwrap();
                    self.listing.push(Listing {
                        location: None,
                        code: format!("**** END   INCLUDED FILE `{}`", file_name),
                    });
                    self.current_line_source_start = continue_source_index;
                    self.current_line_section_start = self.object[self.current_section].origin;
                }
                Action::Label(label) => {
                    if self.handle_label(label).is_err() {
                        self.skip_to_eol();
                        continue;
                    }
                }
                Action::Instruction(instruction) => {
                    match self.handle_instruction(instruction) {
                        Err(error) => {
                            self.errors.push(error);
                            self.current_line_section_end =
                                self.object[self.current_section].origin;
                        }
                        Ok(bytes_inserted) => {
                            self.current_line_section_end =
                                self.current_line_section_start + bytes_inserted;
                        }
                    }

                    self.skip_to_eol();
                }
            }
        }

        if !self.errors.is_empty() {
            Err(self.errors)
        } else {
            Ok((self.object, self.listing))
        }
    }

    fn skip_to_eol(&mut self) {
        while self
            .program
            .next_if(|action| !matches!(action, Action::LineEnd { .. }))
            .is_some()
        {}
    }

    fn handle_instruction(
        &mut self,
        instruction: Spanned<Instruction>,
    ) -> Result<usize, AssemblerError> {
        let mnemonic = instruction.mnemonic.clone();
        // If the instruction is implied then handle it, otherwise take out
        // the operand to use later.
        let (spanned_operand, address_mode) = match instruction.val.operand {
            None => {
                // No operand so it's expected to be implied.
                return self.handle_implied_instruction(mnemonic.val, instruction.span.clone());
            }
            Some(operand) => self.resolve_operand(operand, mnemonic.is_branch()),
        };
        let (operand, operand_span) = (spanned_operand.val, spanned_operand.span);

        // Handle directives first.
        match mnemonic.val {
            Mnemonic::Dfb => return self.handle_dfb(Spanned::new((operand, operand_span.clone()))),
            Mnemonic::Dfw => return self.handle_dfw(Spanned::new((operand, operand_span.clone()))),
            Mnemonic::Equ => return self.create_macro(operand, instruction.span.clone()),
            // Skip hlt directive because it's essentially an implied instruction so
            // it's treated as one. Also skip inl because it's handled in the parser.
            Mnemonic::Org => return self.change_origin(operand, instruction.span.clone()),
            Mnemonic::Sct => return self.change_section(operand, instruction.span),
            _ => {}
        }

        // Insert opcode.
        if let Some(byte) = OPCODES[mnemonic.val][address_mode] {
            self.insert_byte(byte);
        } else {
            return Err(AssemblerError {
                message: format!(
                    "{} address mode is invalid for instruction `{}`",
                    address_mode.string_rep(mnemonic.val),
                    mnemonic.val,
                ),
                labels: vec![(
                    Location {
                        span: operand_span.clone(),
                        file_name: self.include_stack.last().unwrap().0.clone(),
                    },
                    None,
                )],
                help: Some(format!(
                    "Valid address modes are:\n    {}",
                    OPCODES[mnemonic.val]
                        .iter()
                        .filter(|(_, opcode)| opcode.is_some())
                        .map(|(mode, _)| mode.string_rep(mnemonic.val))
                        .collect::<Vec<String>>()
                        .join("\n    ")
                )),
            });
        }

        // Insert operand bytes.
        Ok(match operand {
            ActualValue::Byte(byte) => {
                self.insert_byte(byte);
                2
            }
            ActualValue::Word(word) => {
                self.insert_word(word);
                3
            }
            ActualValue::String(_) => {
                return Err(AssemblerError {
                    message: format!(
                        "String operands are only allowed for the `dfb` and `inl` directives",
                    ),
                    labels: vec![(
                        Location {
                            span: operand_span.clone(),
                            file_name: self.include_stack.last().unwrap().0.clone(),
                        },
                        None,
                    )],
                    help: None,
                });
            }
            ActualValue::Accumulator => 1,
            ActualValue::Reference(reference) => {
                let has_modifier = reference.modifier.is_some();
                self.object[self.current_section].references.push(reference);

                if has_modifier || mnemonic.is_branch() {
                    self.insert_byte(0);
                    2
                } else {
                    self.insert_word(0);
                    3
                }
            }
        })
    }

    fn handle_implied_instruction(
        &mut self,
        mnemonic: Mnemonic,
        instruction_span: Range<usize>,
    ) -> Result<usize, AssemblerError> {
        if let Some(byte) = OPCODES[mnemonic][AddressMode::Implied] {
            self.insert_byte(byte);
            Ok(1)
        } else {
            Err(AssemblerError {
                message: format!(
                    "{} address mode is invalid for instruction `{}`",
                    AddressMode::Implied.string_rep(mnemonic),
                    mnemonic,
                ),
                labels: vec![(
                    Location {
                        span: instruction_span,
                        file_name: self.include_stack.last().unwrap().0.clone(),
                    },
                    None,
                )],
                help: Some(format!(
                    "Valid address modes are:\n    {}",
                    OPCODES[mnemonic]
                        .iter()
                        .filter(|(_, opcode)| opcode.is_some())
                        .map(|(mode, _)| mode.string_rep(mnemonic))
                        .collect::<Vec<String>>()
                        .join("\n    ")
                )),
            })
        }
    }

    fn handle_dfb(&mut self, operand: Spanned<ActualValue>) -> Result<usize, AssemblerError> {
        let operand_location = Location {
            span: operand.span.clone(),
            file_name: self.include_stack.last().unwrap().0.clone(),
        };

        Ok(match operand.val {
            ActualValue::Byte(byte) => {
                self.insert_byte(byte);
                1
            }
            ActualValue::String(string) => {
                let bytes = apple_string(&string, operand_location)?;
                for character in &bytes {
                    self.insert_byte(character.0);
                }
                bytes.len()
            }
            ActualValue::Reference(mut reference) => {
                if reference.modifier.is_some() {
                    self.insert_byte(0);
                    // When references are created it is assumed that tey go after an opcode.
                    reference.offset -= 1;
                    self.object[self.current_section].references.push(reference);
                    1
                } else {
                    return Err(AssemblerError {
                        message: String::from("The `dfb` directive expects a byte operand"),
                        labels: vec![(
                            operand_location,
                            Some(String::from("Referenced symbol is assumed to be a word")),
                        )],
                        help: None,
                    });
                }
            }
            _ => {
                return Err(AssemblerError {
                    message: String::from("The `dfb` directive expects a byte operand"),
                    labels: vec![(operand_location, None)],
                    help: None,
                });
            }
        })
    }

    fn handle_dfw(&mut self, operand: Spanned<ActualValue>) -> Result<usize, AssemblerError> {
        Ok(match operand.val {
            ActualValue::Word(word) => {
                self.insert_word(word);
                2
            }
            ActualValue::Reference(mut reference) => {
                if let Some(modifier) = &reference.modifier {
                    return Err(AssemblerError {
                        message: String::from("The `dfw` directive expects a word operand"),
                        labels: vec![(
                            Location {
                                span: modifier.span.clone(),
                                file_name: self.include_stack.last().unwrap().0.clone(),
                            },
                            Some(String::from("Modifier turns this word into a byte")),
                        )],
                        help: None,
                    });
                }
                self.insert_word(0);
                reference.offset -= 1;
                self.object[self.current_section].references.push(reference);
                2
            }
            _ => {
                return Err(AssemblerError {
                    message: String::from("The `dfw` directive expects a word operand"),
                    labels: vec![(
                        Location {
                            span: operand.span.clone(),
                            file_name: self.include_stack.last().unwrap().0.clone(),
                        },
                        None,
                    )],
                    help: None,
                });
            }
        })
    }

    fn create_macro(
        &mut self,
        operand: ActualValue,
        instruction_span: Range<usize>,
    ) -> Result<usize, AssemblerError> {
        let directive_location = Location {
            span: instruction_span.clone(),
            file_name: self.include_stack.last().unwrap().0.clone(),
        };

        // A label has to appear on the same line.
        if !self.macro_valid {
            return Err(AssemblerError {
                message: String::from("A name is not present for this macro"),
                labels: vec![(directive_location, None)],
                help: None,
            });
        }

        // Take the label from this line as the macro name.
        let macro_name = self.object[self.current_section].labels.pop().unwrap().name;

        if self.macros.contains_key(&macro_name) {
            return Err(AssemblerError {
                message: format!("The macro `{}` has already been defined", macro_name),
                labels: vec![
                    (directive_location, None),
                    (
                        match &self.macros[&macro_name] {
                            Macro::Byte(_, location) => location.clone(),
                            Macro::Word(_, location) => location.clone(),
                            Macro::String(_, location) => location.clone(),
                        },
                        Some(String::from("Already defined here")),
                    ),
                ],
                help: None,
            });
        }

        self.macros.insert(
            macro_name,
            match operand {
                ActualValue::Byte(byte) => Macro::Byte(byte, directive_location),
                ActualValue::Word(word) => Macro::Word(word, directive_location),
                ActualValue::String(string) => Macro::String(string.clone(), directive_location),
                _ => {
                    return Err(AssemblerError {
                        message: String::from("The `equ` directive expects a byte or word operand"),
                        labels: vec![(directive_location, None)],
                        help: None,
                    });
                }
            },
        );

        Ok(0)
    }

    fn change_origin(
        &mut self,
        operand: ActualValue,
        instruction_span: Range<usize>,
    ) -> Result<usize, AssemblerError> {
        let directive_location = Location {
            span: instruction_span.clone(),
            file_name: self.include_stack.last().unwrap().0.clone(),
        };

        if !self.binary {
            return Err(AssemblerError {
                message: String::from("The `org` directive cannot be used in object mode"),
                labels: vec![(directive_location, None)],
                help: None,
            });
        }

        match operand {
            ActualValue::Word(word) => {
                if word as usize > self.object[self.current_section].origin {
                    self.object[self.current_section].highest_origin = word as usize;
                }
                self.object[self.current_section].origin = word as usize;
                self.current_line_section_start = self.object[self.current_section].origin;

                if self.object[self.current_section].lowest_origin.is_none()
                    || (word as usize) < self.object[self.current_section].lowest_origin.unwrap()
                {
                    self.object[self.current_section].lowest_origin = Some(word as usize);
                }
            }
            _ => {
                return Err(AssemblerError {
                    message: String::from("The `org` directive expects a word operand"),
                    labels: vec![(directive_location, None)],
                    help: None,
                });
            }
        }

        Ok(0)
    }

    fn change_section(
        &mut self,
        operand: ActualValue,
        instruction_span: Range<usize>,
    ) -> Result<usize, AssemblerError> {
        let directive_location = Location {
            span: instruction_span.clone(),
            file_name: self.include_stack.last().unwrap().0.clone(),
        };

        if self.binary {
            return Err(AssemblerError {
                message: String::from("The `sct` directive is not allowed in binary mode"),
                labels: vec![(directive_location, None)],
                help: None,
            });
        }

        match operand {
            ActualValue::String(name) => {
                let section_index = self.object.iter().position(|section| section.name == *name);
                if let Some(index) = section_index {
                    self.current_section = index;
                } else {
                    if self.object.len() == 1 {
                        if self.object[self.current_section].origin == 0 {
                            self.object[0].name = name.clone();
                        } else {
                            return Err(AssemblerError {
                                message: String::from(
                                    "The `sct` directive must appear before any code",
                                ),
                                labels: vec![(directive_location, None)],
                                help: None,
                            });
                        }
                    } else {
                        self.current_section = self.object.len();
                        self.object.push(Section {
                            name: name.clone(),
                            data: [0; 65536],
                            origin: 0,
                            lowest_origin: None,
                            highest_origin: 0,
                            used_ranges: Vec::with_capacity(8),
                            labels: Vec::with_capacity(64),
                            references: Vec::with_capacity(128),
                        });
                    }
                }
            }
            _ => {
                return Err(AssemblerError {
                    message: String::from("The `sct` directive expects a string operand"),
                    labels: vec![(directive_location, None)],
                    help: None,
                });
            }
        }

        Ok(0)
    }

    fn create_listing_line(&mut self, line_end: usize) {
        self.listing.push(Listing {
            location: Some((
                self.current_section,
                self.current_line_section_start,
                self.current_line_section_end,
            )),
            code: String::from(
                self.files
                    .get(self.id_table[&self.include_stack.last().unwrap().0])
                    .unwrap()
                    .source()[self.current_line_source_start..line_end]
                    .trim_end(),
            ),
        });
    }

    fn handle_label(&mut self, spanned_label: Spanned<Label>) -> Result<(), ()> {
        match spanned_label.val {
            Label::Top(top_label) => {
                self.macro_valid = true;

                // Expect subsequent sublabels to go under this parent label.
                self.last_parent_label = Some((
                    Spanned::new((top_label.name.clone(), spanned_label.span)),
                    self.include_stack.last().unwrap().0.clone(),
                ));

                // And store it.
                let label_offset = self.object[self.current_section].origin;
                self.object[self.current_section].labels.push(SectionLabel {
                    name: top_label.name,
                    visibility: top_label.visibility,
                    offset: label_offset,
                });
            }
            Label::Sub((parent_label, sublabel)) => {
                // Expect a parent label to exist first.
                let current_parent = if let Some(ref parent_label) = self.last_parent_label {
                    parent_label
                } else {
                    self.errors.push(AssemblerError {
                        message: String::from("No parent label to put sublabel under"),
                        labels: vec![(
                            Location {
                                span: sublabel.span,
                                file_name: self.include_stack.last().unwrap().0.clone(),
                            },
                            None,
                        )],
                        help: Some(format!(
                            "Did you mean to create a parent label `{}`?",
                            sublabel.val,
                        )),
                    });

                    return Err(());
                };

                if let Some(parent_label) = &parent_label {
                    // Make sure explicit parent label matches acual parent.
                    if &parent_label.val != &current_parent.0.val {
                        self.errors.push(AssemblerError {
                            message: String::from(
                                "Explicit parent label does not match most recent parent label",
                            ),
                            labels: vec![
                                (
                                    Location {
                                        span: parent_label.span.clone(),
                                        file_name: self.include_stack.last().unwrap().0.clone(),
                                    },
                                    Some(format!("Found `{}`", parent_label.val)),
                                ),
                                (
                                    Location {
                                        span: current_parent.0.span.clone(),
                                        file_name: current_parent.1.clone(),
                                    },
                                    Some(format!("Expected `{}`", current_parent.0.val)),
                                ),
                            ],
                            help: Some(format!(
                                "Change `{}` to `{}`",
                                parent_label.val, current_parent.0.val
                            )),
                        });

                        return Err(());
                    } else {
                        let offset = self.object[self.current_section].origin;
                        self.object[self.current_section].labels.push(SectionLabel {
                            name: format!("{}.{}", parent_label.val, sublabel.val),
                            visibility: Visibility::Object,
                            offset,
                        });
                    }
                } else {
                    let offset = self.object[self.current_section].origin;
                    self.object[self.current_section].labels.push(SectionLabel {
                        name: format!("{}.{}", current_parent.0.val, sublabel.val),
                        visibility: Visibility::Object,
                        offset,
                    });
                }
            }
        }

        Ok(())
    }

    /// Get the bytes of an operand, resolving a reference to a macro if possible,
    /// and the modifier to a reference if it is not to a macro.
    fn resolve_operand(
        &mut self,
        operand: Spanned<Operand>,
        branch: bool,
    ) -> (Spanned<ActualValue>, AddressMode) {
        // println!("macros:\n{:?}", self.macros);
        // println!("operand {:?}\n", operand);

        let mut address_mode = operand.val.address_mode(branch);
        let span = operand.span.clone();
        let resolved_value = match &operand.value.val {
            // If it's a reference then see if it is a macro first.
            Value::Reference(symbol) => match self.macros.get(symbol) {
                // Reference's address mode is parsed as absolute, but when we resolve it
                // to a byte macro it should be zeropage instead.
                Some(Macro::Byte(byte, _)) => {
                    address_mode = match address_mode {
                        AddressMode::Absolute => AddressMode::Zeropage,
                        AddressMode::AbsoluteX => AddressMode::ZeropageX,
                        AddressMode::AbsoluteY => AddressMode::ZeropageY,
                        _ => address_mode,
                    };
                    match address_mode {
                        AddressMode::Indirect => ActualValue::Word(*byte as u16),
                        _ => ActualValue::Byte(*byte),
                    }
                }
                Some(Macro::Word(word, _)) => {
                    if let Some(spanned_modifier) = &operand.modifier {
                        address_mode = match address_mode {
                            AddressMode::Absolute => AddressMode::Zeropage,
                            AddressMode::AbsoluteX => AddressMode::ZeropageX,
                            AddressMode::AbsoluteY => AddressMode::ZeropageY,
                            _ => address_mode,
                        };
                        match spanned_modifier.val {
                            Modifier::HighByte => ActualValue::Byte((*word >> 8) as u8),
                            Modifier::LowByte => ActualValue::Byte((*word) as u8),
                        }
                    } else {
                        ActualValue::Word(*word)
                    }
                }
                Some(Macro::String(string, _)) => ActualValue::String(string.clone()),
                None => ActualValue::Reference(Reference {
                    name: symbol.clone(),
                    // The origin is the index where the opcode byte will be inserted
                    // because this function is called before doing that, so we add 1 to
                    // indicate where in the section the referenced value will go.
                    offset: self.object[self.current_section].origin + 1,
                    modifier: operand.modifier.clone(),
                    branch,
                    location: Location {
                        span: span.clone(),
                        file_name: self.include_stack.last().unwrap().0.clone(),
                    },
                }),
            },
            Value::Accumulator => ActualValue::Accumulator,
            Value::Byte(byte) => {
                if address_mode == AddressMode::Indirect {
                    ActualValue::Word(*byte as u16)
                } else {
                    ActualValue::Byte(*byte)
                }
            }
            Value::Word(word) => {
                if let Some(spanned_modifier) = &operand.modifier {
                    address_mode = match address_mode {
                        AddressMode::Absolute => AddressMode::Zeropage,
                        AddressMode::AbsoluteX => AddressMode::ZeropageX,
                        AddressMode::AbsoluteY => AddressMode::ZeropageY,
                        _ => address_mode,
                    };
                    match spanned_modifier.val {
                        Modifier::HighByte => ActualValue::Byte((*word >> 8) as u8),
                        Modifier::LowByte => ActualValue::Byte((*word) as u8),
                    }
                } else {
                    ActualValue::Word(*word)
                }
            }
            Value::String(string) => ActualValue::String(string.clone()),
            Value::Include(_) => unreachable!("Included programs are handled during parsing"),
        };

        (Spanned::new((resolved_value, span)), address_mode)
    }

    fn insert_byte(&mut self, byte: u8) {
        let offset = self.object[self.current_section].origin;
        if offset == 0 {
            self.object[self.current_section].lowest_origin = Some(0);
        }
        self.object[self.current_section].data[offset] = byte;
        self.object[self.current_section].origin += 1;

        if offset == self.object[self.current_section].highest_origin {
            self.object[self.current_section].highest_origin += 1;
        }
    }

    fn insert_word(&mut self, word: u16) {
        self.insert_byte(word as u8);
        self.insert_byte((word >> 8) as u8);
    }
}
