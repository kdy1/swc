use crate::{
    bundler::load_transformed::TransformedModule, debug::HygieneVisualizer, util::HygieneRemover,
    Bundler, Id,
};
use std::sync::Arc;
use swc_common::{
    util::move_map::MoveMap, FileName, Fold, FoldWith, Mark, SourceFile, Span, Spanned, Visit,
    VisitWith,
};
use swc_ecma_ast::*;
use swc_ecma_utils::{find_ids, ExprExt, StmtLike};

impl Bundler {
    /// If used_exports is [None], all exports are treated as exported.
    pub(super) fn drop_unused(
        &self,
        fm: Arc<SourceFile>,
        node: Module,
        used_exports: Option<Vec<Id>>,
    ) -> Module {
        let mut v = UsageTracker {
            path: fm.name.clone(),
            pass_cnt: 0,
            mark: self.used_mark,
            included: Default::default(),
            changed: false,
            used_exports,
            marking_phase: false,
        };

        let node = self
            .swc
            .run(|| node.fold_with(&mut v).fold_with(&mut HygieneRemover));

        node
    }
}

#[derive(Debug)]
pub(super) struct UsageTracker {
    pass_cnt: usize,
    /// Applied to used nodes.
    mark: Mark,

    /// Identifiers which should be emitted.
    ///
    /// TODO: use FxHashSet for performance
    included: Vec<Id>,
    changed: bool,

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

        loop {
            log::trace!("UsageTracker running: {}", self.pass_cnt);

            self.pass_cnt += 1;
            self.changed = false;
            items = items.fold_children(self);
            if !self.changed {
                break;
            }
        }

        log::debug!("UsageTracker: Ran {} times", self.pass_cnt);

        items = items.move_flat_map(|item| {
            let item = match item.try_into_stmt() {
                Ok(stmt) => match stmt {
                    Stmt::Empty(..) => return None,
                    _ => T::from_stmt(stmt),
                },
                Err(item) => item,
            };

            if !self.is_marked(item.span()) {
                if cfg!(debug_assertions) {
                    log::info!("{}\n{:?}\nDropping {:?}", self.path, self.included, item);
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
        let mut ctxt = span.ctxt().clone();

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

    pub fn fold_in_marking_phase<T>(&mut self, node: T) -> T
    where
        T: FoldWith<Self>,
    {
        let old = self.marking_phase;
        self.marking_phase = true;
        let node = node.fold_with(self);
        self.marking_phase = old;

        node
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

        let i = match node.decl {
            Decl::Class(ClassDecl { ref ident, .. }) | Decl::Fn(FnDecl { ref ident, .. }) => ident,

            // Preserve types
            Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
                node.span = node.span.apply_mark(self.mark);
                return node;
            }

            // Preserve only exported variables
            Decl::Var(ref mut v) => {
                // TODO: Export only when it's required. (i.e. check self.used_exports)

                if let Some(ref exported_ids) = self.used_exports {
                    v.decls.retain(|d| {
                        let mut v = IdentListVisitor {
                            included_ids: &self.included,
                            exported_ids: &exported_ids,
                            found: false,
                        };

                        d.visit_with(&mut v);

                        v.found
                    });
                }

                if !v.decls.is_empty() {
                    node.span = node.span.apply_mark(self.mark);
                    node.decl = self.fold_in_marking_phase(node.decl);
                }
                return node;
            }
        };

        if self.used_exports.is_none()
            || self
                .used_exports
                .as_ref()
                .unwrap()
                .iter()
                .any(|exported| exported == i)
        {
            node.span = node.span.apply_mark(self.mark);
            node.decl = self.fold_in_marking_phase(node.decl);
        }

        node
    }
}

impl Fold<ExportDefaultExpr> for UsageTracker {
    fn fold(&mut self, mut node: ExportDefaultExpr) -> ExportDefaultExpr {
        if self.is_marked(node.span) {
            return node;
        }

        // TODO: Export only when it's required. (i.e. check self.used_exports)

        node.span = node.span.apply_mark(self.mark);
        node.expr = self.fold_in_marking_phase(node.expr);

        node
    }
}

impl Fold<NamedExport> for UsageTracker {
    fn fold(&mut self, mut node: NamedExport) -> NamedExport {
        if self.is_marked(node.span) {
            return node;
        }

        // TODO: Export only when it's required. (i.e. check self.used_exports)

        node.span = node.span.apply_mark(self.mark);
        node.specifiers = self.fold_in_marking_phase(node.specifiers);

        node
    }
}

impl Fold<ExportDefaultDecl> for UsageTracker {
    fn fold(&mut self, mut node: ExportDefaultDecl) -> ExportDefaultDecl {
        if self.is_marked(node.span) {
            return node;
        }

        // TODO: Export only when it's required. (i.e. check self.used_exports)

        node.span = node.span.apply_mark(self.mark);
        node.decl = self.fold_in_marking_phase(node.decl);

        node
    }
}

impl Fold<ExportAll> for UsageTracker {
    fn fold(&mut self, node: ExportAll) -> ExportAll {
        if self.is_marked(node.span) {
            return node;
        }

        // TODO: Export only when it's required. (i.e. check self.used_exports)

        unimplemented!("drop_unused: `export * from 'foo'`")
    }
}

impl Fold<ExprStmt> for UsageTracker {
    fn fold(&mut self, node: ExprStmt) -> ExprStmt {
        if self.is_marked(node.span) {
            return node;
        }

        if node.expr.may_have_side_effects() {
            log::trace!("UsageTracker: ExprStmt: Entering marking phase");

            let stmt = ExprStmt {
                span: node.span.apply_mark(self.mark),
                expr: self.fold_in_marking_phase(node.expr),
            };
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
            self.changed = true;
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
        if self.is_marked(e.span()) {
            return e;
        }

        e.obj = e.obj.fold_with(self);
        if e.computed {
            e.prop = e.prop.fold_with(self);
        }

        e
    }
}

impl Fold<FnDecl> for UsageTracker {
    fn fold(&mut self, mut f: FnDecl) -> FnDecl {
        if self.is_marked(f.span()) {
            return f;
        }

        if self.marking_phase || self.included.contains(&Id::from(&f.ident)) {
            f.function.span = f.function.span.apply_mark(self.mark);
        }

        f.fold_children(self)
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

#[derive(Debug)]
struct IdentListVisitor<'a> {
    included_ids: &'a [Id],
    exported_ids: &'a [Id],
    found: bool,
}

impl Visit<Ident> for IdentListVisitor<'_> {
    fn visit(&mut self, node: &Ident) {
        if self.found {
            return;
        }

        if self
            .included_ids
            .iter()
            .chain(self.exported_ids)
            .any(|i| i == node)
        {
            self.found = true;
            return;
        }
    }
}
