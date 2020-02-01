use super::Bundler;
use dashmap::DashMap;
use swc_atoms::JsWord;
use swc_common::{SourceFile, SyntaxContext};
use swc_ecma_ast::{Ident, Lit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u128);

impl<'a> From<&'a SourceFile> for ModuleId {
    fn from(fm: &'a SourceFile) -> Self {
        ModuleId(fm.name_hash)
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

#[derive(Debug, Default)]
pub(super) struct Scope {
    pure_constants: DashMap<QualifiedId, Lit>,
}

impl Bundler {
    pub(crate) fn store_pure_constants(&self, module_id: ModuleId, pure_constants: Vec<(Id, Lit)>) {
        for (id, lit) in pure_constants {
            self.scope.pure_constants.insert((module_id, id), lit);
        }
    }
}
