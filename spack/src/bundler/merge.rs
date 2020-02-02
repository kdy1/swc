use super::Bundler;
use crate::{chunk::Chunk, ModuleId};
use anyhow::Error;
use swc_ecma_ast::*;

#[derive(Debug)]
pub(crate) enum MergedModule {
    Modules(Module, Vec<Chunk>),
    Module(Module),
}

impl Bundler {
    pub(super) fn merge_modules(
        &self,
        entry: Module,
        modules: &[ModuleId],
    ) -> Result<MergedModule, Error> {
        let mut v = Merger { to: entry };

        Ok(MergedModule::Module(v.to))
    }
}

#[derive(Debug)]
struct Merger {
    to: Module,
}

/// Returns true if loading a module has any side effect.
fn has_side_effect(module: &Module) -> bool {
    for item in module.body {
        match item {
            ModuleItem::ModuleDecl(_) => {}
            ModuleItem::Stmt(_) => {}
        }
    }

    // De-optimization is better than breaking a code.
    true
}
