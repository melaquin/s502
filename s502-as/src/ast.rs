use std::ops::Range;

/// An AST node with associated span in the source file.
/// This does not contain a Location because this will
/// be used by the generation stage which will get the
/// current file's ID from Item::PushInclude, while
/// Location is used in reporting errors which prevents
/// the generation stage from running.
#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    /// The node value.
    pub val: T,
    /// Where the node is in the source.
    pub span: Range<usize>,
}

impl<T> Spanned<T> {
    pub fn new(data: (T, Range<usize>)) -> Self {
        Self {
            val: data.0,
            span: data.1,
        }
    }
}

/// Convenience for easily working with the node.
impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

/// Convenience for easily working with the node.
impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

/// Information for retrieving an except from the
/// source code when reporting a parsing error.
#[derive(Debug, PartialEq)]
pub struct Location {
    /// Where in the source to point to.
    pub span: Range<usize>,
    /// Which file. This is used as a map key
    /// to find the corresponding file ID.
    pub file_name: String,
}

/// A file inclusion encountered while parsing.
#[derive(PartialEq)]
pub struct Include {
    /// The file being included as it appears in the source.
    pub included: String,
    /// Where in the source the file was included.
    pub loc: Location,
}

/// Representation of an assembly file.
pub type Program = Vec<Action>;

/// Something for the code generator to do.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// The index into the source where a line starts.
    /// This is used in combination with LineEnd to
    /// get the source code for generating listings.
    LineStart(usize),
    /// The index into the source where a line ends.
    LineEnd(usize),
    /// Tell the assembler stage to mark
    /// the current address with a label.
    Label(Spanned<Label>),
    /// Either a directive or instruction.
    Instruction(Spanned<Instruction>),
    /// The parser read in an included file with the
    /// given ID, and the following Items belong to it.
    PushInclude(usize),
    // The most recently included file has ended, go back up a level.
    PopInclude,
}

/// A label that appears at the beginning of a line.
#[derive(Debug, PartialEq)]
pub enum Label {
    Top(TopLabel),
    Sub(String),
}

/// Top level label of the line.
#[derive(Debug, PartialEq)]
pub struct TopLabel {
    pub name: String,
    pub visibility: Visibility,
    pub sublabels: Vec<String>,
}

/// A toplevel label can be exposed to other objects or kept
/// to only the compilation unit in which it appears.
#[derive(Debug, PartialEq)]
pub enum Visibility {
    Global,
    Object,
}

/// A directive or CPU instruction.
#[derive(Debug, PartialEq)]
pub struct Instruction {
    /// What the assembler or CPU should do.
    pub mnemonic: Spanned<Mnemonic>,
    /// The operand that the mnemonic may require.
    pub operand: Option<Spanned<Operand>>,
}

/// An assembler or CPU instruction to execute.
#[derive(Clone, Debug, PartialEq)]
pub enum Mnemonic {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
    Dfb,
    Dfw,
    Equ,
    Hlt,
    Inl,
    Org,
    Sct,
}

impl Mnemonic {
    pub fn is_implied(&self) -> bool {
        self == &Mnemonic::Brk
            || self == &Mnemonic::Clc
            || self == &Mnemonic::Cld
            || self == &Mnemonic::Cli
            || self == &Mnemonic::Clv
            || self == &Mnemonic::Dex
            || self == &Mnemonic::Dey
            || self == &Mnemonic::Inx
            || self == &Mnemonic::Iny
            || self == &Mnemonic::Nop
            || self == &Mnemonic::Pha
            || self == &Mnemonic::Php
            || self == &Mnemonic::Pla
            || self == &Mnemonic::Plp
            || self == &Mnemonic::Rti
            || self == &Mnemonic::Rts
            || self == &Mnemonic::Sec
            || self == &Mnemonic::Sed
            || self == &Mnemonic::Sei
            || self == &Mnemonic::Tax
            || self == &Mnemonic::Tay
            || self == &Mnemonic::Tsx
            || self == &Mnemonic::Txa
            || self == &Mnemonic::Txs
            || self == &Mnemonic::Tya
    }
}

/// The parsed instruction operand.
#[derive(Debug, PartialEq)]
pub struct Operand {
    pub mode: OperandMode,
    pub modifier: Option<Spanned<Modifier>>,
    pub value: Spanned<Value>,
}

/// An appproximation of the operand's address mode.
/// The exact mode cannot be parsed because references
/// may be to bytes or words which makes the operand
/// ambiguous.
#[derive(Debug, PartialEq)]
pub enum OperandMode {
    /// The A register is being used.
    Accumulator,
    /// Some address is being used. This covers
    /// absolute, zeropage, and relative, but which
    /// one it is is not known until generation
    /// when macros and labels are resolved.
    Address,
    /// X is added to an address. This covers
    /// abs,x and zpg,x, but which
    /// one it is is not known until generation
    /// when macros and labels are resolved.
    XIndexed,
    /// Y is added to an address. This covers
    /// abs,y and zpg,y, but which
    /// one it is is not known until generation
    /// when macros and labels are resolved.
    YIndexed,
    /// The operand is an immediate number.
    Immediate,
    /// The operand dereferences an absolute address.
    Indirect,
    /// The operand adds X to a zeropage address
    /// without carry and dereferences the word
    /// at that address.
    XIndirect,
    /// The operand dereferences the word at the
    /// zeropage address and adds Y to it with carry.
    IndirectY,
}

/// A modifier to a word value.
#[derive(Debug, PartialEq)]
pub enum Modifier {
    /// Take the high byte of that word.
    HighByte,
    /// Take the low byte of that word.
    LowByte,
}

/// The value to be modified and used by the operand.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// The accumulator is the value to be used.
    /// This is only used with the Accumulator address mode.
    Accumulator,
    /// The value is a literal byte.
    Byte(u8),
    /// The value is a literal word.
    Word(u16),
    /// The value is a literal strig.
    String(String),
    /// The value is a reference to a macro or label.
    Reference(String),
}
