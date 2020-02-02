use super::Bundler;
use crate::{bundler::transform::TransformedModule, Id, ModuleId, QualifiedId};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::{path::PathBuf, sync::Arc};
use swc_common::Mark;
use swc_ecma_ast::Lit;

#[derive(Debug, Default)]
pub(super) struct Scope {
    pure_constants: DashMap<QualifiedId, Lit>,
    /// Phase 1 cache
    pub cache: DashMap<Arc<PathBuf>, TransformedModule>,

    /// Marks applied to bindings
    pub module_marks: DashMap<ModuleId, Mark, FxBuildHasher>,
}

impl Bundler {
    pub(crate) fn store_pure_constants(&self, module_id: ModuleId, pure_constants: Vec<(Id, Lit)>) {
        for (id, lit) in pure_constants {
            self.scope.pure_constants.insert((module_id, id), lit);
        }
    }
}
