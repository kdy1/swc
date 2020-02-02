use super::Bundler;
use crate::{
    bundler::{load_transformed::TransformedModule, scope::Scope},
    chunk::Chunk,
    ModuleId,
};
use anyhow::{Context, Error};
use swc_common::{errors::Handler, fold::FoldWith, Fold};
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
                    imported.fm.name.clone(),
                    (*imported.module).clone(),
                    Some(ids.clone()),
                );
                let dep = dep.fold_with(&mut Unexporter);

                // TODO: Handle renaming

                prepend_stmts(&mut entry.body, dep.body.into_iter());
            }
        }

        Ok(entry)
    }
}

/// `export var a = 1` => `var a = 1`
struct Unexporter;

impl Fold<ModuleItem> for Unexporter {
    fn fold(&mut self, item: ModuleItem) -> ModuleItem {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::ExportDecl(decl) => ModuleItem::Stmt(Stmt::Decl(decl.decl)),

                // TODO: Handle all
                _ => ModuleItem::ModuleDecl(decl),
            },

            _ => item,
        }
    }
}
