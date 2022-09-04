//! Lexer that recognizes tokens in the assembly code.

use std::fmt;

use logos::Logos;

/// Tokens of the assembly language.
#[derive(Clone, Logos)]
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
    #[regex("\"\\w*\"", |lex| lex.slice()[1..lex.slice().len()-1].to_string())]
    String(String),
    #[regex("[a-zA-Z][a-zA-Z_]*", |lex| lex.slice().to_string())]
    Ident(String),
    #[regex("\n")]
    Eol,
    #[error]
    #[regex("[*].*\n")]
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
            Self::Global => write!(f, "`!!`"),
            Self::Period => write!(f, "`.`"),
            Self::String(string) => write!(f, "`\"{string}\"`"),
            Self::Ident(ident) => write!(f, "`{ident}`"),
            Self::Eol => write!(f, "`<end of line>`"),
            Self::Error => write!(f, "`ERROR`"),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
