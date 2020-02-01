use dashmap::DashMap;
use std::sync::Arc;
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::Lit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u128);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(JsWord, SyntaxContext);

pub type QualifiedId = (ModuleId, Id);

#[derive(Debug, Default)]
pub(super) struct Scope {
    pure_constants: DashMap<QualifiedId, Arc<Lit>>,
}
