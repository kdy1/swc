use crate::{
    bundler::load_transformed::{MergedImports, TransformedModule},
    id::ModuleIdGenerator,
    ModuleId,
};
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use std::{path::PathBuf, sync::Arc};
use swc_common::{Mark, SourceFile};
use swc_ecma_ast::Module;

pub(super) type ModuleWithMetadata = (Arc<SourceFile>, Arc<Module>, Arc<MergedImports>);

#[derive(Debug, Default)]
pub(super) struct Scope {
    pub module_id_gen: ModuleIdGenerator,

    /// Phase 1 cache
    modules: DashMap<ModuleId, ModuleWithMetadata, FxBuildHasher>,

    /// Marks applied to bindings
    pub module_marks: DashMap<ModuleId, Mark, FxBuildHasher>,
}

impl Scope {
    /// Stores module information. The information should contain only
    /// information gotten from module itself. In other words, it should not
    /// contains information from a dependency.
    pub fn store_module(&self, path: Arc<PathBuf>, info: TransformedModule) {
        self.modules.insert(info.0, (info.1, info.2, info.3));
    }

    pub fn get_module_by_path(&self, path: &Arc<PathBuf>) -> Option<TransformedModule> {
        let id = self.module_id_gen.gen(path);
        self.get_module(id).map(|v| (id, v.0, v.1, v.2))
    }

    pub fn get_module(&self, id: ModuleId) -> Option<ModuleWithMetadata> {
        Some(self.modules.get(&id)?.value().clone())
    }
}
