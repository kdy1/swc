#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

use crate::loader::Load;
use anyhow::Error;
use derive_builder::Builder;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::SourceFile;
use swc_ecma_ast::{Module, Program};

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

    /// Javascript compiler.
    jsc: swc::Compiler,
    jsc_options: swc::config::Options,

    module_loader: Box<dyn Load + Sync>,
}

impl Bundler {
    pub fn build(&self, entries: &[PathBuf]) -> Vec<Result<Module, Error>> {
        entries
            .into_par_iter()
            .map(|entry| -> Result<Module, Error> {
                let (fm, module) = self.load_entry_file(entry)?;
                let module = self.transform_module(fm.clone(), module)?;

                Ok(module)
            })
            .collect()
    }

    fn transform_module(&self, fm: Arc<SourceFile>, mut module: Module) -> Result<Module, Error> {
        let imports = self.extract_imports(&mut module);

        let (module, deps) = rayon::join(
            || {
                // Process module
                let config = self.jsc.config_for_file(&self.jsc_options, &*fm)?;

                let program = self.jsc.transform(
                    Program::Module(module),
                    config.external_helpers,
                    config.pass,
                );
                match program {
                    Program::Module(module) => Ok(module),
                    _ => unreachable!(),
                }
            },
            || {
                // Load dependencies
                imports.into_par_iter()
            },
        );

        let module = module?;

        Ok(module)
    }

    pub fn load_entry_file(&self, path: &Path) -> Result<(Arc<SourceFile>, Module), Error> {
        self.module_loader.load(
            &self.jsc.cm,
            &self.working_dir,
            &path.as_os_str().to_string_lossy(),
        )
    }
}
