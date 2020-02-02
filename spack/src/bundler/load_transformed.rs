use super::Bundler;
use crate::{bundler::import_analysis::ImportInfo, Id, ModuleId};
use anyhow::{Context, Error};
use fxhash::FxHashMap;
use rayon::prelude::*;
use std::{path::Path, sync::Arc};
use swc_common::{FileName, SourceFile};
use swc_ecma_ast::{Module, Program, Str};

/// Module after applying transformations.
pub(super) type TransformedModule = (ModuleId, Arc<SourceFile>, Arc<Module>, Arc<MergedImports>);

#[derive(Debug, Default)]
pub(super) struct MergedImports {
    pub ids: FxHashMap<Id, Source>,
    pub side_effect_imports: Vec<Source>,
}

#[derive(Debug, Clone)]
pub(super) struct Source {
    pub module_id: ModuleId,
    // Clone is relatively cheap, thanks to string_cache.
    pub src: Str,
}

impl Bundler {
    /// Phase 1 (discovery)
    ///
    /// We apply transforms at this phase to make cache efficient.
    /// As we cache in this phase, changing dependency does not affect cache.  
    pub(super) fn load_transformed(
        &self,
        base: &Path,
        s: &str,
    ) -> Result<TransformedModule, Error> {
        let path = self
            .resolver
            .resolve(base, s)
            .context("failed to resolve")?;

        if let Some(cached) = self.scope.cache.get(&path) {
            return Ok(cached.clone());
        }

        let (id, fm, module) = self.load(&path).context("Bundler.load failed")?;

        let v = self
            .transform_module(id, fm.clone(), module)
            .context("failed to transform module")?;

        self.scope.cache.insert(Arc::new(path), v.clone());

        Ok(v)
    }

    fn load(&self, path: &Path) -> Result<(ModuleId, Arc<SourceFile>, Module), Error> {
        let module_id = self.module_id_gen.gen();

        let path = Arc::new(path);

        let (fm, module) = self.loader.load(&path).context("Loader.load failed")?;
        let module = self.drop_unused(module, None);

        Ok((module_id, fm, module))
    }

    fn transform_module(
        &self,
        id: ModuleId,
        fm: Arc<SourceFile>,
        mut module: Module,
    ) -> Result<TransformedModule, Error> {
        log::trace!("transform_module({})", fm.name);

        let mut imports = self.extract_import_info(&mut module);

        let (module, imports) = rayon::join(
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
                    let p = match fm.name {
                        FileName::Real(ref p) => p,
                        // stdin compilation
                        FileName::Anon => &self.working_dir,
                        _ => unreachable!("{} module in spack", fm.name),
                    };

                    // Load dependencies
                    self.load_imports(&p, imports)
                })
            },
        );

        let imports = imports?;
        let module = Arc::new(module?);

        Ok((id, fm, module, Arc::new(imports)))
    }

    fn load_imports(&self, base: &Path, info: ImportInfo) -> Result<MergedImports, Error> {
        log::trace!("load_imports({})", base.display());

        let mut merged = MergedImports::default();
        let ImportInfo {
            imports,
            requires,
            partial_requires,
            dynamic_imports,
        } = info;

        let loaded = imports
            .into_par_iter()
            .map(|import| import.src)
            .chain(partial_requires.into_par_iter().map(|require| require.src))
            .chain(requires)
            .chain(dynamic_imports)
            .map(|src| -> Result<_, Error> {
                //
                let res = self.load_transformed(base, &src.value)?;

                Ok((res, src))
            })
            .collect::<Vec<_>>();

        for res in loaded {
            // TODO: Report error and proceed instead of returning an error
            let (res, src): (TransformedModule, Str) = res?;

            merged.ids.extend(res.3.ids.clone());
            merged
                .side_effect_imports
                .extend(res.3.side_effect_imports.clone());
        }

        Ok(merged)
    }
}
