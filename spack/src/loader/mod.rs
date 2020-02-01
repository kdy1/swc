use anyhow::Error;
use std::{path::PathBuf, sync::Arc};
use swc_common::{SourceFile, SourceMap};
use swc_ecma_ast::Module;

/// Implementors of [Load] should not try parallel loading.
pub trait Load {
    fn load(
        &self,
        cm: &Arc<SourceMap>,
        base: &PathBuf,
        import: &str,
    ) -> Result<(Arc<SourceFile>, Module), Error>;
}

impl<T: ?Sized + Load> Load for Box<T> {
    fn load(
        &self,
        cm: &Arc<SourceMap>,
        base: &PathBuf,
        import: &str,
    ) -> Result<(Arc<SourceFile>, Module), Error> {
        T::load(self, cm, base, import)
    }
}

impl<'a, T: ?Sized + Load> Load for &'a mut T {
    fn load(
        &self,
        cm: &Arc<SourceMap>,
        base: &PathBuf,
        import: &str,
    ) -> Result<(Arc<SourceFile>, Module), Error> {
        T::load(self, cm, base, import)
    }
}

struct Resolver<L: Load> {
    inner: L,
}

pub trait Resolve {}
