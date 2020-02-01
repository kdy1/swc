#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

pub use self::id::{Id, ModuleId, QualifiedId};
use self::{analysis::ImportInfo, load::Load, scope::Scope};
use crate::{id::ModuleIdGenerator, resolve::Resolve, transform::TransformedModule};
use anyhow::Error;
use dashmap::DashMap;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::{errors::Handler, SourceFile, SourceMap};
use swc_ecma_ast::Module;

mod analysis;
mod id;
pub mod load;
pub mod resolve;
mod scope;
mod transform;

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

    module_id_gen: ModuleIdGenerator,
    cache: DashMap<Arc<PathBuf>, TransformedModule>,

    resolver: Box<dyn Resolve + Sync>,
    loader: Box<dyn Load + Sync>,

    scope: Scope,
}

impl Bundler {
    pub fn new(
        cm: Arc<SourceMap>,
        handler: Arc<Handler>,
        working_dir: PathBuf,
        swc: Arc<swc::config::Options>,
        resolver: Box<dyn Resolve + Sync>,
        loader: Box<dyn Load + Sync>,
    ) -> Self {
        Bundler {
            working_dir,
            config: Config { tree_shake: true },
            swc: swc::Compiler::new(cm, handler),
            swc_options: swc,
            loader,
            resolver,
            scope: Default::default(),
            module_id_gen: Default::default(),
            cache: Default::default(),
        }
    }

    pub fn bundle(
        &self,
        entries: &[PathBuf],
    ) -> Vec<Result<(Arc<SourceFile>, Arc<Module>), Error>> {
        entries
            .into_par_iter()
            .map(|entry: &PathBuf| -> Result<_, Error> {
                let (_, fm, module) =
                    self.load_transformed(&self.working_dir, &entry.to_string_lossy())?;

                Ok((fm, module))
            })
            .collect()
    }

    fn load_imports(&self, base: &Path, info: ImportInfo) -> Result<(), Error> {
        log::trace!("load_imports({})", base.display());

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
                            .map(|import| self.load_transformed(base, &import.src.value))
                            .collect::<Vec<_>>()
                    },
                    || {
                        // Partial requires
                        partial_requires
                            .into_par_iter()
                            .map(|require| self.load_transformed(base, &require.src.value))
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
                            .map(|require| self.load_transformed(base, &require.value))
                            .collect::<Vec<_>>()
                    },
                    || {
                        // Dynamic imports
                        dynamic_imports
                            .into_par_iter()
                            .map(|require| self.load_transformed(base, &require.value))
                            .collect::<Vec<_>>()
                    },
                )
            },
        );
        Ok(())
    }

    pub fn swc(&self) -> &swc::Compiler {
        &self.swc
    }
}
