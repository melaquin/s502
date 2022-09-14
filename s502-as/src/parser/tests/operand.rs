use std::collections::HashMap;

use codespan_reporting::files::SimpleFiles;

use crate::{ast::*, error::AssemblerError, parser::ParserContext};

#[test]
fn accumlator() {
    let source = "A".to_string();
    let source_name = "accumulator operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Accumulator,
                modifier: None,
                value: Spanned::new((Value::Accumulator, 0..1))
            },
            0..1
        ))))
    );
}

#[test]
fn literal() {
    let source = "3".to_string();
    let source_name = "literal operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: None,
                value: Spanned::new((Value::Byte(3), 0..1))
            },
            0..1
        ))))
    );
}

#[test]
fn string() {
    let source = "\"test string\"".to_string();
    let source_name = "string operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: None,
                value: Spanned::new((Value::String(String::from("test string")), 0..13))
            },
            0..13
        ))))
    );
}

#[test]
fn high_string() {
    let source = "<\"ab\"".to_string();
    let source_name = "high string operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: Some(Spanned::new((Modifier::HighByte, 0..1))),
                value: Spanned::new((Value::String(String::from("ab")), 1..5))
            },
            0..5
        ))))
    );
}

#[test]
fn identfifier() {
    let source = "alabel".to_string();
    let source_name = "identifier operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: None,
                value: Spanned::new((Value::Reference(String::from("alabel")), 0..6))
            },
            0..6
        ))))
    );
}

#[test]
fn low_literal() {
    let source = ">2".to_string();
    let source_name = "low literal operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: Some(Spanned::new((Modifier::LowByte, 0..1))),
                value: Spanned::new((Value::Byte(2), 1..2))
            },
            0..2
        ))))
    );
}

#[test]
fn high_identifier() {
    let source = "<mylabel".to_string();
    let source_name = "high identifier operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: Some(Spanned::new((Modifier::HighByte, 0..1))),
                value: Spanned::new((Value::Reference(String::from("mylabel")), 1..8))
            },
            0..8
        ))))
    );
}

#[test]
fn immediate() {
    let source = "#$2F".to_string();
    let source_name = "immediate operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Immediate,
                modifier: None,
                value: Spanned::new((Value::Byte(0x2F), 1..4))
            },
            0..4
        ))))
    );
}

#[test]
fn low_indirect() {
    let source = "(>$1000)".to_string();
    let source_name = "indirect operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Indirect,
                modifier: Some(Spanned::new((Modifier::LowByte, 1..2))),
                value: Spanned::new((Value::Word(0x1000), 2..7))
            },
            0..8
        ))))
    );
}

#[test]
fn high_x_indirect() {
    let source = "(<$148F,x)".to_string();
    let source_name = "x indexed indirect operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::XIndirect,
                modifier: Some(Spanned::new((Modifier::HighByte, 1..2))),
                value: Spanned::new((Value::Word(0x148F), 2..7))
            },
            0..10
        ))))
    );
}

#[test]
fn indirect_y() {
    let source = "(17),y".to_string();
    let source_name = "indirect y indexed operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::IndirectY,
                modifier: None,
                value: Spanned::new((Value::Byte(17), 1..3))
            },
            0..6
        ))))
    );
}

#[test]
fn x_indexed() {
    let source = "$2525,x".to_string();
    let source_name = "absolute x indexed operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::XIndexed,
                modifier: None,
                value: Spanned::new((Value::Word(0x2525), 0..5))
            },
            0..7
        ))))
    );
}

#[test]
fn y_indexed() {
    let source = "$2526,y".to_string();
    let source_name = "absolute y indexed operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::YIndexed,
                modifier: None,
                value: Spanned::new((Value::Word(0x2526), 0..5))
            },
            0..7
        ))))
    );
}

#[test]
fn missing_comma_ind_y() {
    let source = "(2),".to_string();
    let source_name = "missing comma indirect y operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Err(AssemblerError {
            message: String::from("Expected `Y` after `,`"),
            labels: vec![(
                Location {
                    span: 3..4,
                    file_name: String::from("missing comma indirect y operand test"),
                },
                None
            )],
            help: None
        })
    );
}

#[test]
fn missing_rparen_x_ind() {
    let source = "(2,x".to_string();
    let source_name = "missing rparen x indirect operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Err(AssemblerError {
            message: String::from("Expected `)` after `X`"),
            labels: vec![(
                Location {
                    span: 3..4,
                    file_name: String::from("missing rparen x indirect operand test"),
                },
                None
            )],
            help: None
        })
    );
}

#[test]
fn missing_x_x_ind() {
    let source = "(2,".to_string();
    let source_name = "missing x x indirect operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Err(AssemblerError {
            message: String::from("Expected `X` after `,`"),
            labels: vec![(
                Location {
                    span: 2..3,
                    file_name: String::from("missing x x indirect operand test"),
                },
                None
            )],
            help: None
        })
    );
}

#[test]
fn indirect_no_rparen_comma() {
    let source = "(2".to_string();
    let source_name = "indirect no rparen comma operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Err(AssemblerError {
            message: String::from("Expected `)` or `,` after value"),
            labels: vec![(
                Location {
                    span: 1..2,
                    file_name: String::from("indirect no rparen comma operand test"),
                },
                None
            )],
            help: None
        })
    );
}

#[test]
fn index_no_x_y() {
    let source = "$2000,".to_string();
    let source_name = "indexed no x y operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Err(AssemblerError {
            message: String::from("Expected `X` or `Y` after `,`"),
            labels: vec![(
                Location {
                    span: 5..6,
                    file_name: String::from("indexed no x y operand test"),
                },
                None
            )],
            help: None
        })
    );
}

#[test]
fn missing_value_modifier() {
    let source = "(<)".to_string();
    let source_name = "missing value after modifier operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Err(AssemblerError {
            message: String::from("Expected value after modifier"),
            labels: vec![(
                Location {
                    span: 1..2,
                    file_name: String::from("missing value after modifier operand test"),
                },
                None
            )],
            help: None
        })
    );
}

#[test]
fn address_eol() {
    let source = "$2010\n".to_string();
    let source_name = "address eol operand test".to_string();
    let mut files = SimpleFiles::<String, String>::new();
    let mut include_stack = vec![Include {
        included: source_name.clone(),
        loc: Location {
            span: 0..1,
            file_name: "<test harness>".to_string(),
        },
    }];
    let mut id_table = HashMap::<String, usize>::new();

    let mut parser_context = ParserContext::new(
        source_name.clone(),
        &source,
        &mut files,
        &mut include_stack,
        &mut id_table,
    );

    let operand = parser_context.parse_operand();
    assert_eq!(
        operand,
        Ok(Some(Spanned::new((
            Operand {
                mode: OperandMode::Address,
                modifier: None,
                value: Spanned::new((Value::Word(0x2010), 0..5))
            },
            0..5
        ))))
    );
}
