#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]

pub use self::id::{Id, ModuleId, QualifiedId};
use self::{load::Load, scope::Scope};
use crate::{id::ModuleIdGenerator, resolve::Resolve};
use anyhow::Error;
use rayon::prelude::*;
use std::{path::PathBuf, sync::Arc};
use swc_common::{errors::Handler, Mark, SourceFile, SourceMap};
use swc_ecma_ast::Module;

mod export;
mod id;
mod import;
pub mod load;
pub mod resolve;
mod scope;
mod transform;
mod usage_analysis;

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

    resolver: Box<dyn Resolve + Sync>,
    loader: Box<dyn Load + Sync>,

    /// Mark for used statements
    used_mark: Mark,

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
            used_mark: Mark::fresh(Mark::root()),
        }
    }

    pub fn bundle(&self, entries: &[PathBuf]) -> Vec<Result<(Arc<SourceFile>, Module), Error>> {
        entries
            .into_par_iter()
            .map(|entry: &PathBuf| -> Result<_, Error> {
                let (_, fm, module, imports) =
                    self.load_transformed(&self.working_dir, &entry.to_string_lossy())?;

                let module = self.mark_all_as_used((*module).clone())?;

                Ok((fm, module))
            })
            .collect()
    }

    fn mark_all_as_used(&self, module: Module) -> Result<Module, Error> {
        let module = self.drop_unused(module, None);

        Ok(module)
    }

    pub fn swc(&self) -> &swc::Compiler {
        &self.swc
    }
}
