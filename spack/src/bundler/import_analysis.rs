use super::Bundler;
use std::mem::replace;
use swc_atoms::js_word;
use swc_common::{util::move_map::MoveMap, Fold, FoldWith};
use swc_ecma_ast::*;
use swc_ecma_utils::find_ids;

impl Bundler {
    /// This methods removes import statements (statements like `import a as b
    /// from 'foo'`) from module, but require calls and dynamic imports
    /// remain as-is.
    ///
    /// This method also drops empty statements from the module.
    pub(super) fn extract_import_info(&self, module: &mut Module) -> ImportInfo {
        let body = replace(&mut module.body, vec![]);

        let mut v = ImportFinder {
            top_level: false,
            info: Default::default(),
        };
        let body = body.fold_with(&mut v);
        module.body = body;

        v.info
    }
}

#[derive(Debug, Default)]
pub(super) struct ImportInfo {
    /// Unconditional imports. This includes require on top level.
    pub imports: Vec<ImportDecl>,

    /// Non-top-level imports.
    ///
    /// # Examaple
    ///
    /// ```js
    /// try{
    ///     const { watch } = require('watcher');
    /// } catch (e) {
    /// }
    /// ```
    pub lazy_imports: Vec<ImportDecl>,
    pub dynamic_imports: Vec<Str>,
}

struct ImportFinder {
    top_level: bool,
    info: ImportInfo,
}

impl Fold<Vec<ModuleItem>> for ImportFinder {
    fn fold(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        self.top_level = true;
        items.move_flat_map(|item| {
            //

            match item {
                ModuleItem::Stmt(Stmt::Empty(..)) => None,
                ModuleItem::Stmt(Stmt::Decl(Decl::Var(mut var))) => {
                    var.decls.retain(|d| match d.name {
                        Pat::Invalid(..) => false,
                        _ => true,
                    });

                    if var.decls.is_empty() {
                        None
                    } else {
                        Some(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var))))
                    }
                }

                ModuleItem::ModuleDecl(ModuleDecl::Import(i)) => {
                    self.info.imports.push(i);
                    None
                }

                _ => Some(item.fold_with(self)),
            }
        })
    }
}

impl Fold<Vec<Stmt>> for ImportFinder {
    fn fold(&mut self, items: Vec<Stmt>) -> Vec<Stmt> {
        self.top_level = false;
        items.fold_children(self)
    }
}

impl Fold<Expr> for ImportFinder {
    fn fold(&mut self, e: Expr) -> Expr {
        let e: Expr = e.fold_children(self);

        match e {
            Expr::Call(e) if e.args.len() == 1 => {
                let src = match e.args.first().unwrap() {
                    ExprOrSpread {
                        spread: None,
                        expr: box Expr::Lit(Lit::Str(s)),
                    } => s,
                    _ => return Expr::Call(e),
                };

                match e.callee {
                    ExprOrSuper::Expr(box Expr::Ident(Ident {
                        span,
                        sym: js_word!("require"),
                        ..
                    })) => {
                        let decl = ImportDecl {
                            span,
                            specifiers: vec![],
                            src: src.clone(),
                        };

                        // if self.top_level {
                        //     self.info.imports.push(decl);
                        //     return *undefined(span);
                        // }

                        self.info.lazy_imports.push(decl);
                        return Expr::Call(e);
                    }

                    ExprOrSuper::Expr(box Expr::Ident(Ident {
                        sym: js_word!("import"),
                        ..
                    })) => {
                        self.info.dynamic_imports.push(src.clone());
                    }

                    _ => {}
                }

                return Expr::Call(e);
            }

            _ => {}
        }

        e
    }
}

/// ```js
/// const { readFile } = required('fs');
/// ```
///
/// is treated as
///
///  ```js
/// import { readFile } from 'fs';
/// ```
impl Fold<VarDeclarator> for ImportFinder {
    fn fold(&mut self, node: VarDeclarator) -> VarDeclarator {
        match node.init {
            Some(box Expr::Call(CallExpr {
                span,
                callee:
                    ExprOrSuper::Expr(box Expr::Ident(Ident {
                        sym: js_word!("require"),
                        ..
                    })),
                ref args,
                ..
            })) if args.len() == 1 => {
                let src = match args.first().unwrap() {
                    ExprOrSpread {
                        spread: None,
                        expr: box Expr::Lit(Lit::Str(s)),
                    } => s.clone(),
                    _ => return node,
                };

                let ids: Vec<Ident> = find_ids(&node.name);

                let decl = ImportDecl {
                    span,
                    specifiers: ids
                        .into_iter()
                        .map(|ident| {
                            ImportSpecifier::Specific(ImportSpecific {
                                span,
                                local: ident,
                                imported: None,
                            })
                        })
                        .collect(),
                    src,
                };

                // if self.top_level {
                //     self.info.imports.push(decl);
                //     node.init = None;
                //     node.name = Pat::Invalid(Invalid { span: DUMMY_SP });
                //     return node;
                // }

                self.info.lazy_imports.push(decl);

                return VarDeclarator {
                    name: node.name.fold_with(self),
                    ..node
                };
            }

            _ => {}
        }

        node.fold_children(self)
    }
}
