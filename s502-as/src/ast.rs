use std::{fmt, ops::Range};

use enum_map::{enum_map, Enum, EnumMap};
use lazy_static::lazy_static;
use phf::phf_map;

use crate::error::AssemblerError;

use AddressMode::*;
use Mnemonic::*;

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

/// Convenience for easily working with the contained data.
impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

/// Information for retrieving an except from the
/// source code when reporting a parsing error.
#[derive(Clone, Debug, PartialEq)]
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

/// An instruction for the code generator perform.
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
    /// given name, and the following `Action`s happen in it.
    PushInclude(String),
    // The most recently included file has ended, go back up a level.
    PopInclude,
}

/// A label that appears at the beginning of a line.
#[derive(Debug, PartialEq)]
pub enum Label {
    Top(TopLabel),
    /// The first String is the optional explicit parent,
    /// and the second is the sublabel.
    Sub((Option<Spanned<String>>, Spanned<String>)),
}

/// Top level label of the line.
#[derive(Debug, PartialEq)]
pub struct TopLabel {
    pub name: String,
    pub visibility: Visibility,
}

/// A toplevel label can be exposed to other objects or kept
/// to only the compilation unit in which it appears.
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, Enum, PartialEq)]
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
    #[cfg(not(tarpaulin_include))]
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

    pub fn is_branch(&self) -> bool {
        self == &Mnemonic::Bcc
            || self == &Mnemonic::Bcs
            || self == &Mnemonic::Beq
            || self == &Mnemonic::Bmi
            || self == &Mnemonic::Bne
            || self == &Mnemonic::Bpl
            || self == &Mnemonic::Bvc
            || self == &Mnemonic::Bvs
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).as_str().to_lowercase())
    }
}

/// The parsed instruction operand.
#[derive(Debug, PartialEq)]
pub struct Operand {
    pub mode: OperandMode,
    pub modifier: Option<Spanned<Modifier>>,
    pub value: Spanned<Value>,
}

impl Operand {
    /// Translate the OperandMode and Value combination into an address mode.
    /// This will be used to get the opcode of the instruction.
    pub fn address_mode(&self, branch: bool) -> AddressMode {
        match (&self.mode, &self.value.val) {
            (OperandMode::Accumulator, _) => AddressMode::Accumulator,
            (OperandMode::Address, Value::Byte(_)) => AddressMode::Zeropage,
            (OperandMode::Address, _) => {
                if branch {
                    // Treat zeropage as relative.
                    AddressMode::Zeropage
                } else {
                    AddressMode::Absolute
                }
            }
            (OperandMode::XIndexed, Value::Byte(_)) => AddressMode::ZeropageX,
            (OperandMode::XIndexed, _) => AddressMode::AbsoluteX,
            (OperandMode::YIndexed, Value::Byte(_)) => AddressMode::ZeropageY,
            (OperandMode::YIndexed, _) => AddressMode::AbsoluteY,
            (OperandMode::Immediate, _) => AddressMode::Immediate,
            (OperandMode::Indirect, _) => AddressMode::Indirect,
            (OperandMode::XIndirect, _) => AddressMode::XIndirect,
            (OperandMode::IndirectY, _) => AddressMode::IndirectY,
        }
    }
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
#[derive(Clone, Debug, PartialEq)]
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
    /// An included program.
    Include((String, Program)),
}

/// The address mode of an operand. Relative is missing
/// because its syntax is the same as Zeropage and
/// this is constructed from lexing.
#[derive(Clone, Copy, Debug, Enum, PartialEq)]
pub enum AddressMode {
    /// Accumulator.
    Accumulator,
    /// Absolute.
    Absolute,
    /// Absolute, X-indexed.
    AbsoluteX,
    /// Absolute, Y-indexed.
    AbsoluteY,
    /// Immediate.
    Immediate,
    // Implied.
    Implied,
    /// Indirect.
    Indirect,
    /// X-indexed, indirect.
    XIndirect,
    /// Indirect, Y-indexed.
    IndirectY,
    /// Zeropage.
    Zeropage,
    /// Zeropage, X-indexed.
    ZeropageX,
    /// Zeropage, Y-indexed.
    ZeropageY,
}

impl AddressMode {
    /// Get the string representation of AddressMode. This is used instead of Display
    /// because zeropage and relative have identicla syntax, and which it is depends
    /// on the mnemonic.
    pub fn string_rep(&self, mnemonic: Mnemonic) -> String {
        String::from(
            if mnemonic.is_branch() && matches!(self, AddressMode::Zeropage) {
                "Relative"
            } else {
                match self {
                    AddressMode::Accumulator => "Accumulator",
                    AddressMode::Absolute => "Absolute",
                    AddressMode::AbsoluteX => "Absolute, X-Indexed",
                    AddressMode::AbsoluteY => "Absolute, Y-Indexed",
                    AddressMode::Immediate => "Immediate",
                    AddressMode::Implied => "Implied",
                    AddressMode::Indirect => "Indirect",
                    AddressMode::XIndirect => "X-indexed, indirect",
                    AddressMode::IndirectY => "Indirect, Y-indexed",
                    AddressMode::Zeropage => "Zeropage",
                    AddressMode::ZeropageX => "Zeropage, X-indexed",
                    AddressMode::ZeropageY => "Zeropage, Y-indexed",
                }
            },
        )
    }
}

lazy_static! {
/// Lookup opcode based on mnemonic and address mode. None Indicates an invalid combination.
pub static ref OPCODES: EnumMap<Mnemonic, EnumMap<AddressMode, Option<u8>>> = enum_map! {
    Adc => enum_map! {Accumulator => None,       Absolute => Some(0x6d), AbsoluteX => Some(0x7d), AbsoluteY => Some(0x79),
                      Immediate   => Some(0x69), Implied  => None,       Indirect  => None,       XIndirect => Some(0x61),
                      IndirectY   => Some(0x71), Zeropage => Some(0x65), ZeropageX => Some(0x75), ZeropageY => None},
    And => enum_map! {Accumulator => None,       Absolute => Some(0x2d), AbsoluteX => Some(0x3d), AbsoluteY => Some(0x39),
                      Immediate   => Some(0x29), Implied  => None,       Indirect  => None,       XIndirect => Some(0x21),
                      IndirectY   => Some(0x31), Zeropage => Some(0x25), ZeropageX => Some(0x35), ZeropageY => None},
    Asl => enum_map! {Accumulator => Some(0x0a), Absolute => Some(0x0e), AbsoluteX => Some(0x1e), AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x06), ZeropageX => Some(0x16), ZeropageY => None},
    Bcc => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x90), ZeropageX => None,       ZeropageY => None},
    Bcs => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xb0), ZeropageX => None,       ZeropageY => None},
    Beq => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xf0), ZeropageX => None,       ZeropageY => None},
    Bit => enum_map! {Accumulator => None,       Absolute => Some(0x2c), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x24), ZeropageX => None,       ZeropageY => None},
    Bmi => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x30), ZeropageX => None,       ZeropageY => None},
    Bne => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xd0), ZeropageX => None,       ZeropageY => None},
    Bpl => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x10), ZeropageX => None,       ZeropageY => None},
    Brk => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x00), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Bvc => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x50), ZeropageX => None,       ZeropageY => None},
    Bvs => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x70), ZeropageX => None,       ZeropageY => None},
    Clc => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x18), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Cld => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xd8), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Cli => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x58), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Clv => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xb8), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Cmp => enum_map! {Accumulator => None,       Absolute => Some(0xcd), AbsoluteX => Some(0xdd), AbsoluteY => Some(0xd9),
                      Immediate   => Some(0xc9), Implied  => None,       Indirect  => None,       XIndirect => Some(0xc1),
                      IndirectY   => Some(0xd1), Zeropage => Some(0xc5), ZeropageX => Some(0xd5), ZeropageY => None},
    Cpx => enum_map! {Accumulator => None,       Absolute => Some(0xec), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => Some(0xe0), Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xe4), ZeropageX => None,       ZeropageY => None},
    Cpy => enum_map! {Accumulator => None,       Absolute => Some(0xcc), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => Some(0xc0), Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xc4), ZeropageX => None,       ZeropageY => None},
    Dec => enum_map! {Accumulator => None,       Absolute => Some(0xce), AbsoluteX => Some(0xde), AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xc6), ZeropageX => Some(0xd6), ZeropageY => None},
    Dex => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xca), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Dey => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x88), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Eor => enum_map! {Accumulator => None,       Absolute => Some(0x4d), AbsoluteX => Some(0x5d), AbsoluteY => Some(0x59),
                      Immediate   => Some(0x49), Implied  => None,       Indirect  => None,       XIndirect => Some(0x41),
                      IndirectY   => Some(0x51), Zeropage => Some(0x45), ZeropageX => Some(0x55), ZeropageY => None},
    Inc => enum_map! {Accumulator => None,       Absolute => Some(0xee), AbsoluteX => Some(0xfe), AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xe6), ZeropageX => Some(0xf6), ZeropageY => None},
    Inx => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xe8), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Iny => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xc8), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Jmp => enum_map! {Accumulator => None,       Absolute => Some(0x4c), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => Some(0x6c), XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Jsr => enum_map! {Accumulator => None,       Absolute => Some(0x20), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Lda => enum_map! {Accumulator => None,       Absolute => Some(0xad), AbsoluteX => Some(0xbd), AbsoluteY => Some(0xb9),
                      Immediate   => Some(0xa9), Implied  => None,       Indirect  => None,       XIndirect => Some(0xa1),
                      IndirectY   => Some(0xb1), Zeropage => Some(0xa5), ZeropageX => Some(0xb5), ZeropageY => None},
    Ldx => enum_map! {Accumulator => None,       Absolute => Some(0xae), AbsoluteX => None,       AbsoluteY => Some(0xbe),
                      Immediate   => Some(0xa2), Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xa6), ZeropageX => None,       ZeropageY => Some(0xb6)},
    Ldy => enum_map! {Accumulator => None,       Absolute => Some(0xac), AbsoluteX => Some(0xbc), AbsoluteY => None,
                      Immediate   => Some(0xa0), Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0xa4), ZeropageX => Some(0xb4), ZeropageY => None},
    Lsr => enum_map! {Accumulator => Some(0x4a), Absolute => Some(0x4e), AbsoluteX => Some(0x5e), AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x46), ZeropageX => Some(0x56), ZeropageY => None},
    Nop => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xea), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Ora => enum_map! {Accumulator => None,       Absolute => Some(0x0d), AbsoluteX => Some(0x1d), AbsoluteY => Some(0x19),
                      Immediate   => Some(0x09), Implied  => None,       Indirect  => None,       XIndirect => Some(0x01),
                      IndirectY   => Some(0x11), Zeropage => Some(0x05), ZeropageX => Some(0x15), ZeropageY => None},
    Pha => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x48), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Php => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x08), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Pla => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x68), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Plp => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x28), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Rol => enum_map! {Accumulator => Some(0x2a), Absolute => Some(0x2e), AbsoluteX => Some(0x3e), AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x26), ZeropageX => Some(0x36), ZeropageY => None},
    Ror => enum_map! {Accumulator => Some(0x6a), Absolute => Some(0x6e), AbsoluteX => Some(0x7e), AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x66), ZeropageX => Some(0x76), ZeropageY => None},
    Rti => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x40), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Rts => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x60), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Sbc => enum_map! {Accumulator => None,       Absolute => Some(0xed), AbsoluteX => Some(0xfd), AbsoluteY => Some(0xf9),
                      Immediate   => Some(0xe9), Implied  => None,       Indirect  => None,       XIndirect => Some(0xe1),
                      IndirectY   => Some(0xf1), Zeropage => Some(0xe5), ZeropageX => Some(0xf5), ZeropageY => None},
    Sec => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x38), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Sed => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xf8), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Sei => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x78), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Sta => enum_map! {Accumulator => None,       Absolute => Some(0x8d), AbsoluteX => Some(0x9d), AbsoluteY => Some(0x99),
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => Some(0x81),
                      IndirectY   => Some(0x91), Zeropage => Some(0x85), ZeropageX => Some(0x95), ZeropageY => None},
    Stx => enum_map! {Accumulator => None,       Absolute => Some(0x8e), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x86), ZeropageX => None,       ZeropageY => Some(0x96)},
    Sty => enum_map! {Accumulator => None,       Absolute => Some(0x8c), AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => Some(0x84), ZeropageX => Some(0x94), ZeropageY => None},
    Tax => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xaa), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Tay => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xa8), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Tsx => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0xba), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Txa => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x8a), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Txs => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x9a), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Tya => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x98), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Dfb => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Dfw => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Equ => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Hlt => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => Some(0x02), Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Inl => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Org => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
    Sct => enum_map! {Accumulator => None,       Absolute => None,       AbsoluteX => None,       AbsoluteY => None,
                      Immediate   => None,       Implied  => None,       Indirect  => None,       XIndirect => None,
                      IndirectY   => None,       Zeropage => None,       ZeropageX => None,       ZeropageY => None},
};
}

static APPLE_CHARACTER_MAP: phf::Map<char, u8> = phf_map! {
    ' ' => 0xa0,
    '!' => 0xa1,
    '"' => 0xa2,
    '#' => 0xa3,
    '$' => 0xa4,
    '%' => 0xa5,
    '&' => 0xa6,
    '\'' => 0xa7,
    '(' => 0xa8,
    ')' => 0xa9,
    '*' => 0xaa,
    '+' => 0xab,
    ',' => 0xac,
    '-' => 0xad,
    '.' => 0xae,
    '/' => 0xaf,
    '0' => 0xb0,
    '1' => 0xb1,
    '2' => 0xb2,
    '3' => 0xb3,
    '4' => 0xb4,
    '5' => 0xb5,
    '6' => 0xb6,
    '7' => 0xb7,
    '8' => 0xb8,
    '9' => 0xb9,
    ':' => 0xba,
    ';' => 0xbb,
    '<' => 0xbc,
    '=' => 0xbd,
    '>' => 0xbe,
    '?' => 0xbf,
    '@' => 0xc0,
    'a' => 0xc1,
    'b' => 0xc2,
    'c' => 0xc3,
    'd' => 0xc4,
    'e' => 0xc5,
    'f' => 0xc6,
    'g' => 0xc7,
    'h' => 0xc8,
    'i' => 0xc9,
    'j' => 0xca,
    'k' => 0xcb,
    'l' => 0xcc,
    'm' => 0xcd,
    'n' => 0xce,
    'o' => 0xcf,
    'p' => 0xd0,
    'q' => 0xd1,
    'r' => 0xd2,
    's' => 0xd3,
    't' => 0xd4,
    'u' => 0xd5,
    'v' => 0xd6,
    'w' => 0xd7,
    'x' => 0xd8,
    'y' => 0xd9,
    'z' => 0xda,
    '[' => 0xdb,
    '\\' => 0xdc,
    ']' => 0xdd,
    '^' => 0xde,
    '_' => 0xdf,
};

enum AppleCharacterMode {
    Normal,
    Flashing,
    Inverse,
}

pub struct AppleCharacter(pub u8);

impl From<char> for AppleCharacter {
    fn from(character: char) -> Self {
        AppleCharacter(
            APPLE_CHARACTER_MAP
                .get(&character.to_ascii_lowercase())
                .cloned()
                .unwrap_or(0xe0),
        )
    }
}

pub fn apple_string(
    string: &String,
    string_location: Location,
) -> Result<Vec<AppleCharacter>, AssemblerError> {
    let mut mode = AppleCharacterMode::Normal;
    let mut bytes = Vec::with_capacity(string.len());

    let mut characters = string.as_str().chars().enumerate().peekable();
    while characters.peek().is_some() {
        let (character_index, mut character) = characters.next().unwrap();
        if character == '\\' {
            // Handle escape sequence
            let (_, escape) = characters
                .next()
                .ok_or(AssemblerError {
                    message: String::from("Expected an escape sequence, found end of string"),
                    labels: vec![(string_location.clone(), None)],
                    help: None,
                })
                .ok()
                .filter(|(_, next)| {
                    *next == 'n'
                        || *next == 'i'
                        || *next == 'f'
                        || *next == '0'
                        || *next == '"'
                        || *next == '\\'
                })
                .ok_or(AssemblerError {
                    message: String::from("Invalid escape sequence"),
                    labels: vec![(string_location.clone(), None)],
                    help: Some(String::from(
                        "Valid escape sequences are `\\n`, `\\f`, `\\i`, '\\0', '\\'",
                    )),
                })?;
            if escape == '"' || escape == '\\' {
                character = escape;
            } else {
                mode = match escape {
                    'n' => AppleCharacterMode::Normal,
                    'f' => AppleCharacterMode::Flashing,
                    'i' => AppleCharacterMode::Inverse,
                    _ => {
                        bytes.push(AppleCharacter(0x00));
                        mode
                    }
                };
                continue;
            }
        }
        let byte = APPLE_CHARACTER_MAP
            .get(&character.to_ascii_lowercase())
            .cloned();

        if let Some(byte) = byte {
            let modifier = match mode {
                AppleCharacterMode::Normal => 0x00,
                AppleCharacterMode::Flashing => {
                    if byte >= 0xc0 {
                        0x80
                    } else {
                        0x40
                    }
                }
                AppleCharacterMode::Inverse => {
                    if byte >= 0xc0 {
                        0xc0
                    } else {
                        0x80
                    }
                }
            };
            bytes.push(AppleCharacter(byte - modifier));
        } else {
            return Err(AssemblerError {
                message: format!("Character `{}` is invalid", character),
                labels: vec![(
                    Location {
                        span: string_location.span.start + character_index + 1
                            ..string_location.span.start + character_index + 2,
                        file_name: string_location.file_name.clone(),
                    },
                    None,
                )],
                help: None,
            });
        }
    }

    Ok(bytes)
}
