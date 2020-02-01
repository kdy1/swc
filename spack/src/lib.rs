#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

use crate::{import::ImportInfo, loader::Load};
use anyhow::Error;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::{errors::Handler, FileName, SourceFile, SourceMap};
use swc_ecma_ast::{Module, Program, Str};

mod import;
pub mod loader;
pub mod plugin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub tree_shake: bool,
}

pub struct Bundler {
    working_dir: PathBuf,
    config: Config,

    /// Javascript compiler.
    jsc: swc::Compiler,
    jsc_options: Arc<swc::config::Options>,

    module_loader: Box<dyn Load + Sync>,
}

impl Bundler {
    pub fn new(
        cm: Arc<SourceMap>,
        handler: Arc<Handler>,
        working_dir: PathBuf,
        swc: Arc<swc::config::Options>,
        module_loader: Box<dyn Load + Sync>,
    ) -> Self {
        Bundler {
            working_dir,
            config: Config { tree_shake: true },
            jsc: swc::Compiler::new(cm, handler),
            jsc_options: swc,
            module_loader,
        }
    }

    pub fn bundle(&self, entries: &[PathBuf]) -> Vec<Result<(Arc<SourceFile>, Module), Error>> {
        entries
            .into_par_iter()
            .map(|entry| -> Result<_, Error> {
                let (fm, module) = self.load_entry_file(entry)?;
                let module = self.transform_module(fm.clone(), module)?;

                Ok((fm, module))
            })
            .collect()
    }

    fn transform_module(&self, fm: Arc<SourceFile>, mut module: Module) -> Result<Module, Error> {
        let imports = self.extract_imports(&mut module);

        let (module, deps) = rayon::join(
            || -> Result<_, Error> {
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
                self.load_imports(&fm.name, imports)
            },
        );

        deps?;
        let module = module?;

        Ok(module)
    }

    fn load_imports(&self, base: &FileName, info: ImportInfo) -> Result<(), Error> {
        let ImportInfo {
            imports,
            requires,
            partial_requires,
            dynamic_imports,
        } = info;

        let ((a, b), (c, d)) = rayon::join(
            || {
                rayon::join(
                    || {
                        // imports
                        imports
                            .into_par_iter()
                            .map(|import| self.load_dep(base, &import.src))
                            .collect::<Vec<_>>()
                    },
                    || {
                        // Partial requires
                        partial_requires
                            .into_par_iter()
                            .map(|require| self.load_dep(base, &require.src))
                            .collect::<Vec<_>>()
                    },
                )
            },
            || {
                rayon::join(
                    || {
                        // Requires
                        requires
                            .into_par_iter()
                            .map(|require| self.load_dep(base, &require))
                            .collect::<Vec<_>>()
                    },
                    || {
                        // Dynamic imports
                        dynamic_imports
                            .into_par_iter()
                            .map(|require| self.load_dep(base, &require))
                            .collect::<Vec<_>>()
                    },
                )
            },
        );

        Ok(())
    }

    pub fn load_dep(&self, base: &FileName, s: &Str) -> Result<(Arc<SourceFile>, Module), Error> {
        let base = match base {
            FileName::Real(ref path) => path,
            _ => unreachable!(),
        };

        self.module_loader.load(&base, &s.value)
    }

    pub fn load_entry_file(&self, path: &Path) -> Result<(Arc<SourceFile>, Module), Error> {
        self.module_loader
            .load(&self.working_dir, &path.as_os_str().to_string_lossy())
    }

    pub fn jsc(&self) -> &swc::Compiler {
        &self.jsc
    }
}
