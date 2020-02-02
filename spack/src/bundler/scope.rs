use super::Bundler;
use crate::{bundler::load_transformed::TransformedModule, Id, ModuleId, QualifiedId};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::{path::PathBuf, sync::Arc};
use swc_common::Mark;
use swc_ecma_ast::Lit;

#[derive(Debug, Default)]
pub(super) struct Scope {
    /// Phase 1 cache
    pub cache: DashMap<Arc<PathBuf>, TransformedModule>,

    /// Marks applied to bindings
    pub module_marks: DashMap<ModuleId, Mark, FxBuildHasher>,
}
