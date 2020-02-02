use crate::load::Load;
use anyhow::Error;
use std::{path::Path, sync::Arc};
use swc_common::{errors::Handler, SourceFile, SourceMap};
use swc_ecma_ast::{Module, Program};

/// JavaScript loader
pub struct JsLoader {
    compiler: swc::Compiler,
    options: Arc<swc::config::Options>,
}

impl JsLoader {
    pub fn new(
        cm: Arc<SourceMap>,
        handler: Arc<Handler>,
        options: Arc<swc::config::Options>,
    ) -> Self {
        JsLoader {
            compiler: swc::Compiler::new(cm, handler),
            options,
        }
    }
}

impl Load for JsLoader {
    fn load(&self, path: &Path) -> Result<(Arc<SourceFile>, Module), Error> {
        log::debug!("JsLoader.load({})", path.display());

        let fm = self.compiler.cm.load_file(path)?;

        log::trace!("JsLoader.load: loaded");

        let config = self.compiler.config_for_file(&self.options, &fm)?;

        log::trace!("JsLoader.load: loaded config");

        let program =
            self.compiler
                .parse_js(fm.clone(), config.target, config.syntax, true, true)?;

        log::trace!("JsLoader.load: parsed");

        match program {
            Program::Module(m) => Ok((fm, m)),
            Program::Script(_) => unreachable!(),
        }
    }
}
