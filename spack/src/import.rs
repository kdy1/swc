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
    pub(super) fn extract_info(&self, module: &mut Module) -> ImportInfo {
        let body = replace(&mut module.body, vec![]);

        let mut v = ImportFinder::default();
        let body = body.fold_with(&mut v);
        module.body = body;

        v.info
    }
}

#[derive(Default)]
pub(super) struct ImportInfo {
    pub imports: Vec<ImportDecl>,
    pub requires: Vec<Str>,
    pub partial_requires: Vec<ImportDecl>,
    pub dynamic_imports: Vec<Str>,
}

#[derive(Default)]
struct ImportFinder {
    info: ImportInfo,
}

impl Fold<Vec<ModuleItem>> for ImportFinder {
    fn fold(&mut self, items: Vec<ModuleItem>) -> Vec<ModuleItem> {
        items.move_flat_map(|item| match item {
            ModuleItem::ModuleDecl(ModuleDecl::Import(i)) => {
                self.info.imports.push(i);
                None
            }

            _ => Some(item.fold_with(self)),
        })
    }
}

impl Fold<CallExpr> for ImportFinder {
    fn fold(&mut self, node: CallExpr) -> CallExpr {
        if node.args.len() != 1 {
            return node;
        }
        let src = match node.args.first().unwrap() {
            ExprOrSpread {
                spread: None,
                expr: box Expr::Lit(Lit::Str(s)),
            } => s,
            _ => return node,
        };

        match node.callee {
            ExprOrSuper::Expr(box Expr::Ident(Ident {
                sym: js_word!("require"),
                ..
            })) => {
                self.info.requires.push(src.clone());
                return node;
            }

            ExprOrSuper::Expr(box Expr::Ident(Ident {
                sym: js_word!("import"),
                ..
            })) => {
                self.info.dynamic_imports.push(src.clone());
                return node;
            }

            _ => {}
        }

        node.fold_children(self)
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

                self.info.partial_requires.push(ImportDecl {
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
                });

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
