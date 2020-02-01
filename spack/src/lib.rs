#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

pub use self::scope::{Id, ModuleId, QualifiedId};
use self::{analysis::ImportInfo, loader::Load, scope::Scope};
use anyhow::Error;
use dashmap::DashMap;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::{errors::Handler, FileName, SourceFile, SourceMap, SyntaxContext};
use swc_ecma_ast::{Lit, Module, Program, Str};

mod analysis;
pub mod loader;
pub mod plugin;
mod scope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub tree_shake: bool,
}

pub struct Bundler {
    working_dir: PathBuf,
    config: Config,

    /// Javascript compiler.
    swc: swc::Compiler,
    swc_options: Arc<swc::config::Options>,

    module_loader: Box<dyn Load + Sync>,

    scope: Scope,
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
            swc: swc::Compiler::new(cm, handler),
            swc_options: swc,
            module_loader,
            scope: Default::default(),
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
        log::trace!("transform_module({})", fm.name);

        let info = self.extract_info(&mut module);
        self.store_pure_constants(ModuleId::from(&*fm), info.exports.pure_constants);
        let imports = info.imports;

        let (module, deps) = rayon::join(
            || -> Result<_, Error> {
                self.swc.run(|| {
                    // Process module
                    let config = self.swc.config_for_file(&self.swc_options, &*fm)?;

                    let program = self.swc.transform(
                        Program::Module(module),
                        config.external_helpers,
                        config.pass,
                    );
                    match program {
                        Program::Module(module) => Ok(module),
                        _ => unreachable!(),
                    }
                })
            },
            || {
                self.swc.run(|| {
                    // Load dependencies
                    self.load_imports(&fm.name, imports)
                })
            },
        );

        deps?;
        let module = module?;

        Ok(module)
    }

    fn load_imports(&self, base: &FileName, info: ImportInfo) -> Result<(), Error> {
        log::trace!("load_imports({})", base);

        let ImportInfo {
            imports,
            requires,
            partial_requires,
            dynamic_imports,
        } = info;

        rayon::join(
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

    fn load_dep(&self, base: &FileName, s: &Str) -> Result<(Arc<SourceFile>, Module), Error> {
        log::trace!("load_dep({}) -> {}", base, s.value);

        let base = match base {
            FileName::Real(ref path) => path,
            _ => unreachable!(),
        };
        let (fm, module) = self.module_loader.load(&base, &s.value)?;
        let module = self.transform_module(fm.clone(), module)?;

        Ok((fm, module))
    }

    fn load_entry_file(&self, path: &Path) -> Result<(Arc<SourceFile>, Module), Error> {
        self.module_loader
            .load(&self.working_dir, &path.as_os_str().to_string_lossy())
    }

    pub fn swc(&self) -> &swc::Compiler {
        &self.swc
    }
}
