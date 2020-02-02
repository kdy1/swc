use crate::Id;
use swc_common::{Fold, FoldWith, DUMMY_SP};
use swc_ecma_ast::*;

#[derive(Debug, Default)]
pub(super) struct ExportInfo {
    pub pure_constants: Vec<(Id, Lit)>,
}

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
