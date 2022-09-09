use std::ops::Range;

// TODO rename lexer's literal to immediate,
// make new Literal as an AST node,
// then make lexer a private mod
use crate::parser::lexer::Literal;

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
    pub name: String,
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
pub type Program = Vec<Item>;

#[derive(Debug, PartialEq)]
pub enum Item {
    /// Tell the assembler stage to mark
    /// the current address with a label.
    Label(Label),
    /// Either a directive or instruction.
    Instruction(Instruction),
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
    Dfb,
    Dfw,
    Equ,
    Inl,
    Hlt,
    Org,
    Sct,
}

impl Mnemonic {
    pub fn is_implied(&self) -> bool {
        self == &Mnemonic::Hlt
    }
}

/// The parsed instruction operand.
#[derive(Debug, PartialEq)]
pub enum Operand {
    Literal(Literal),
    // TODO don't make lexer create a Literal
    // because the parser should also create a Reference literal
}
