#[macro_use]
extern crate indoc;

use std::fs;

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
mod lexer;
mod parser;

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
        .partition(|name| name.ends_with(".65a"));

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

        let (file_id, program_result) = parser::parse_program(file_name, source, &mut files);
        match program_result {
            // Report errors if there are any.
            Err(errors) => {
                for error in errors {
                    let mut label = Label::primary(file_id, error.span);
                    if let Some(note) = error.note {
                        label = label.with_message(note);
                    }
                    let diagnostic = Diagnostic::error()
                        .with_message(error.message)
                        .with_labels(vec![label]);

                    let _ = term::emit(
                        &mut stderr_writer.lock(),
                        &codespan_config,
                        &files,
                        &diagnostic,
                    );
                }
            }
            Ok(_program) => {
                // TODO emit object/binary, symbol table, and listing
            }
        }
    }
}
