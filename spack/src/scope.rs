use super::Bundler;
use crate::{Id, ModuleId, QualifiedId};
use dashmap::DashMap;
use swc_ecma_ast::Lit;

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
