use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use swc_atoms::JsWord;
use swc_common::SyntaxContext;
use swc_ecma_ast::Ident;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u64);

#[derive(Debug, Default)]
pub(crate) struct ModuleIdGenerator(AtomicU64);

impl ModuleIdGenerator {
    pub fn gen(&self) -> ModuleId {
        ModuleId(self.0.fetch_add(1, SeqCst))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(JsWord, SyntaxContext);

impl From<Ident> for Id {
    fn from(i: Ident) -> Self {
        Id(i.sym, i.span.ctxt())
    }
}

impl<'a> From<&'a Ident> for Id {
    fn from(i: &Ident) -> Self {
        Id(i.sym.clone(), i.span.ctxt())
    }
}

pub type QualifiedId = (ModuleId, Id);
