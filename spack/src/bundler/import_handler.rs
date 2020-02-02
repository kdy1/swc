use super::Bundler;
use crate::bundler::{import_analysis::ImportInfo, scope::Scope};
use anyhow::Error;
use std::sync::Arc;
use swc_common::{Fold, FoldWith};
use swc_ecma_ast::*;

impl Bundler {
    pub(super) fn handle_imports(
        &self,
        module: Module,
        imports: Arc<ImportInfo>,
    ) -> Result<Module, Error> {
        let mut v = Folder {
            scope: &self.scope,
            imports,
        };

        Ok(module.fold_with(&mut v))
    }
}

struct Folder<'a> {
    scope: &'a Scope,
    imports: Arc<ImportInfo>,
}

impl Fold<Expr> for Folder<'_> {
    fn fold(&mut self, e: Expr) -> Expr {
        let e: Expr = e.fold_children(self);

        match e {
            Expr::Ident(i) => {
                // Replace ident with constant, if possible
            }

            _ => {}
        }

        e
    }
}

impl Fold<MemberExpr> for Folder<'_> {
    fn fold(&mut self, mut e: MemberExpr) -> MemberExpr {
        e.obj = e.obj.fold_with(self);
        if e.computed {
            e.prop = e.prop.fold_with(self);
        }

        e
    }
}
