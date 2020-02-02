use super::Bundler;
use crate::{
    bundler::load_transformed::{MergedImports, TransformedModule},
    Id, ModuleId, QualifiedId,
};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::{Mark, SourceFile};
use swc_ecma_ast::{Lit, Module};

pub(super) type ModuleWithMetadata = (Arc<SourceFile>, Arc<Module>, Arc<MergedImports>);

#[derive(Debug, Default)]
pub(super) struct Scope {
    /// Phase 1 cache
    cache: DashMap<Arc<PathBuf>, TransformedModule>,
    /// Phase 1 cache
    modules: DashMap<ModuleId, ModuleWithMetadata>,

    /// Marks applied to bindings
    pub module_marks: DashMap<ModuleId, Mark, FxBuildHasher>,
}

impl Scope {
    /// Stores module information. The information should contain only
    /// information gotten from module itself. In other words, it should not
    /// contains information from a dependency.
    pub fn store_module(&self, path: Arc<PathBuf>, info: TransformedModule) {
        self.cache.insert(path, info.clone());
        self.modules.insert(info.0, (info.1, info.2, info.3));
    }

    pub fn get_module_by_path(&self, path: &PathBuf) -> Option<TransformedModule> {
        Some(self.cache.get(path)?.value().clone())
    }
    pub fn get_module(&self, id: ModuleId) -> Option<ModuleWithMetadata> {
        Some(self.modules.get(id)?.value().clone())
    }
}
