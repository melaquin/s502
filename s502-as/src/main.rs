use std::{fs, process};

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

mod assembler;
mod ast;
mod error;
mod lexer;

/// This assembler takes *.65a files and outputs either a binary file or object file for either.
// #[derive(Parser)]
// #[clap(author, version, about, long_about = None)]
// struct Arguments {
//     /// Output binary format `*filename*.bin` (default is object format).
//     #[clap(short)]
//     binary: bool,

//     /// Output listing `*filename*.65t` (only if outputting in binary format).
//     #[clap(short = 't')]
//     listing: bool,

//     /// Output symbol table `*filename*.65s`.
//     #[clap(short)]
//     symbol_table: bool,

//     /// Only assemble, don't link. This implies `-k` and has no effect with `-b`.
//     #[clap(short)]
//     assemble_only: bool,
// }

fn main() {
    let matches = command!()
        .arg(arg!(-o [OUTPUT] "Output name if only one source").multiple_values(false))
        .arg(arg!(<SOURCE> "h502 source files").multiple_values(true))
        .get_matches();

    // let output_filename = matches.value_of("OUTPUT");
    // guaranteed to be Some because get_matches exits if no sources are given
    let file_names = unsafe { matches.values_of("SOURCE").unwrap_unchecked() }
        .map(|name| name.to_string())
        .collect::<Vec<String>>();

    let mut files = SimpleFiles::<String, String>::new();
    let stderr_writer = StandardStream::stderr(ColorChoice::Always);
    let codespan_config = Config::default();

    for file_name in file_names {
        let source = fs::read_to_string(&file_name);
        let source = match source {
            Err(error) => {
                println!("{error}");
                process::exit(1);
            }
            Ok(source) => source,
        };

        let (file_id, errors) = assembler::Assembler::parse_file(file_name, source, &mut files);
        if !errors.is_empty() {
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
        // TODO else emit object, symbol table, and listing
    }
}
