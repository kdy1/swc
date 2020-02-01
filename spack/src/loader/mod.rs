use anyhow::Error;
use std::{path::Path, sync::Arc};
use swc_common::SourceFile;
use swc_ecma_ast::Module;

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

struct Resolver<L: Load> {
    inner: L,
}

pub trait Resolve {}
