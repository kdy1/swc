use anyhow::Error;
use std::{path::Path, sync::Arc};
use swc_common::{errors::Handler, FileName, SourceFile, SourceMap};
use swc_ecma_ast::{Module, Program};

/// Implementors of [Load] should not try parallel loading.
pub trait Load {
    fn load(&self, base: &Path, import: &str) -> Result<(Arc<SourceFile>, Module), Error>;
}

impl<T: ?Sized + Load> Load for Box<T> {
    fn load(&self, base: &Path, import: &str) -> Result<(Arc<SourceFile>, Module), Error> {
        T::load(self, base, import)
    }
}

impl<'a, T: ?Sized + Load> Load for &'a mut T {
    fn load(&self, base: &Path, import: &str) -> Result<(Arc<SourceFile>, Module), Error> {
        T::load(self, base, import)
    }
}

/// JavaScript loader
pub struct JsLoader<R = Resolver> {
    compiler: swc::Compiler,
    options: Arc<swc::config::Options>,
    resolver: R,
}

impl<R> JsLoader<R> {
    pub fn new(
        cm: Arc<SourceMap>,
        handler: Arc<Handler>,
        options: Arc<swc::config::Options>,
        resolver: R,
    ) -> Self {
        JsLoader {
            compiler: swc::Compiler::new(cm, handler),
            options,
            resolver,
        }
    }
}

impl<R> Load for JsLoader<R> {
    fn load(&self, base: &Path, import: &str) -> Result<(Arc<SourceFile>, Module), Error> {
        log::debug!("JsLoader.load({}) -> {}", base.display(), import);

        let path = node_resolve::Resolver::new()
            .with_basedir(base.parent().unwrap().into())
            .resolve(import)?;

        let fm = self.compiler.cm.load_file(&path)?;

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

pub struct Resolver {}

pub trait Resolve {}

impl Resolve for Resolver {}
