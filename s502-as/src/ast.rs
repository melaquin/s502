use std::ops::Range;

/// Representation of an assembly file.
pub struct Program {
    pub lines: Vec<Line>,
}

/// Representatioon of one line of assembly.
pub struct Line {
    pub label: Option<Label>,
}

/// A label that appears at the beginning of a line.
#[derive(Debug, PartialEq)]
pub enum Label {
    Top(TopLabel),
    Sub(SubLabel),
    // TODO Push(file_id) variant for includes and Pop for end of include
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
