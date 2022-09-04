use std::ops::Range;

#[derive(PartialEq)]
pub enum Visibility {
    Global,
    Object,
}

pub struct Label {
    pub name: String,
    pub span: Range<usize>,
    pub visibility: Visibility,
    pub sublabels: Vec<SubLabel>,
}

pub struct SubLabel {
    pub name: String,
    pub span: Range<usize>,
}
