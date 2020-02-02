use super::Bundler;
use crate::bundler::import_analysis::ImportInfo;
use anyhow::Error;
use std::sync::Arc;
use swc_ecma_ast::Module;

impl Bundler {
    pub(super) fn handle_imports(
        &self,
        module: Module,
        imports: Arc<ImportInfo>,
    ) -> Result<Module, Error> {
    }
}
