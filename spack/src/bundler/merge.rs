use super::Bundler;
use crate::{
    bundler::{load_transformed::TransformedModule, scope::Scope},
    chunk::Chunk,
    ModuleId,
};
use anyhow::{Context, Error};
use swc_common::errors::Handler;
use swc_ecma_ast::*;
use swc_ecma_utils::prepend_stmts;

#[derive(Debug)]
pub(crate) enum MergedModule {
    Modules(Module, Vec<Chunk>),
    Module(Module),
}

impl Bundler {
    pub(super) fn merge_modules(
        &self,
        mut entry: Module,
        info: &TransformedModule,
    ) -> Result<Module, Error> {
        for (src, ids) in &info.merged_imports.ids {
            //
            if let Some(imported) = self.scope.get_module(src.module_id) {
                let dep: Module = self.drop_unused(
                    info.fm.name.clone(),
                    (*info.module).clone(),
                    Some(ids.clone()),
                );

                // TODO: Handle rename

                prepend_stmts(&mut entry.body, dep.body.into_iter());
            }
        }

        Ok(entry)
    }
}
