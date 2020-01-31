use anyhow::Error;
use std::path::PathBuf;
use swc_ecma_ast::Module;

pub trait Loader {
    fn load(&mut self, base: &PathBuf) -> Result<Module, Error>;
}

pub struct Swc {}
