use super::Bundler;
use crate::{
    bundler::load_transformed::{Source, Specifier},
    Id,
};
use fxhash::FxHashMap;
use std::mem::replace;
use swc_atoms::js_word;
use swc_common::{Fold, FoldWith, Spanned, SyntaxContext, DUMMY_SP};
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
    pub items: FxHashMap<Option<Str>, Vec<Specifier>>,
}

#[derive(Debug, Default)]
pub(super) struct Exports {
    pub pure_constants: Vec<(Id, Lit)>,
    /// Key is None if it's exported from the module itself.
    pub items: FxHashMap<Option<Source>, Vec<Specifier>>,
}

#[derive(Debug, Default)]
struct ExportFinder {
    info: RawExports,
}

impl Fold<ModuleItem> for ExportFinder {
    fn fold(&mut self, item: ModuleItem) -> ModuleItem {
        let span = item.span();

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
                self.info.items.entry(None).or_default().push({
                    let i = match decl.decl {
                        Decl::Class(ref c) => &c.ident,
                        Decl::Fn(ref f) => &f.ident,
                        Decl::Var(ref v) => {
                            return;
                        }
                        _ => unreachable!("Decl in ExportDecl: {:?}", decl.decl),
                    };
                    Specifier::Specific {
                        local: i.into(),
                        alias: None,
                    }
                });

                return ModuleItem::Stmt(Stmt::Decl(decl.decl));
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(decl)) => {
                let i = private_ident!("_default");

                self.info
                    .items
                    .entry(None)
                    .or_default()
                    .push(Specifier::Specific {
                        local: (&i).into(),
                        alias: Some(Id::new(js_word!("default"), SyntaxContext::empty())),
                    });

                let expr = match decl.decl {
                    DefaultDecl::Class(c) => box Expr::Class(c),
                    DefaultDecl::Fn(f) => box Expr::Fn(f),
                    DefaultDecl::TsInterfaceDecl(decl) => {
                        return ModuleItem::Stmt(Stmt::Decl(Decl::TsInterface(decl)))
                    }
                };

                return ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                    span: decl.span,
                    kind: VarDeclKind::Var,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(i),
                        init: Some(expr),
                        definite: false,
                    }],
                })));
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(expr)) => {
                let i = private_ident!("_default");

                self.info
                    .items
                    .entry(None)
                    .or_default()
                    .push(Specifier::Specific {
                        local: (&i).into(),
                        alias: Some(Id::new(js_word!("default"), SyntaxContext::empty())),
                    });

                return ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                    span: expr.span,
                    kind: VarDeclKind::Var,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(i),
                        init: Some(expr.expr),
                        definite: false,
                    }],
                })));
            }

            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named)) => {
                let mut v = self.info.items.entry(named.src).or_default();
                for s in named.specifiers {
                    match s {
                        ExportSpecifier::Namespace(n) => v.push(Specifier::Namespace {
                            local: n.name.into(),
                        }),
                        ExportSpecifier::Default(d) => {
                            v.push(Specifier::Specific {
                                local: d.exported.into(),
                                alias: Some(Id::new(js_word!("default"), SyntaxContext::empty())),
                            });
                        }
                        ExportSpecifier::Named(n) => {
                            v.push(Specifier::Specific {
                                local: n.orig.into(),
                                alias: n.exported.map(From::from),
                            });
                        }
                    }
                }
                return ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span }));
            }

            // ModuleItem::ModuleDecl(ModuleDecl::ExportAll(all)) => {}
            _ => {}
        }

        item
    }
}
