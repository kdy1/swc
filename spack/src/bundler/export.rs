use super::Bundler;
use crate::Id;
use anyhow::Error;
use std::mem::replace;
use swc_common::{Fold, FoldWith, DUMMY_SP};
use swc_ecma_ast::*;

impl Bundler {
    /// This method removes exported pure constants from the module.
    ///
    /// A pure constant is a exported literal.
    ///
    ///
    /// TODO: Support pattern like
    ///     export const [a, b] = [1, 2]
    pub(super) fn extract_export_info(&self, module: &mut Module) -> Result<ExportInfo, Error> {
        let mut v = ExportFinder::default();

        let m = replace(
            module,
            Module {
                span: DUMMY_SP,
                body: vec![],
                shebang: None,
            },
        );
        let m = m.fold_with(&mut v);

        *module = m;

        Ok((v.info))
    }
}

#[derive(Debug, Default)]
pub(super) struct ExportInfo {
    pub pure_constants: Vec<(Id, Lit)>,
}

#[derive(Debug, Default)]
struct ExportFinder {
    info: ExportInfo,
}

impl Fold<ModuleItem> for ExportFinder {
    fn fold(&mut self, item: ModuleItem) -> ModuleItem {
        match item {
            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                decl: Decl::Var(v),
                ..
            })) if v.kind == VarDeclKind::Const
                && v.decls.iter().all(|v| {
                    (match v.name {
                        Pat::Ident(..) => true,
                        _ => false,
                    }) && (match v.init {
                        Some(box Expr::Lit(..)) => true,
                        _ => false,
                    })
                }) =>
            {
                self.info
                    .pure_constants
                    .extend(v.decls.into_iter().map(|decl| {
                        let id = match decl.name {
                            Pat::Ident(i) => i.into(),
                            _ => unreachable!(),
                        };

                        let lit = match decl.init {
                            Some(box Expr::Lit(l)) => l,
                            _ => unreachable!(),
                        };

                        (id, lit)
                    }));

                return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }));
            }

            _ => {}
        }

        item.fold_children(self)
    }
}
