use super::Bundler;
use crate::{
    bundler::load_transformed::{Specifier, TransformedModule},
    chunk::Chunk,
};
use anyhow::Error;
use swc_common::{fold::FoldWith, Fold, Mark, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_transforms::{hygiene, optimization::dce};
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
        let mut buf = vec![];
        for (src, specifiers) in &info.merged_imports.specifiers {
            if src.is_unconditional {
                if let Some(imported) = self.scope.get_module(src.module_id) {
                    let dep = (*imported.module).clone();
                    let dep: Module =
                        self.drop_unused(imported.fm.clone(), dep, Some((*specifiers).clone()));
                    let mut dep = dep.fold_with(&mut Unexporter).fold_with(&mut dce());
                    // TODO: Handle renaming exports

                    if !specifiers.is_empty() {
                        let mut v = ImportMarker {
                            mark: imported.mark(),
                            specifiers: &specifiers,
                        };
                        entry = entry.fold_with(&mut v);
                        dep = dep.fold_with(&mut v);
                    }

                    buf.extend(dep.body);
                }
            } else {
                unimplemented!("conditional dependency: {} -> {}", info.id, src.module_id)
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
    specifiers: &'a [Specifier],
}

impl Fold<Ident> for ImportMarker<'_> {
    fn fold(&mut self, mut node: Ident) -> Ident {
        if self.specifiers.iter().any(|id| *id.local() == node) {
            node.span = node.span.apply_mark(self.mark);
        }

        node
    }
}
