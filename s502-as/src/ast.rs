use std::ops::Range;

use super::lexer::Literal;

#[derive(Debug, PartialEq)]
/// Information for retrieving an except from the source code.
pub struct Location {
    /// Where in the source.
    pub span: Range<usize>,
    /// Which file. This is used as a map key to find the
    /// corresponding file ID.
    pub name: String,
}

#[derive(PartialEq)]
/// A file inclusion encountered while parsing.
pub struct Include {
    /// The file being included as it appears in the source.
    pub included: String,
    /// Where in the source the file was included.
    pub loc: Location,
}

/// Representation of an assembly file.
pub struct Program {
    pub lines: Vec<Line>,
}

pub struct Line {
    /// Label giving the address of this
    /// line a meaningful name
    pub label: Option<Label>,
    /// What to do for this line.
    pub action: Option<Action>,
}

/// Something for either the target CPU or the assembler to do.
pub enum Action {
    /// An assembler directive or CPU instruction.
    Instruction(Mnemonic, Option<Operand>),
    /// This tells the next stage that the following lines
    /// came from the file with this ID.
    PushInclude(usize),
    /// This tells the next stage that the most recently
    /// included file has ended.
    PopInclude,
}

/// Representatioon of one line of assembly.
pub enum OldLine {
    Instruction(Instruction),
    /// This tells the next stage that the foollowing lines
    /// came from the file with this ID.
    PushInclude(usize),
    /// This tells the next stage that the most recently
    /// included file has ended.
    PopInclude,
}

pub struct Instruction {
    pub label: Option<Label>,
}

/// A label that appears at the beginning of a line.
#[derive(Debug, PartialEq)]
pub enum Label {
    Top(TopLabel),
    Sub(SubLabel),
}

/// Top level label of the line.
#[derive(Debug, PartialEq)]
pub struct TopLabel {
    pub name: String,
    pub span: Range<usize>,
    pub visibility: Visibility,
    pub sublabels: Vec<SubLabel>,
}

/// A toplevel label can be exposed to other objects or kept
/// to only the compilation unit in which it appears.
#[derive(Debug, PartialEq)]
pub enum Visibility {
    Global,
    Object,
}

/// A sublabel is only visible within the toplevel label it is under.
#[derive(Debug, PartialEq)]
pub struct SubLabel {
    pub name: String,
    pub span: Range<usize>,
}

/// An instruction or directive to execute.
pub enum Mnemonic {
    Dfb,
    Dfw,
    Equ,
    Inl,
    // TODO implied instructions don't need an operand
    // so don't try to parse comment as operand, use LUT
    Hlt,
    Org,
    Sct,
}

/// The parsed instruction operand.
pub enum Operand {
    Literal(Literal),
    // TODO probalby don't make lexer create a Literal
    // because the parser should also create a Reference literal
}
