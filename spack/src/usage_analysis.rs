use crate::Bundler;
use std::mem::replace;
use swc_common::{util::move_map::MoveMap, Fold, FoldWith, Mark, Span, Spanned};
use swc_ecma_ast::*;
use swc_ecma_utils::StmtLike;

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
        };

        node.fold_with(&mut v)
    }
}

#[derive(Debug)]
pub(super) struct UsageTracker {
    pass_cnt: usize,
    /// Applied to used nodes.
    mark: Mark,
    /// If it's none, all statement with side-effect will marked as used.
    changed: Option<Vec<Ident>>,
}

impl<T> Fold<Vec<T>> for UsageTracker
where
    T: StmtLike,
    T: FoldWith<Self>,
{
    fn fold(&mut self, items: Vec<T>) -> Vec<T> {
        let parent_cnt = self.pass_cnt;
        let upper_changed = replace(&mut self.changed, Some(vec![]));

        self.pass_cnt += 1;
        let mut items = items.fold_children(self);

        loop {
            if self.changed.is_some() && self.changed.as_ref().unwrap().is_empty() {
                break;
            }

            self.pass_cnt += 1;
            self.changed = Some(vec![]);
            items = items.fold_children(self)
        }

        log::debug!("Ran UsageTracker for {} times", self.pass_cnt);

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
