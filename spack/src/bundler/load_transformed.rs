use super::Bundler;
use crate::{bundler::import_analysis::ImportInfo, Id, ModuleId};
use anyhow::{Context, Error};
use fxhash::FxHashMap;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::{FileName, SourceFile};
use swc_ecma_ast::{ImportDecl, ImportSpecifier, Module, Program, Str};

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
        Ok(self.load_transformed_inner(base, s)?.1)
    }

    fn load_transformed_inner(
        &self,
        base: &Path,
        s: &str,
    ) -> Result<(Arc<PathBuf>, TransformedModule), Error> {
        let path = self
            .resolver
            .resolve(base, s)
            .context("failed to resolve")?;

        let path = Arc::new(path);
        if let Some(cached) = self.scope.get_module_by_path(&path) {
            return Ok((path, cached.clone()));
        }

        let (id, fm, module) = self.load(&path).context("Bundler.load failed")?;

        let v = self
            .transform_module(id, fm.clone(), module)
            .context("failed to transform module")?;

        self.scope.store_module(path.clone(), v.clone());

        Ok((path, v))
    }

    fn load(&self, path: &Arc<PathBuf>) -> Result<(ModuleId, Arc<SourceFile>, Module), Error> {
        let module_id = self.scope.module_id_gen.gen(path);

        let path = Arc::new(path);

        let (fm, module) = self.loader.load(&path).context("Loader.load failed")?;
        let module = self.drop_unused(fm.name.clone(), module, None);

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
            .chain(partial_requires)
            .chain(
                requires
                    .into_par_iter()
                    .chain(dynamic_imports.into_par_iter())
                    .map(|src| ImportDecl {
                        span: src.span,
                        specifiers: vec![],
                        src,
                    }),
            )
            .map(|decl| -> Result<_, Error> {
                //
                let res = self.load_transformed_inner(base, &decl.src.value)?;

                Ok((res, decl))
            })
            .collect::<Vec<_>>();

        for res in loaded {
            // TODO: Report error and proceed instead of returning an error
            let ((path, res), decl): ((Arc<PathBuf>, TransformedModule), ImportDecl) = res?;

            if let Some(src) = self.scope.get_module_by_path(&path) {
                let src = Source {
                    module_id: src.0,
                    src: decl.src,
                };

                if decl.specifiers.is_empty() {
                    merged.side_effect_imports.push(src);
                } else {
                    for s in decl.specifiers {
                        match s {
                            ImportSpecifier::Specific(s) => {
                                merged.ids.insert(Id::from(s.local), src.clone());
                            }
                            ImportSpecifier::Default(s) => {
                                merged.ids.insert(Id::from(s.local), src.clone());
                            }
                            ImportSpecifier::Namespace(s) => {
                                merged.ids.insert(Id::from(s.local), src.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(merged)
    }
}
