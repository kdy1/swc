use self::scope::Scope;
use crate::{
    bundler::load_transformed::TransformedModule, id::ModuleIdGenerator, load::Load,
    resolve::Resolve, Config, ModuleId,
};
use anyhow::{Context, Error};
use petgraph::{dot::Dot, Graph};
use rayon::prelude::*;
use std::{path::PathBuf, sync::Arc};
use swc_common::{errors::Handler, Mark, SourceFile, SourceMap};
use swc_ecma_ast::Module;

mod export;
mod import_analysis;
mod import_handler;
mod load_transformed;
mod scope;
mod usage_analysis;

pub struct Bundler {
    working_dir: PathBuf,
    config: Config,

    /// Javascript compiler.
    swc: swc::Compiler,
    swc_options: swc::config::Options,

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
        swc: swc::config::Options,
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
        fn add(graph: &mut Graph<String, usize>, info: &TransformedModule) {
            graph.add_node(info.1.name.to_string());

            let v = &info.3;
            for src in &v.side_effect_imports {
                graph.add_node(src.src.value.to_string());
            }

            for (_, src) in &v.ids {
                graph.add_node(src.src.value.to_string());
            }
        }

        let results = entries
            .into_par_iter()
            .map(|entry: &PathBuf| -> Result<_, Error> {
                Ok(self.load_transformed(&self.working_dir, &entry.to_string_lossy())?)
            })
            .collect::<Vec<_>>();

        let mut graph = Graph::<String, usize>::new();

        for res in results {
            let info: TransformedModule = res.context("failed to load module").unwrap();

            add(&mut graph, &info);
        }

        println!("{}", Dot::with_config(&graph, &[]));

        unimplemented!()
    }

    pub fn swc(&self) -> &swc::Compiler {
        &self.swc
    }
}
