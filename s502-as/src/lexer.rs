//! Lexer that recognizes tokens in the assembly code.

use std::fmt;

use logos::{Lexer, Logos};

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Byte(u8),
    Word(u16),
    String(String),
}

/// Parse a number literal using the radix prefix.
fn lex_number(lex: &mut Lexer<Token>) -> Option<Literal> {
    // SAFETY [0] and the [1..] on non-base 10 slices will not panic because
    // this function is only called when the slice contains 1 or more digits.
    let base = match lex.slice().as_bytes()[0] {
        b'%' => 2,
        b'@' => 8,
        b'$' => 16,
        _ => 10,
    };
    let number_string = if base == 10 {
        &lex.slice()
    } else {
        &lex.slice()[1..]
    };

    // numbers that fit into a byte can be padded with 0s to take a word.
    let is_word = match base {
        2 => {
            if number_string.len() > 16 {
                return None;
            } else {
                number_string.len() > 8
            }
        }
        8 => {
            if number_string.len() > 6 {
                return None;
            } else {
                number_string.len() > 3
            }
        }
        10 => {
            if number_string.len() > 5 {
                return None;
            } else {
                number_string.len() > 3
            }
        }
        _ => {
            if number_string.len() > 4 {
                return None;
            } else {
                number_string.len() > 2
            }
        }
    };

    u16::from_str_radix(number_string, base)
        .map(|number| {
            if number > 255 || is_word {
                Literal::Word(number)
            } else {
                Literal::Byte(number as u8)
            }
        })
        .ok()
}

/// Tokens of the assembly language.
#[derive(Clone, Debug, Logos, PartialEq)]
pub enum Token {
    #[token("adc", priority = 2, ignore(case))]
    Adc,
    #[token("and", priority = 2, ignore(case))]
    And,
    #[token("asl", priority = 2, ignore(case))]
    Asl,
    #[token("bcc", priority = 2, ignore(case))]
    Bcc,
    #[token("bcs", priority = 2, ignore(case))]
    Bcs,
    #[token("beq", priority = 2, ignore(case))]
    Beq,
    #[token("bit", priority = 2, ignore(case))]
    Bit,
    #[token("bmi", priority = 2, ignore(case))]
    Bmi,
    #[token("bne", priority = 2, ignore(case))]
    Bne,
    #[token("bpl", priority = 2, ignore(case))]
    Bpl,
    #[token("brk", priority = 2, ignore(case))]
    Brk,
    #[token("bvc", priority = 2, ignore(case))]
    Bvc,
    #[token("bvs", priority = 2, ignore(case))]
    Bvs,
    #[token("clc", priority = 2, ignore(case))]
    Clc,
    #[token("cld", priority = 2, ignore(case))]
    Cld,
    #[token("cli", priority = 2, ignore(case))]
    Cli,
    #[token("clv", priority = 2, ignore(case))]
    Clv,
    #[token("cmp", priority = 2, ignore(case))]
    Cmp,
    #[token("cpx", priority = 2, ignore(case))]
    Cpx,
    #[token("cpy", priority = 2, ignore(case))]
    Cpy,
    #[token("dec", priority = 2, ignore(case))]
    Dec,
    #[token("dex", priority = 2, ignore(case))]
    Dex,
    #[token("dey", priority = 2, ignore(case))]
    Dey,
    #[token("eor", priority = 2, ignore(case))]
    Eor,
    #[token("inc", priority = 2, ignore(case))]
    Inc,
    #[token("inx", priority = 2, ignore(case))]
    Inx,
    #[token("iny", priority = 2, ignore(case))]
    Iny,
    #[token("jmp", priority = 2, ignore(case))]
    Jmp,
    #[token("jsr", priority = 2, ignore(case))]
    Jsr,
    #[token("lda", priority = 2, ignore(case))]
    Lda,
    #[token("ldx", priority = 2, ignore(case))]
    Ldx,
    #[token("ldy", priority = 2, ignore(case))]
    Ldy,
    #[token("lsr", priority = 2, ignore(case))]
    Lsr,
    #[token("nop", priority = 2, ignore(case))]
    Nop,
    #[token("ora", priority = 2, ignore(case))]
    Ora,
    #[token("pha", priority = 2, ignore(case))]
    Pha,
    #[token("php", priority = 2, ignore(case))]
    Php,
    #[token("pla", priority = 2, ignore(case))]
    Pla,
    #[token("plp", priority = 2, ignore(case))]
    Plp,
    #[token("rol", priority = 2, ignore(case))]
    Rol,
    #[token("ror", priority = 2, ignore(case))]
    Ror,
    #[token("rti", priority = 2, ignore(case))]
    Rti,
    #[token("rts", priority = 2, ignore(case))]
    Rts,
    #[token("sbc", priority = 2, ignore(case))]
    Sbc,
    #[token("sec", priority = 2, ignore(case))]
    Sec,
    #[token("sed", priority = 2, ignore(case))]
    Sed,
    #[token("sei", priority = 2, ignore(case))]
    Sei,
    #[token("sta", priority = 2, ignore(case))]
    Sta,
    #[token("stx", priority = 2, ignore(case))]
    Stx,
    #[token("sty", priority = 2, ignore(case))]
    Sty,
    #[token("tax", priority = 2, ignore(case))]
    Tax,
    #[token("tay", priority = 2, ignore(case))]
    Tay,
    #[token("tsx", priority = 2, ignore(case))]
    Tsx,
    #[token("txa", priority = 2, ignore(case))]
    Txa,
    #[token("txs", priority = 2, ignore(case))]
    Txs,
    #[token("tya", priority = 2, ignore(case))]
    Tya,
    #[token("dfb", priority = 2, ignore(case))]
    Dfb,
    #[token("dfw", priority = 2, ignore(case))]
    Dfw,
    #[token("equ", priority = 2, ignore(case))]
    Equ,
    #[token("hlt", priority = 2, ignore(case))]
    Hlt,
    #[token("inl", priority = 2, ignore(case))]
    Inl,
    #[token("org", priority = 2, ignore(case))]
    Org,
    #[token("sct", priority = 2, ignore(case))]
    Sct,
    #[token("txt", priority = 2, ignore(case))]
    Txt,
    #[token("a", priority = 2, ignore(case))]
    A,
    #[token("x", priority = 2, ignore(case))]
    X,
    #[token("y", priority = 2, ignore(case))]
    Y,
    #[token("!")]
    Global,
    #[token(".")]
    Period,
    #[regex("\\$[0-9a-fA-F]+", lex_number)]
    #[regex("%[0-1]+", lex_number)]
    #[regex("@[0-7]+", lex_number)]
    #[regex("[0-9]+", lex_number)]
    #[regex("\"\\w*\"", |lex| Literal::String(lex.slice()[1..lex.slice().len()-1].to_string()))]
    Literal(Literal),
    #[regex("[a-zA-Z][a-zA-Z_]*", |lex| lex.slice().to_string())]
    Ident(String),
    #[regex("\n")]
    Eol,
    #[error]
    #[regex("[*].*\n", logos::skip)]
    #[regex(r"[ \t]+", logos::skip)]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adc => write!(f, "`adc`"),
            Self::And => write!(f, "`and`"),
            Self::Asl => write!(f, "`asl`"),
            Self::Bcc => write!(f, "`bcc`"),
            Self::Bcs => write!(f, "`bcs`"),
            Self::Beq => write!(f, "`beq`"),
            Self::Bit => write!(f, "`bit`"),
            Self::Bmi => write!(f, "`bmi`"),
            Self::Bne => write!(f, "`bne`"),
            Self::Bpl => write!(f, "`bpl`"),
            Self::Brk => write!(f, "`brk`"),
            Self::Bvc => write!(f, "`bvc`"),
            Self::Bvs => write!(f, "`bvs`"),
            Self::Clc => write!(f, "`clc`"),
            Self::Cld => write!(f, "`cld`"),
            Self::Cli => write!(f, "`cli`"),
            Self::Clv => write!(f, "`clv`"),
            Self::Cmp => write!(f, "`cmp`"),
            Self::Cpx => write!(f, "`cpx`"),
            Self::Cpy => write!(f, "`cpy`"),
            Self::Dec => write!(f, "`dec`"),
            Self::Dex => write!(f, "`dex`"),
            Self::Dey => write!(f, "`dey`"),
            Self::Eor => write!(f, "`eor`"),
            Self::Inc => write!(f, "`inc`"),
            Self::Inx => write!(f, "`inx`"),
            Self::Iny => write!(f, "`iny`"),
            Self::Jmp => write!(f, "`jmp`"),
            Self::Jsr => write!(f, "`jsr`"),
            Self::Lda => write!(f, "`lda`"),
            Self::Ldx => write!(f, "`ldx`"),
            Self::Ldy => write!(f, "`ldy`"),
            Self::Lsr => write!(f, "`lsr`"),
            Self::Nop => write!(f, "`nop`"),
            Self::Ora => write!(f, "`ora`"),
            Self::Pha => write!(f, "`pha`"),
            Self::Php => write!(f, "`php`"),
            Self::Pla => write!(f, "`pla`"),
            Self::Plp => write!(f, "`plp`"),
            Self::Rol => write!(f, "`rol`"),
            Self::Ror => write!(f, "`ror`"),
            Self::Rti => write!(f, "`rti`"),
            Self::Rts => write!(f, "`rts`"),
            Self::Sbc => write!(f, "`sbc`"),
            Self::Sec => write!(f, "`sec`"),
            Self::Sed => write!(f, "`sed`"),
            Self::Sei => write!(f, "`sei`"),
            Self::Sta => write!(f, "`sta`"),
            Self::Stx => write!(f, "`stx`"),
            Self::Sty => write!(f, "`sty`"),
            Self::Tax => write!(f, "`tax`"),
            Self::Tay => write!(f, "`tay`"),
            Self::Tsx => write!(f, "`tsx`"),
            Self::Txa => write!(f, "`txa`"),
            Self::Txs => write!(f, "`txs`"),
            Self::Tya => write!(f, "`tya`"),
            Self::Dfb => write!(f, "`dfb`"),
            Self::Dfw => write!(f, "`dfw`"),
            Self::Equ => write!(f, "`equ`"),
            Self::Hlt => write!(f, "`hlt`"),
            Self::Inl => write!(f, "`inl`"),
            Self::Org => write!(f, "`org`"),
            Self::Sct => write!(f, "`sct`"),
            Self::Txt => write!(f, "`txt`"),
            Self::A => write!(f, "`A`"),
            Self::X => write!(f, "`X`"),
            Self::Y => write!(f, "`Y`"),
            Self::Global => write!(f, "`!`"),
            Self::Period => write!(f, "`.`"),
            Self::Literal(op) => match op {
                Literal::Byte(byte) => write!(f, "byte `{}`", byte),
                Literal::Word(word) => write!(f, "word `{}`", word),
                Literal::String(string) => write!(f, "`\"{string}\"`"),
            },
            Self::Ident(ident) => write!(f, "`{ident}`"),
            Self::Eol => write!(f, "`<end of line>`"),
            Self::Error => write!(f, "`ERROR`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_binary_numbers() {
        let source = "%0 %10101 %000000111 %1011000010001111".to_string();
        let mut lexer = Token::lexer(&source);

        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0b0)));
        assert_eq!(
            lexer.next().unwrap(),
            Token::Literal(Literal::Byte(0b10101))
        );
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(0b111)));
        assert_eq!(
            lexer.next().unwrap(),
            Token::Literal(Literal::Word(0b1011_0000_1000_1111))
        );
    }

    #[test]
    fn lexes_octal_numbers() {
        let source = "@0 @273 @00101 @473".to_string();
        let mut lexer = Token::lexer(&source);

        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0o0)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0o273)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(0o101)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(0o473)));
    }

    #[test]
    fn lexes_decimal_numbers() {
        let source = "0 2 255 256 65535".to_string();
        let mut lexer = Token::lexer(&source);

        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(2)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(255)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(256)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(65535)));
    }

    #[test]
    fn lexes_hex_numbers() {
        let source = "$0 $3D $7F $101 $beef".to_string();
        let mut lexer = Token::lexer(&source);

        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0x3D)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Byte(0x7F)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(0x101)));
        assert_eq!(lexer.next().unwrap(), Token::Literal(Literal::Word(0xbeef)));
    }

    #[test]
    fn rejects_bad_numbers() {
        let source = "%11000100010001000 @401201 000000 $2D3C8".to_string();
        let mut lexer = Token::lexer(&source);

        assert!(lexer.all(|token| token == Token::Error));
    }

    #[test]
    fn lexes_string() {
        let source = "\"test\"".to_string();
        let mut lexer = Token::lexer(&source);

        assert_eq!(
            lexer.next().unwrap(),
            Token::Literal(Literal::String("test".to_string()))
        );
    }
}
