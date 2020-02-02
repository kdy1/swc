use crate::Bundler;
use std::mem::replace;
use swc_common::{util::move_map::MoveMap, Fold, FoldWith, Mark, Span, Spanned};
use swc_ecma_ast::*;
use swc_ecma_utils::{find_ids, ExprExt, Id, StmtLike};

impl Bundler {
    /// If used_exports is [None], all exports are treated as exported.
    pub(super) fn drop_unused<T>(&self, node: T, used_exports: Option<Vec<Ident>>) -> T
    where
        T: FoldWith<UsageTracker>,
    {
        let mut v = UsageTracker {
            pass_cnt: 0,
            mark: self.used_mark,
            changed: used_exports,
            marking_phase: false,
        };

        self.swc.run(|| node.fold_with(&mut v))
    }
}

#[derive(Debug)]
pub(super) struct UsageTracker {
    pass_cnt: usize,
    /// Applied to used nodes.
    mark: Mark,
    /// If it's none, all statement with side-effect will marked as used.
    changed: Option<Vec<Ident>>,

    /// If true, idents are added to [changed].
    marking_phase: bool,
}

impl<T> Fold<Vec<T>> for UsageTracker
where
    T: StmtLike + FoldWith<Self> + Spanned + std::fmt::Debug,
{
    fn fold(&mut self, items: Vec<T>) -> Vec<T> {
        let parent_cnt = self.pass_cnt;
        let upper_changed = replace(&mut self.changed, Some(vec![]));

        self.pass_cnt += 1;
        let mut items = items.fold_children(self);

        let mut len;
        loop {
            if self.changed.is_some() && self.changed.as_ref().unwrap().is_empty() {
                break;
            }

            self.pass_cnt += 1;
            len = self.changed.as_ref().map(|v| v.len()).unwrap_or(0);
            if self.changed.is_none() {
                self.changed = Some(vec![]);
            }
            items = items.fold_children(self);
            if len == self.changed.as_ref().unwrap().len() {
                break;
            }
        }

        log::debug!("UsageTracker: Ran {} times", self.pass_cnt);

        items = items.move_flat_map(|item| {
            if !self.is_marked(item.span()) {
                if cfg!(debug_assertions) {
                    println!("Dropping {:?}", item);
                }

                return None;
            }
            Some(item)
        });

        self.changed = upper_changed;
        self.pass_cnt = parent_cnt;

        items
    }
}

impl UsageTracker {
    pub fn is_marked(&self, span: Span) -> bool {
        let mut ctxt = span.ctxt();

        loop {
            let mark = ctxt.remove_mark();

            if mark == Mark::root() {
                return false;
            }

            if mark == self.mark {
                return true;
            }
        }
    }
}

impl Fold<ImportDecl> for UsageTracker {
    fn fold(&mut self, import: ImportDecl) -> ImportDecl {
        if self.is_marked(import.span) {
            return import;
        }

        let mut import: ImportDecl = import.fold_children(self);

        // TODO: Drop unused imports.
        //      e.g) import { foo, bar } from './foo';
        //           foo()

        if let Some(changed) = &self.changed {
            if changed.is_empty() {
                return import;
            }

            let ids: Vec<Id> = find_ids(&import.specifiers);

            println!(
                "=========================\n{:?}\n{:?}\n=========================",
                changed, ids,
            );

            for id in ids {
                for c in changed {
                    if c.sym == id.0 && c.span.ctxt() == id.1 {
                        import.span = import.span.apply_mark(self.mark);
                        return import;
                    }
                }
            }
        }

        if import.specifiers.is_empty() {
            import.span = import.span.apply_mark(self.mark);
        }

        import
    }
}

impl Fold<ExportDecl> for UsageTracker {
    fn fold(&mut self, mut node: ExportDecl) -> ExportDecl {
        if self.is_marked(node.span) {
            return node;
        }
        // TODO: Export only when it's required. (i.e. check self.changes)

        node.span = node.span.apply_mark(self.mark);

        let old = self.marking_phase;
        self.marking_phase = true;
        node.decl = node.decl.fold_with(self);
        self.marking_phase = old;

        node
    }
}

//impl Fold<ExportDefaultExpr> for UsageTracker {}
//
//impl Fold<ExportSpecifier> for UsageTracker {}

impl Fold<ExprStmt> for UsageTracker {
    fn fold(&mut self, node: ExprStmt) -> ExprStmt {
        if node.expr.may_have_side_effects() {
            log::trace!("UsageTracker: ExprStmt: Entering marking phase");

            let old = self.marking_phase;
            self.marking_phase = true;
            let stmt = ExprStmt {
                span: node.span.apply_mark(self.mark),
                expr: node.expr.fold_children(self),
            };
            self.marking_phase = old;
            return stmt;
        }

        node.fold_children(self)
    }
}

impl Fold<Ident> for UsageTracker {
    fn fold(&mut self, i: Ident) -> Ident {
        if self.is_marked(i.span) {
            return i;
        }

        if self.marking_phase {
            if let Some(ref mut vec) = self.changed {
                log::debug!("UsageTracker: Marking {} as used", i.sym);
                vec.push(i.clone());
            }
        }

        i
    }
}

impl Fold<VarDecl> for UsageTracker {
    fn fold(&mut self, var: VarDecl) -> VarDecl {
        let var: VarDecl = var.fold_children(self);

        if let Some(ref idents) = self.changed {
            if idents.is_empty() {
                return var;
            }

            let ids: Vec<Ident> = find_ids(&var.decls);

            for i in ids {
                for i1 in idents {
                    if i1.sym == i.sym {
                        return VarDecl {
                            span: var.span.apply_mark(self.mark),
                            ..var
                        };
                    }
                }
            }
        }

        var
    }
}

impl Fold<MemberExpr> for UsageTracker {
    fn fold(&mut self, mut e: MemberExpr) -> MemberExpr {
        e.obj = e.obj.fold_with(self);
        if e.computed {
            e.prop = e.prop.fold_with(self);
        }

        e
    }
}

macro_rules! simple {
    ($T:ty) => {
        impl Fold<$T> for UsageTracker {
            fn fold(&mut self, node: $T) -> $T {
                if self.is_marked(node.span()) {
                    return node;
                }

                node.fold_children(self)
            }
        }
    };
}

simple!(Stmt);
simple!(ModuleItem);
simple!(ModuleDecl);
