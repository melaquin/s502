use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum Visibility {
    Global,
    Object,
}

#[derive(Debug, PartialEq)]
pub struct Label {
    pub name: String,
    pub span: Range<usize>,
    pub visibility: Visibility,
    pub sublabels: Vec<SubLabel>,
}

#[derive(Debug, PartialEq)]
pub struct SubLabel {
    pub name: String,
    pub span: Range<usize>,
}
