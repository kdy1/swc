#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]
#![feature(test)]

extern crate test;

use spack::{loaders::swc::JsLoader, Bundler};
use std::{
    env,
    fs::{create_dir_all, read_dir},
    io::{self},
    path::Path,
    sync::Arc,
};
use swc_common::{fold::FoldWith, FileName};
use swc_ecma_ast::Program;
use test::{
    test_main, DynTestFn, Options, ShouldPanic::No, TestDesc, TestDescAndFn, TestName, TestType,
};
use testing::{DropSpan, NormalizedOutput};
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

        let _ = create_dir_all(entry.path().join("output"));

        let entries = read_dir(entry.path().join("input"))?
            .filter(|e| match e {
                Ok(e) => {
                    if e.path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("entry")
                    {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            })
            .map(|e| -> Result<_, io::Error> { Ok(e?.path()) })
            .collect::<Result<Vec<_>, _>>()?;

        let name = format!(
            "fixture::{}::{}",
            if errors { "error" } else { "pass" },
            dir_name
        );

        let ignore = !name.contains(&env::var("TEST").ok().unwrap_or("".into()));

        add_test(tests, name, ignore, move || {
            let _ = pretty_env_logger::formatted_builder()
                .is_test(true)
                .try_init();

            eprintln!("\n\n========== Running reference test {}\n", dir_name);

            testing::run_test2(true, |cm, handler| {
                let handler = Arc::new(handler);
                let bundler = Bundler::new(
                    cm.clone(),
                    handler.clone(),
                    env::current_dir().unwrap(),
                    Default::default(),
                    box spack::resolve::NodeResolver,
                    box JsLoader::new(cm.clone(), handler.clone(), Default::default()),
                );

                assert_ne!(entries.len(), 0);

                let modules = bundler.bundle(&entries);

                for bundled in modules {
                    let (fm, module) = bundled.expect("failed to bundle module");

                    let code = bundler
                        .swc()
                        .print(&module, fm.clone(), false, false)
                        .expect("failed to emit bundle")
                        .code;

                    let name = match fm.name {
                        FileName::Real(ref p) => p.clone(),
                        _ => unreachable!(),
                    };

                    let output_path = entry.path().join("output").join(name.file_name().unwrap());

                    let program = bundler
                        .swc()
                        .parse_js(fm, Default::default(), Default::default(), true, true)
                        .expect("failed to parse output file as program")
                        .fold_with(&mut DropSpan);

                    if program == Program::Module(module.clone()).fold_with(&mut DropSpan) {
                        continue;
                    }

                    let s = NormalizedOutput::from(code);

                    s.compare_to_file(&output_path).expect("failed to print");
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
    reference_tests(&mut tests, true).unwrap();
    test_main(&args, tests, Some(Options::new()));
}
