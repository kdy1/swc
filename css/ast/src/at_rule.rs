use crate::{media_query::MediaQuery, DeclBlock, Number, ParenProperty, Str, Text};
use swc_common::{ast_node, Span};
#[ast_node]
pub enum AtRule {
    #[tag("CharsetRule")]
    Charset(CharsetRule),
    #[tag("MediaRule")]
    Media(MediaRule),
    #[tag("ImportRule")]
    Import(ImportRule),
    #[tag("SupportsRule")]
    Supports(SupportsRule),
    #[tag("KeyframesRule")]
    Keyframes(KeyframesRule),
    #[tag("FontFaceRule")]
    FontFace(FontFaceRule),
    #[tag("*")]
    Unknown(UnknownAtRule),
}

#[ast_node]
pub struct CharsetRule {
    pub span: Span,
    pub charset: Str,
}

#[ast_node]
pub struct MediaRule {
    pub span: Span,
    pub query: Box<MediaQuery>,
}

#[ast_node]
pub struct ImportRule {
    pub span: Span,
    pub src: Str,
}

#[ast_node]
pub struct PageRule {
    pub span: Span,
    /// TODO: Create a dedicated ast type.
    pub selector: Option<Text>,
}

#[ast_node]
pub struct SupportsRule {
    pub span: Span,
    /// TODO: Create a dedicated ast type.
    pub query: Box<SupportsQuery>,

    pub rules: Vec<DeclBlock>,
}

#[ast_node]
pub enum SupportsQuery {
    #[tag("Property")]
    Property(ParenProperty),

    #[tag("AndSupportsQuery")]
    And(AndSupportsQuery),

    #[tag("OrSupportsQuery")]
    Or(OrSupportsQuery),
}

#[ast_node]
pub struct AndSupportsQuery {
    pub span: Span,
    pub first: Box<SupportsQuery>,
    pub second: Box<SupportsQuery>,
}

#[ast_node]
pub struct OrSupportsQuery {
    pub span: Span,
    pub first: Box<SupportsQuery>,
    pub second: Box<SupportsQuery>,
}

#[ast_node]
pub struct KeyframesRule {
    pub span: Span,
    pub name: Text,
    pub keyframes: Vec<KeyframeElement>,
}

#[ast_node]
pub struct KeyframeElement {
    pub span: Span,
    pub selector: KeyframeSelector,
    pub block: DeclBlock,
}

#[ast_node]
pub enum KeyframeSelector {
    #[tag("KeyframePercentSelector")]
    Percent(KeyframePercentSelector),
    #[tag("Text")]
    Extra(Text),
}

#[ast_node]
pub struct KeyframePercentSelector {
    pub span: Span,
    pub percent: Number,
}

#[ast_node]
pub struct FontFaceRule {
    pub span: Span,
    pub block: DeclBlock,
}

#[ast_node]
pub struct UnknownAtRule {
    pub span: Span,
    pub name: Text,
    pub extras: Text,
}
