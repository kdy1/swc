use super::Bundler;
use crate::ModuleId;
use anyhow::{Context, Error};
use std::{path::Path, sync::Arc};
use swc_common::{FileName, SourceFile};
use swc_ecma_ast::{Module, Program};

/// Module after applying transformations.
pub(crate) type TransformedModule = (ModuleId, Arc<SourceFile>, Arc<Module>);

impl Bundler {
    /// Phase 1 (discovery) and Phase 2 (linking)
    ///
    /// We apply transforms at this phase to make cache efficient.
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

        Ok((module_id, fm, module))
    }

    fn transform_module(
        &self,
        id: ModuleId,
        fm: Arc<SourceFile>,
        mut module: Module,
    ) -> Result<TransformedModule, Error> {
        log::trace!("transform_module({})", fm.name);

        let info = self.extract_info(&mut module);
        self.store_pure_constants(id, info.exports.pure_constants);
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
                    let p = match fm.name {
                        FileName::Real(ref p) => p,
                        // stdin compilation
                        FileName::Anon => &self.working_dir,
                        _ => unreachable!("{} module in spack", fm.name),
                    };

                    // Load dependencies
                    self.load_imports(&p, &imports)
                })
            },
        );

        deps?;
        let module = Arc::new(module?);

        Ok((id, fm, module))
    }
}
