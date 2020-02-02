use self::scope::Scope;
use crate::{
    bundler::load_transformed::TransformedModule, load::Load, resolve::Resolve, Config, ModuleId,
};
use anyhow::{Context, Error};
use petgraph::{dot::Dot, graphmap::DiGraphMap};
use rayon::prelude::*;
use std::{path::PathBuf, sync::Arc};
use swc_common::{Mark, SourceFile};
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
    swc: Arc<swc::Compiler>,
    swc_options: swc::config::Options,

    resolver: Box<dyn Resolve + Sync>,
    loader: Box<dyn Load + Sync>,

    /// Mark for used statements
    used_mark: Mark,

    scope: Scope,
}

type ModuleGraph = DiGraphMap<ModuleId, u32>;

impl Bundler {
    pub fn new(
        working_dir: PathBuf,
        swc: Arc<swc::Compiler>,
        swc_options: swc::config::Options,
        resolver: Box<dyn Resolve + Sync>,
        loader: Box<dyn Load + Sync>,
    ) -> Self {
        let used_mark = swc.run(|| Mark::fresh(Mark::root()));
        Bundler {
            working_dir,
            config: Config { tree_shake: true },
            swc,
            swc_options,
            loader,
            resolver,
            scope: Default::default(),
            used_mark,
        }
    }

    fn add(&self, graph: &mut ModuleGraph, info: &TransformedModule) -> ModuleId {
        if graph.contains_node(info.0) {
            return info.0;
        }

        let node = graph.add_node(info.0);

        let v = &info.3;
        for src in &v.side_effect_imports {
            let to = self.add_module(graph, src.module_id);

            graph.add_edge(node, to, 1);
        }

        for (_, src) in &v.ids {
            let to = self.add_module(graph, src.module_id);

            graph.add_edge(node, to, 1);
        }

        node
    }

    fn add_module(&self, graph: &mut ModuleGraph, id: ModuleId) -> ModuleId {
        let v = self.scope.get_module(id).unwrap();
        self.add(graph, &(id, v.0, v.1, v.2))
    }

    pub fn bundle(&self, entries: &[PathBuf]) -> Vec<Result<(Arc<SourceFile>, Module), Error>> {
        let results = entries
            .into_par_iter()
            .map(|entry: &PathBuf| -> Result<_, Error> {
                Ok(self.load_transformed(&self.working_dir, &entry.to_string_lossy())?)
            })
            .collect::<Vec<_>>();

        let mut graph = ModuleGraph::default();

        let mut infos = Vec::with_capacity(results.len());
        for res in results {
            let info: TransformedModule = res.context("failed to load module").unwrap();

            self.add(&mut graph, &info);

            infos.push(info);
        }

        println!("{}", Dot::with_config(&graph.into_graph::<usize>(), &[]));

        unimplemented!()
    }

    pub fn swc(&self) -> &swc::Compiler {
        &self.swc
    }
}
