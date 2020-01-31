use super::Bundler;
use std::mem::replace;
use swc_common::{util::move_map::MoveMap, Fold, FoldWith};
use swc_ecma_ast::*;

impl Bundler {
    /// This methods removes import statements (statements like `import a as b
    /// from 'foo'`) from module, but require calls and dynamic imports
    /// remain as-is.
    pub(super) fn extract_imports(&self, module: &mut Module) -> ImportInfo {
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
    pub requires: Vec<ImportDecl>,
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

impl Fold<CallExpr> for ImportFinder {}

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
                callee:
                    ExprOrSuper::Expr(box Expr::Ident(Ident {
                        sym: js_word!("require"),
                        ..
                    })),
                ..
            })) => {}
        }

        node.fold_children(self)
    }
}
