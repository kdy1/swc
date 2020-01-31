use anyhow::Error;
use std::path::PathBuf;
use swc_ecma_ast::Module;

pub trait Load {
    fn load(&self, base: &PathBuf, import: &str) -> Result<Module, Error>;
}

impl<T: ?Sized + Load> Load for Box<T> {
    fn load(&self, base: &PathBuf, import: &str) -> Result<Module, Error> {
        T::load(self, base, import)
    }
}

impl<'a, T: ?Sized + Load> Load for &'a mut T {
    fn load(&self, base: &PathBuf, import: &str) -> Result<Module, Error> {
        T::load(self, base, import)
    }
}

struct Resolver<L: Load> {
    inner: L,
}

pub trait Resolve {}
