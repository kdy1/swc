use crate::{Bundler, Id};
use swc_common::{util::move_map::MoveMap, FileName, Fold, FoldWith, Mark, Span, Spanned};
use swc_ecma_ast::*;
use swc_ecma_transforms::{resolver, resolver::Resolver};
use swc_ecma_utils::{find_ids, ExprExt, StmtLike};

impl Bundler {
    /// If used_exports is [None], all exports are treated as exported.
    pub(super) fn drop_unused<T>(&self, path: FileName, node: T, used_exports: Option<Vec<Id>>) -> T
    where
        T: FoldWith<UsageTracker> + for<'any> FoldWith<Resolver<'any>>,
    {
        let mut v = UsageTracker {
            path,
            pass_cnt: 0,
            mark: self.used_mark,
            included: Default::default(),
            used_exports,
            marking_phase: false,
        };

        self.swc
            .run(|| node.fold_with(&mut resolver()).fold_with(&mut v))
    }
}

#[derive(Debug)]
pub(super) struct UsageTracker {
    pass_cnt: usize,
    /// Applied to used nodes.
    mark: Mark,

    /// Identifiers which should be emitted.
    included: Vec<Id>,

    used_exports: Option<Vec<Id>>,

    /// If true, idents are added to [changed].
    marking_phase: bool,
    path: FileName,
}

impl<T> Fold<Vec<T>> for UsageTracker
where
    T: StmtLike + FoldWith<Self> + Spanned + std::fmt::Debug,
{
    fn fold(&mut self, mut items: Vec<T>) -> Vec<T> {
        let parent_cnt = self.pass_cnt;
        //        let upper_changed = replace(&mut self.changed, Default::default());

        let mut len;
        loop {
            self.pass_cnt += 1;
            len = self.included.len();
            items = items.fold_children(self);
            if len == self.included.len() {
                break;
            }
        }

        log::debug!("UsageTracker: Ran {} times", self.pass_cnt);

        items = items.move_flat_map(|item| {
            if !self.is_marked(item.span()) {
                if cfg!(debug_assertions) {
                    println!("{}\n{:?}\nDropping {:?}", self.path, self.included, item);
                }

                return None;
            }
            Some(item)
        });

        //        self.changed = upper_changed;
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

        if self.included.is_empty() {
            return import;
        }

        let ids: Vec<Ident> = find_ids(&import.specifiers);

        for id in ids {
            for c in &self.included {
                if *c == id {
                    import.span = import.span.apply_mark(self.mark);
                    return import;
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
        // TODO: Export only when it's required. (i.e. check self.used_exports)

        node.span = node.span.apply_mark(self.mark);

        let old = self.marking_phase;
        self.marking_phase = true;
        node.decl = node.decl.fold_with(self);
        self.marking_phase = old;

        node
    }
}

impl Fold<ExportDefaultExpr> for UsageTracker {
    fn fold(&mut self, node: ExportDefaultExpr) -> ExportDefaultExpr {
        if self.is_marked(node.span) {
            return node;
        }

        unimplemented!()
    }
}

impl Fold<NamedExport> for UsageTracker {
    fn fold(&mut self, node: NamedExport) -> NamedExport {
        if self.is_marked(node.span) {
            return node;
        }

        unimplemented!()
    }
}

impl Fold<ExportDefaultDecl> for UsageTracker {
    fn fold(&mut self, node: ExportDefaultDecl) -> ExportDefaultDecl {
        if self.is_marked(node.span) {
            return node;
        }

        unimplemented!()
    }
}

impl Fold<ExportAll> for UsageTracker {
    fn fold(&mut self, node: ExportAll) -> ExportAll {
        if self.is_marked(node.span) {
            return node;
        }

        unimplemented!()
    }
}

impl Fold<ExprStmt> for UsageTracker {
    fn fold(&mut self, node: ExprStmt) -> ExprStmt {
        if self.is_marked(node.span) {
            return node;
        }

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
            log::debug!(
                "UsageTracker:{}\nMarking {}{:?} as used",
                self.path,
                i.sym,
                i.span.ctxt()
            );
            self.included.push((&i).into());
        }

        i
    }
}

impl Fold<VarDecl> for UsageTracker {
    fn fold(&mut self, var: VarDecl) -> VarDecl {
        if self.is_marked(var.span) {
            return var;
        }

        let var: VarDecl = var.fold_children(self);

        if self.included.is_empty() {
            return var;
        }

        let ids: Vec<Ident> = find_ids(&var.decls);

        for i in ids {
            for i1 in &self.included {
                if *i1 == i {
                    return VarDecl {
                        span: var.span.apply_mark(self.mark),
                        ..var
                    };
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
