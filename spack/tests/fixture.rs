#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]
#![feature(test)]

extern crate test;

use spack::{loader::Resolver, Bundler, Config};
use std::{
    env,
    fs::{read_dir, File},
    io::{self, Read},
    path::Path,
    sync::Arc,
};
use swc_common::{Fold, FoldWith};
use swc_ecma_ast::*;
use test::{
    test_main, DynTestFn, Options, ShouldPanic::No, TestDesc, TestDescAndFn, TestName, TestType,
};
use testing::StdErr;
use walkdir::WalkDir;

fn add_test<F: FnOnce() + Send + 'static>(
    tests: &mut Vec<TestDescAndFn>,
    name: String,
    ignore: bool,
    f: F,
) {
    if ignore {
        return;
    }
    tests.push(TestDescAndFn {
        desc: TestDesc {
            test_type: TestType::UnitTest,
            name: TestName::DynTestName(name),
            ignore,
            should_panic: No,
            allow_fail: false,
        },
        testfn: DynTestFn(box f),
    });
}

fn reference_tests(tests: &mut Vec<TestDescAndFn>, errors: bool) -> Result<(), io::Error> {
    let root = {
        let mut root = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        root.push("tests");
        root.push(if errors { "error" } else { "pass" });
        root
    };

    eprintln!("Loading tests from {}", root.display());

    let dir = root;

    for entry in WalkDir::new(&dir).into_iter() {
        let entry = entry?;
        if !entry.path().join("input").exists() {
            continue;
        }

        let dir_name = entry
            .path()
            .strip_prefix(&dir)
            .expect("failed to strip prefix")
            .to_str()
            .unwrap()
            .to_string();

        let entries = read_dir(entry.path())?
            .filter(|e| match e {
                Ok(e) if e.path().to_string_lossy().starts_with("entry") => true,
                _ => false,
            })
            .map(|e| -> Result<_, io::Error> { Ok(e?.path()) })
            .collect::<Result<Vec<_>, _>>()?;

        let ignore = false;

        let dir = dir.clone();
        let name = format!(
            "fixture::{}::{}",
            if errors { "error" } else { "pass" },
            dir_name
        );
        add_test(tests, name, ignore, move || {
            eprintln!("\n\n========== Running reference test {}\n", dir_name);

            let options = Arc::new(swc::config::Options::default());
            testing::run_test2(true, |cm, handler| {
                let handler = Arc::new(handler);
                let bundler = Bundler::new(
                    cm.clone(),
                    handler.clone(),
                    env::current_dir().unwrap(),
                    options.clone(),
                    box spack::loader::JsLoader::new(
                        cm.clone(),
                        handler.clone(),
                        options.clone(),
                        Resolver {},
                    ),
                );

                let modules = bundler.bundle(&entries);

                for (entry, module) in entries.into_iter().zip(modules) {
                    let (fm, module) = module.expect("failed to bundle module");

                    let code = bundler
                        .jsc()
                        .print(&Program::Module(module), fm.clone(), false, false)?
                        .code;
                }

                Ok(())
            })
            .expect("failed to process a module");
        });
    }

    Ok(())
}

#[test]
fn pass() {
    let args: Vec<_> = env::args().collect();
    let mut tests = Vec::new();
    reference_tests(&mut tests, false).unwrap();
    test_main(&args, tests, Some(Options::new()));
}

#[test]
fn errors() {
    let args: Vec<_> = env::args().collect();
    let mut tests = Vec::new();
    reference_tests(&mut tests, false).unwrap();
    test_main(&args, tests, Some(Options::new()));
}
