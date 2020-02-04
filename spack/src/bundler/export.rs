use super::Bundler;
use crate::{
    bundler::load_transformed::{Source, Specifier},
    Id,
};
use anyhow::Error;
use fxhash::FxHashMap;
use std::mem::replace;
use swc_common::{Fold, FoldWith, DUMMY_SP};
use swc_ecma_ast::*;
use swc_ecma_utils::private_ident;

impl Bundler {
    /// This method removes exported pure constants from the module.
    ///
    /// A pure constant is a exported literal.
    ///
    ///
    /// TODO: Support pattern like
    ///     export const [a, b] = [1, 2]
    pub(super) fn extract_export_info(&self, module: &mut Module) -> RawExports {
        self.swc.run(|| {
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

            v.info
        })
    }
}

#[derive(Debug, Default)]
pub(super) struct RawExports {
    pub pure_constants: Vec<(Id, Lit)>,
    /// Key is None if it's exported from the module itself.
    pub items: FxHashMap<Option<Str>, Vec<ExportSpec>>,
}

#[derive(Debug, Default)]
pub(super) struct Exports {
    pub pure_constants: Vec<(Id, Lit)>,
    /// Key is None if it's exported from the module itself.
    pub items: FxHashMap<Option<Source>, Vec<ExportSpec>>,
}

#[derive(Debug, Default)]
pub struct ExportSpec {
    pub specifier: Vec<Specifier>,
    pub src: Option<Str>,
}

#[derive(Debug, Default)]
struct ExportFinder {
    info: RawExports,
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

            ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(decl)) => {
                self.info.items.entry(None).or_default().push(decl.ident());

                return ModuleItem::Stmt(Stmt::Decl(decl.decl));
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(decl)) => {
                let i = private_ident!("_default");
                self.info.items.entry(None).or_default().push(i.clone());
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(expr)) => {
                let i = private_ident!("_default");

                self.info.items.entry(None).or_default().push(i.clone());
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named)) => {
                self.info
                    .items
                    .entry(named.src)
                    .or_default()
                    .push(i.clone());
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportAll(all)) => {}

            _ => {}
        }

        item
    }
}
