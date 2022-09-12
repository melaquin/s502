#[cfg(fuzzing)]
extern crate afl;
#[macro_use]
extern crate indoc;

use std::{collections::HashMap, fs, path::Path};

use ast::{Include, Location};
use clap::{arg, command};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};

mod ast;
mod error;
mod parser;

/// The normal entry point for running.
#[cfg(not(fuzzing))]
fn main() {
    let matches = command!()
        .about("An assembler for the MOS 6502")
        .arg(
            arg!(-b --binary "Output a binary instead of an object").long_help(indoc! {
                "Output a raw binary file instead of an object.
                
                In binary mode, label addresses and reference are resolved right away,
                and addresses are set with the `org` directive. Otherwise, the code is
                organized in sections with the `sct` directive and the linker resolves
                addresses using a linker script."
            }),
        )
        .arg(
            arg!(-s --symbol "Output a symbol table for each source").long_help(indoc! {
                "Output a symbol table for each source.
                
                If -b is set, then a <source_file_name>_symbols.65a will be created that
                contains macros associating exported labels with their addresses in memory.
                Otherwise, then a <source_file_name>.65s file will be created that contains
                the exported labels and their offsets into the section that they belong to.
                
                See the `-o` option for details on specifying the file name."
            }),
        )
        .arg(
            arg!(-o [OUTPUT] "Output file name")
                .multiple_values(false)
                .long_help(indoc! {
                "Specify the output file name.
                
                If no file extension is present then one will be added to the specified
                name: `.bin` for a binary file, `.65o` for an object file, and `.65s` for
                a symbol table Otherwise, it will be used only for the output binary or
                object file, and the default scheme wil be used for the symbol table."
                }),
        )
        .arg(arg!(<SOURCES> "h502 source and symbol table files").multiple_values(true))
        .get_matches();

    // Transform sources into `String`s and partition out source code files.
    // TODO handle other names later, partition symbol tables from unrecognized
    let (source_names, _other_names): (Vec<_>, Vec<_>) = matches
        .values_of("SOURCES")
        .unwrap()
        .map(|name| name.to_string())
        .partition(|name| {
            Path::new(name)
                .extension()
                .filter(|&extension| extension == "65a")
                .is_some()
        });

    // This takes file IDs and spans to fetch excerpts from source code in error reporting.
    let mut files = SimpleFiles::<String, String>::new();
    let stderr_writer = StandardStream::stderr(ColorChoice::Always);
    let codespan_config = Config::default();

    if source_names.len() == 0 {
        let diagnostic = Diagnostic::<usize>::error()
            .with_message("Expected at least one .65a source file".to_string());
        let _ = term::emit(
            &mut stderr_writer.lock(),
            &codespan_config,
            &files,
            &diagnostic,
        );
    }

    // TODO spawn a thread that does all of this for parallel compilation, deal with mutex to terminal
    for file_name in source_names {
        // Skip the sources that couldn't be read because they're separate compilation units.
        let source = fs::read_to_string(&file_name);
        let source = match source {
            Err(error) => {
                let diagnostic = Diagnostic::<usize>::error()
                    .with_message(format!("Could not read {}: {}", file_name, error));
                let _ = term::emit(
                    &mut stderr_writer.lock(),
                    &codespan_config,
                    &files,
                    &diagnostic,
                );
                continue;
            }
            Ok(source) => source,
        };

        // Build a context for the parser.

        // Stack of included files used to prevent recursion.
        // Start with an entry including the top level file and say the command line
        // included it. If a file tries to include it then it will get <command line>
        // when finding out who already included it.
        let mut include_stack = vec![Include {
            included: file_name.clone(),
            loc: Location {
                span: 0..1,
                file_name: "<command line>".to_string(),
            },
        }];

        // Table associating file names with their file IDs.
        let mut id_table = HashMap::<String, usize>::new();

        let parser_context = parser::ParserContext::new(
            file_name.clone(),
            &source,
            &mut files,
            &mut include_stack,
            &mut id_table,
        );

        let program_result = parser_context.parse_program();
        // Don't put a duplicate source in the files.
        let file_id = if id_table.contains_key(&file_name) {
            id_table[&file_name]
        } else {
            files.add(file_name.clone(), source)
        };

        // Insert the toplevel file now after getting its ID.
        id_table.insert(file_name, file_id);
        // And make command line point to it as well because it's used as the key when
        // looking up who included the top level file..
        id_table.insert("<command line>".to_string(), file_id);

        match program_result {
            // Report errors if there are any.
            Err(errors) => {
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

                    let _ = term::emit(
                        &mut stderr_writer.lock(),
                        &codespan_config,
                        &files,
                        &diagnostic,
                    );

                    // And create a second help diagnostic if one was given.
                    if let Some(note) = error.help {
                        let diagnostic = Diagnostic::help().with_message(note);

                        let _ = term::emit(
                            &mut stderr_writer.lock(),
                            &codespan_config,
                            &files,
                            &diagnostic,
                        );
                    }
                }
            }
            Ok(_program) => {
                // TODO emit object/binary, symbol table, and listing
            }
        }
    }
}

/// The entry point used for fuzzing with `afl.rs`.
#[cfg(fuzzing)]
fn main() {
    use afl::fuzz;

    fuzz!(|data: &[u8]| {
        // Create context for parsing.

        let file_name = String::from("AFL input");
        let source = match String::from_utf8(data.to_vec()) {
            Err(_) => return,
            Ok(string) => string,
        };
        let mut files = SimpleFiles::<String, String>::new();
        let mut include_stack = vec![Include {
            included: file_name.clone(),
            loc: Location {
                span: 0..1,
                file_name: "<command line>".to_string(),
            },
        }];

        let mut id_table = HashMap::<String, usize>::new();

        let parser_context = parser::ParserContext::new(
            file_name.clone(),
            &source,
            &mut files,
            &mut include_stack,
            &mut id_table,
        );

        // Fuzz the parser.
        let program_result = parser_context.parse_program();
    })
}
