#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

use crate::loader::Load;
use anyhow::Error;
use derive_builder::Builder;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use swc_ecma_ast::Module;

mod import;
pub mod loader;
pub mod plugin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub tree_shake: bool,
}

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Bundler {
    working_dir: PathBuf,
    config: Config,

    module_loader: Box<dyn Load>,
}

impl Bundler {
    pub fn build(&self, entries: &[PathBuf]) -> Vec<Result<Module, Error>> {
        entries
            .into_par_iter()
            .map(|entry| -> Result<Module, Error> {
                let mut module = self.load_entry_file(entry)?;
                let imports = self.extract_imports(&mut module);
            })
            .collect()
    }

    pub fn load_entry_file(&self, path: &Path) -> Result<Module, Error> {
        self.module_loader
            .load(&self.working_dir, &path.as_os_str().to_string_lossy())
    }
}
