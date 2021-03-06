#![deny(unused)]

pub use self::{at_rule::*, media_query::*};
pub use at_rule::AtRule;
use serde::{Deserialize, Serialize};
use swc_atoms::JsWord;
use swc_common::{ast_node, Span};

mod at_rule;
mod media_query;

/// Quoted string
#[ast_node]
pub struct Str {
    /// Includes quotes
    pub span: Span,
    /// Does not include quotes
    pub sym: String,
}

#[ast_node]
pub struct Text {
    pub span: Span,
    pub sym: JsWord,
}

#[ast_node]
pub struct Number {
    pub span: Span,
    pub value: Box<Value>,
}

#[ast_node]
pub struct Stylesheet {
    pub span: Span,
    pub rules: Vec<Rule>,
}

#[ast_node]
pub enum Rule {
    #[tag("AtRule")]
    At(AtRule),
    #[tag("StyleRule")]
    Style(StyleRule),
}

#[ast_node]
pub struct StyleRule {
    pub span: Span,
    pub selectors: Vec<Box<Selector>>,
    pub block: DeclBlock,
}

#[ast_node]
pub struct DeclBlock {
    /// Includes `{` and `}`.
    pub span: Span,
    pub properties: Vec<Property>,
}

/// A selector is composed of `CompoundSelector`s separated by
/// `Combinator`s. It selects elements based on their parent selectors.
#[ast_node]
pub struct Selector {
    pub span: Span,
    /// This is never empty.
    pub components: Vec<SelectorComponent>,
}

#[ast_node]
pub enum SimpleSelector {
    /// `*`
    #[tag("UniversalSelector")]
    Universal(UniversalSelector),

    /// A pseudo-class or pseudo-element selector.
    ///
    /// The semantics of a specific pseudo selector depends on its name. Some
    /// selectors take arguments, including other selectors.
    #[tag("PseudoSelector")]
    Pseudo(PseudoSelector),

    /// A type selector.
    ///
    /// This selects elements whose name equals the given name.
    #[tag("TagSelector")]
    Tag(TagSelector),
    #[tag("IdSelector")]
    Id(IdSelector),

    /// A class selector.
    ///
    /// This selects elements whose `class` attribute contains an identifier
    /// with the given name.
    #[tag("ClassSelector")]
    Class(ClassSelector),

    #[tag("AttributeSelector")]
    Attribute(AttributeSelector),
}

#[ast_node]
pub struct UniversalSelector {
    pub span: Span,
}

#[ast_node]
pub struct AttributeSelector {
    pub span: Span,
    pub attr: Text,
    /// TODO: Change this to something better.  
    pub value: Option<Text>,
    pub modifier: Option<char>,
    pub op: AttributeOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AttributeOp {
    /// \[attr\]
    ///
    /// Represents elements with an attribute name of `attr`
    Any,

    /// [attr=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value is exactly `value`
    Equals,

    /// [attr~=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value is a whitespace-separated list of words,
    /// one of which is exactly `value`
    Include,

    /// [attr|=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value can be exactly value or can begin with
    /// `value` immediately followed by a hyphen (`-`)
    Dash,

    /// [attr^=value]
    Prefix,

    /// [attr$=value]
    Suffix,

    /// [attr*=value]
    ///
    /// Represents elements with an attribute name of `attr`
    /// whose value contains at least one occurrence of
    /// `value` within the string
    Contains,
}

/// e.g. `a.my-btn`
#[ast_node]
pub struct CompoundSelector {
    pub span: Span,
    pub selectors: Vec<SimpleSelector>,
}

#[ast_node]
pub enum SelectorComponent {
    #[tag("CompoundSelector")]
    Compound(CompoundSelector),
    #[tag("CombinatorSelector")]
    Combinator(CombinatorSelector),
}

#[ast_node]
pub struct CombinatorSelector {
    pub span: Span,
    pub combinator: Combinator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Combinator {
    /// Matches the right-hand selector if it's immediately adjacent to the
    /// left-hand selector in the DOM tree.
    ///
    /// `'+'`
    NextSibling,

    /// Matches the right-hand selector if it's a direct child of the left-hand
    /// selector in the DOM tree.
    ///
    /// `'>'`
    Child,

    /// Matches the right-hand selector if it comes after the left-hand selector
    /// in the DOM tree.
    ///
    /// `'~'`
    FollowingSibling,
}

#[ast_node]
pub struct PseudoSelector {
    pub span: Span,

    /// The name of this selector.
    pub name: Text,

    /// Whether this is a pseudo-class selector.
    ///
    /// If this is false, this is a pseudo-element selector
    pub is_class: bool,

    /// Whether this is syntactically a pseudo-class selector.
    ///
    /// This is the same as `is_class` unless this selector is a pseudo-element
    /// that was written syntactically as a pseudo-class (`:before`, `:after`,
    /// `:first-line`, or `:first-letter`).
    ///
    /// If this is false, it is syntactically a psuedo-element
    pub is_syntactic_class: bool,

    /// The non-selector argument passed to this selector.
    ///
    /// This is `None` if there's no argument. If `argument` and `selector` are
    /// both non-`None`, the selector follows the argument.
    pub arguments: Option<Vec<Text>>,

    /// The selector argument passed to this selector.
    ///
    /// This is `None` if there's no selector. If `argument` and `selector` are
    /// both non-`None`, the selector follows the argument.
    pub selector: Option<Vec<Box<Selector>>>,
}

#[ast_node]
pub struct IdSelector {
    pub span: Span,
    /// Does not include `#`
    pub text: Text,
}

#[ast_node]
pub struct ClassSelector {
    pub span: Span,
    /// Does not include `.`
    pub text: Text,
}

#[ast_node]
pub struct TagSelector {
    pub span: Span,
    pub text: Text,
}

#[ast_node]
pub enum BaseSelector {
    #[tag("IdSelector")]
    Id(IdSelector),
    #[tag("ClassSelector")]
    Class(ClassSelector),
}

#[ast_node]
pub struct Property {
    pub span: Span,
    pub name: Text,
    pub value: Value,
    /// The span includes `!`
    pub important: Option<Span>,
}

#[ast_node]
pub enum Value {
    #[tag("ParenValue")]
    Paren(ParenValue),
    #[tag("UnitValue")]
    Unit(UnitValue),
    #[tag("Number")]
    Number(Number),
    #[tag("HashValue")]
    Hash(HashValue),
    #[tag("Text")]
    Text(Text),
    #[tag("Str")]
    Str(Str),
}

#[ast_node]
pub struct ParenProperty {
    /// Includes `(` and `)`
    pub span: Span,
    pub property: Property,
}

#[ast_node]
pub struct HashValue {
    /// Includes `#`
    pub span: Span,
    /// Does **not** include `#`
    pub value: Text,
}

#[ast_node]
pub struct UnitValue {
    pub span: Span,
    pub value: Text,
    pub unit: SpannedUnit,
}

#[ast_node]
pub struct SpannedUnit {
    pub span: Span,
    pub unit: Unit,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    Px,
}

#[ast_node]
pub struct ParenValue {
    pub span: Span,
    pub value: Box<Value>,
}
