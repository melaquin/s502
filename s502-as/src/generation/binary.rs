//! The file output for raw binary files.

use std::fs;

use super::*;

/// Resolve each reference to a label.
fn resolve_references(section: &mut Section) -> Result<(), Vec<AssemblerError>> {
    let labels: HashMap<String, usize> = section
        .labels
        .iter()
        .map(|label| (label.name.clone(), label.offset))
        .collect();

    let mut errors = Vec::new();

    for reference in &section.references {
        let label_search = labels.get(&reference.name);
        let label = if label_search.is_none() {
            errors.push(AssemblerError {
                message: format!("Label `{}` being referenced does not exist", reference.name),
                labels: vec![(reference.location.clone(), None)],
                help: None,
            });
            continue;
        } else {
            *label_search.unwrap()
        };

        if reference.branch {
            // The target address is relative to the pc after reading the operand and incrementing.
            let offset = label as isize - (reference.offset as isize + 1);
            if offset < -128 || offset > 127 {
                errors.push(AssemblerError {
                    message: format!("The relative target is out of range, at `{}` bytes", offset),
                    labels: vec![(reference.location.clone(), None)],
                    help: None,
                });
                continue;
            }
            section.data[reference.offset] = (offset) as u8;
            continue;
        }

        match reference.modifier {
            None => {
                section.data[reference.offset] = (label) as u8;
                section.data[reference.offset + 1] = (label >> 8) as u8;
            }
            Some(Spanned {
                val: Modifier::HighByte,
                span: _,
            }) => section.data[reference.offset] = (label >> 8) as u8,
            Some(Spanned {
                val: Modifier::LowByte,
                span: _,
            }) => section.data[reference.offset] = (label) as u8,
        }
    }

    if !errors.is_empty() {
        Err(errors)
    } else {
        Ok(())
    }
}

pub fn emit_binary(
    section: &mut Section,
    output_filename: &String,
) -> Result<(), Vec<AssemblerError>> {
    let _ = resolve_references(section)?;

    fs::write(
        output_filename,
        &section.data[section.lowest_origin.unwrap_or(0)..section.highest_origin],
    )
    .map_err(|error| {
        vec![AssemblerError {
            message: format!("Error writing binary file `{}`: {}", output_filename, error),
            labels: vec![],
            help: None,
        }]
    })?;

    Ok(())
}

pub fn create_listing(
    section: &Section,
    listings: Vec<Listing>,
    file_name: String,
) -> Result<(), AssemblerError> {
    let mut listing = listings
        .into_iter()
        .map(|listing| {
            let (_, start, end) = if listing.location.is_none() {
                return listing.code;
            } else {
                listing.location.unwrap()
            };

            let mut listing_line = format!("{:04x?} ", start);
            match end - start {
                0 => listing_line.push_str("         "),
                1 => listing_line.push_str(&format!("{:02x?}       ", section.data[start])),
                2 => listing_line.push_str(&format!(
                    "{:02x?} {:02x?}    ",
                    section.data[start],
                    section.data[start + 1],
                )),
                3 => listing_line.push_str(&format!(
                    "{:02x?} {:02x?} {:02x?} ",
                    section.data[start],
                    section.data[start + 1],
                    section.data[start + 2],
                )),
                mut count_bytes => {
                    listing_line.push_str(&format!(
                        "{:02x?} {:02x?} {:02x?} {}\n{:04x?} ",
                        section.data[start],
                        section.data[start + 1],
                        section.data[start + 2],
                        listing.code,
                        start + 3,
                    ));
                    count_bytes -= 3;

                    let mut start_line = start + 3;
                    while count_bytes > 3 {
                        listing_line.push_str(&format!(
                            "{:02x?} {:02x?} {:02x?} \n{:04x?} ",
                            section.data[start_line],
                            section.data[start_line + 1],
                            section.data[start_line + 2],
                            start_line + 3
                        ));
                        count_bytes -= 3;
                        start_line += 3;
                    }

                    match count_bytes {
                        0 => {}
                        1 => listing_line
                            .push_str(&format!("{:02x?}       ", section.data[start_line])),
                        2 => listing_line.push_str(&format!(
                            "{:02x?} {:02x?}    ",
                            section.data[start_line],
                            section.data[start_line + 1],
                        )),
                        _ => listing_line.push_str(&format!(
                            "{:02x?} {:02x?} {:02x?} ",
                            section.data[start_line],
                            section.data[start_line + 1],
                            section.data[start_line + 2],
                        )),
                    }
                }
            }

            if end - start <= 3 {
                listing_line.push_str(&listing.code);
            }

            listing_line
        })
        .collect::<Vec<String>>();
    listing.push(format!(
        "              * Assembled {} bytes",
        section.highest_origin - section.lowest_origin.unwrap_or(0)
    ));
    fs::write(&file_name, listing.join("\n")).map_err(|error| AssemblerError {
        message: format!("Error writing listing file `{}`: {}", file_name, error),
        labels: vec![],
        help: None,
    })?;

    Ok(())
}

pub fn create_symbol_table(section: &Section, file_name: String) -> Result<(), AssemblerError> {
    fs::write(
        &file_name,
        section
            .labels
            .iter()
            .filter(|label| label.visibility == Visibility::Global)
            .map(|label| format!("{} equ ${:04x}", label.name, label.offset))
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .map_err(|error| AssemblerError {
        message: format!("Error writing symbol table `{}`: {}", file_name, error),
        labels: vec![],
        help: None,
    })?;

    Ok(())
}
