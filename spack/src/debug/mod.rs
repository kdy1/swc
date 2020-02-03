use swc_common::Fold;
use swc_ecma_ast::Ident;

pub struct HygieneVisualizer;

impl Fold<Ident> for HygieneVisualizer {
    fn fold(&mut self, node: Ident) -> Ident {
        Ident {
            sym: format!("{}{:?}", node.sym, node.span.ctxt()).into(),
            ..node
        }
    }
}
