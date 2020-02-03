use super::Bundler;
use crate::{
    bundler::{load_transformed::TransformedModule, scope::Scope},
    chunk::Chunk,
    ModuleId,
};
use anyhow::{Context, Error};
use swc_common::{errors::Handler, fold::FoldWith, util::move_map::MoveMap, Fold, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms::optimization::{dce, simplifier};
use swc_ecma_utils::{prepend_stmts, StmtLike};

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
        let mut buf = vec![];
        for (src, ids) in &info.merged_imports.ids {
            //
            if let Some(imported) = self.scope.get_module(src.module_id) {
                let dep = (*imported.module).clone();
                let dep: Module =
                    self.drop_unused(imported.fm.clone(), dep, imported.mark(), Some(ids.clone()));
                let dep = dep.fold_with(&mut Unexporter).fold_with(&mut dce());

                // TODO: Handle renaming
                buf.extend(dep.body);
            }
        }

        prepend_stmts(&mut entry.body, buf.into_iter());

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
                ModuleDecl::ExportDefaultExpr(..) => {
                    ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
                }
                // TODO: Handle all
                _ => ModuleItem::ModuleDecl(decl),
            },

            _ => item,
        }
    }
}
