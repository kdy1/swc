use super::Bundler;
use crate::{
    bundler::{load_transformed::TransformedModule, scope::Scope},
    chunk::Chunk,
    debug::HygieneVisualizer,
    util::IdentMarker,
    Id, ModuleId,
};
use anyhow::{Context, Error};
use swc_common::{errors::Handler, fold::FoldWith, util::move_map::MoveMap, Fold, Mark, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms::{
    hygiene,
    optimization::{dce, simplifier},
};
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
                let dep: Module = self.drop_unused(imported.fm.clone(), dep, Some(ids.clone()));
                let dep = dep
                    .fold_with(&mut Unexporter)
                    .fold_with(&mut dce())
                    .fold_with(&mut IdentMarker(imported.mark()));

                if !ids.is_empty() {
                    entry = entry.fold_with(&mut ImportMarker {
                        mark: imported.mark(),
                        ids: &ids,
                    });
                }

                // TODO: Handle renaming
                buf.extend(dep.body);
            }
        }

        prepend_stmts(&mut entry.body, buf.into_iter());

        Ok(entry.fold_with(&mut hygiene()))
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

struct ImportMarker<'a> {
    mark: Mark,
    ids: &'a [Id],
}

impl Fold<Ident> for ImportMarker<'_> {
    fn fold(&mut self, mut node: Ident) -> Ident {
        if self.ids.iter().any(|id| *id == node) {
            node.span = node.span.apply_mark(self.mark);
        }

        node
    }
}
