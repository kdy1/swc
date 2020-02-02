use super::Bundler;
use crate::{bundler::transform::TransformedModule, Id, ModuleId};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::{path::PathBuf, sync::Arc};
use swc_common::Mark;
use swc_ecma_ast::Lit;

pub(super) type PureConstants = DashMap<ModuleId, Vec<(Id, Lit)>, FxBuildHasher>;

#[derive(Debug, Default)]
pub(super) struct Scope {
    pure_constants: PureConstants,
    /// Phase 1 cache
    pub cache: DashMap<Arc<PathBuf>, TransformedModule>,

    /// Marks applied to bindings
    pub module_marks: DashMap<ModuleId, Mark, FxBuildHasher>,
}

impl Bundler {
    pub(super) fn store_pure_constants(&self, module_id: ModuleId, pure_constants: Vec<(Id, Lit)>) {
        self.scope.pure_constants.insert(module_id, pure_constants);
    }
}

impl Scope {
    pub(super) fn pure_constants(&self) -> &PureConstants {
        &self.pure_constants
    }
}
