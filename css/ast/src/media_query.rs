use crate::{ParenProperty, Text};
use swc_common::{ast_node, Span};

#[ast_node]
pub enum MediaQuery {
    #[tag("AndMediaQuery")]
    And(AndMediaQuery),
    #[tag("OrMediaQuery")]
    Or(OrMediaQuery),
    /// e.g. `screen`
    #[tag("Text")]
    Text(Text),
    #[tag("ParenPropery")]
    Value(ParenProperty),
}

#[ast_node]
pub struct AndMediaQuery {
    pub span: Span,
    pub first: Box<MediaQuery>,
    pub second: Box<MediaQuery>,
}

#[ast_node]
pub struct OrMediaQuery {
    pub span: Span,
    pub first: Box<MediaQuery>,
    pub second: Box<MediaQuery>,
}
