use self::scope::Scope;
use crate::{bundler::load_transformed::TransformedModule, load::Load, resolve::Resolve, Config};
use anyhow::{Context, Error};
use rayon::prelude::*;
use std::{path::PathBuf, sync::Arc};
use swc_common::{Mark, SourceFile};
use swc_ecma_ast::Module;

mod export;
mod import_analysis;
mod load_transformed;
mod merge;
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

    scope: Scope,
}

impl Bundler {
    pub fn new(
        working_dir: PathBuf,
        swc: Arc<swc::Compiler>,
        swc_options: swc::config::Options,
        resolver: Box<dyn Resolve + Sync>,
        loader: Box<dyn Load + Sync>,
    ) -> Self {
        Bundler {
            working_dir,
            config: Config { tree_shake: true },
            swc,
            swc_options,
            loader,
            resolver,
            scope: Default::default(),
        }
    }

    /*fn add(&self, graph: &mut ModuleGraph, info: &TransformedModule) -> ModuleId {
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
    }*/

    pub fn bundle(&self, entries: &[PathBuf]) -> Vec<Result<(Arc<SourceFile>, Module), Error>> {
        let results = entries
            .into_par_iter()
            .map(|entry: &PathBuf| -> Result<_, Error> {
                Ok(self
                    .load_transformed(&self.working_dir, &entry.to_string_lossy())
                    .context("load_transformed failed")?)
            })
            .collect::<Vec<_>>();

        // We collect at here to handle dynamic imports
        // TODO: Handle dynamic imports

        let mut output = vec![];
        for res in results {
            let res: Result<_, Error> = try {
                let m: TransformedModule = res?;
                let module = self
                    .merge_modules((*m.module).clone(), &m)
                    .context("failed to merge module")?;

                (m.fm, module)
            };

            output.push(res);
        }

        //
        //        let mut entries = Vec::with_capacity(entries.len());
        //
        //        {
        //            for res in results {
        //                let m: TransformedModule = res?;
        //
        //                if m.3.is_dynamic {}
        //            }
        //        }
        // We does not support chunking yet.
        // TODO: chunk
        //        let mut graph = ModuleGraph::default();
        //        let mut infos = Vec::with_capacity(results.len());
        //        for res in results {
        //            let info: TransformedModule = res.unwrap();
        //            self.add(&mut graph, &info);
        //            infos.push(info);
        //        }

        output
    }

    pub fn swc(&self) -> &swc::Compiler {
        &self.swc
    }
}
