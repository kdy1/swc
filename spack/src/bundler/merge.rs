use super::Bundler;
use anyhow::Error;
use fxhash::FxHashMap;
use std::sync::Arc;
use swc_common::{Fold, FromVariant};
use swc_ecma_ast::Module;

#[derive(Debug)]
pub enum MergedModule {
    Modules(Module, FxHashMap<JsWord, Module>),
    Module(Module),
}

impl Bundler {
    pub(super) fn merge_modules(
        &self,
        entry: Module,
        modules: &[Arc<Module>],
    ) -> Result<MergedModule, Error> {
        let mut v = Merger { to: entry };

        Ok(MergedModule::Module(v.to))
    }
}

#[derive(Debug)]
struct Merger {
    to: Module,
}
