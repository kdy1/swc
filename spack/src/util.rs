use swc_common::{Fold, Mark, Span, SyntaxContext};
use swc_ecma_ast::Ident;

pub struct HygieneRemover;

impl Fold<Span> for HygieneRemover {
    fn fold(&mut self, s: Span) -> Span {
        s.with_ctxt(SyntaxContext::empty())
    }
}
