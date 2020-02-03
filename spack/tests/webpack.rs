#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(specialization)]
#![feature(test)]

extern crate test;

use spack::{loaders::swc::JsLoader, Bundler};
use std::{
    env,
    fs::{create_dir_all, File},
    io::{self, Write},
    path::Path,
    process::Command,
    sync::Arc,
};
use swc_common::{fold::FoldWith, FileName};
use tempfile::tempdir_in;
use test::{
    test_main, DynTestFn, Options, ShouldPanic::No, TestDesc, TestDescAndFn, TestName, TestType,
};
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
            name: TestName::DynTestName(name.replace("-", "_").replace("/", "::")),
            ignore,
            should_panic: No,
            allow_fail: false,
        },
        testfn: DynTestFn(box f),
    });
}

fn load(tests: &mut Vec<TestDescAndFn>) -> Result<(), io::Error> {
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    let root = {
        let mut root = project_dir.clone();
        root.push("tests");
        root.push("webpack");
        root
    };

    eprintln!("Loading tests from {}", root.display());

    let dir = root;

    for entry in WalkDir::new(&dir).into_iter() {
        let entry = entry?;
        if !entry.path().join("index.js").exists() {
            continue;
        }

        let dir_name = entry
            .path()
            .strip_prefix(&dir)
            .expect("failed to strip prefix")
            .to_str()
            .unwrap()
            .to_string();

        let input = entry.path().join("index.js");

        let name = format!("{}", dir_name);

        let ignore = !name.contains(&env::var("TEST").ok().unwrap_or("".into()));

        add_test(tests, name, ignore, {
            let project_dir = project_dir.clone();
            move || {
                let _ = pretty_env_logger::formatted_builder()
                    .is_test(true)
                    .try_init();

                eprintln!("\n\n========== Running webpack test {}\n", dir_name);

                testing::run_test2(true, |cm, handler| {
                    let compiler = Arc::new(swc::Compiler::new(cm.clone(), Arc::new(handler)));
                    let bundler = Bundler::new(
                        env::current_dir().unwrap(),
                        compiler.clone(),
                        swc::config::Options {
                            swcrc: true,
                            ..Default::default()
                        },
                        box spack::resolve::NodeResolver,
                        box JsLoader::new(compiler, Default::default()),
                    );

                    let modules = bundler.bundle(&[input]);

                    for bundled in modules {
                        let (fm, module) = bundled.expect("failed to bundle module");

                        let code = bundler
                            .swc()
                            .print(&module, fm.clone(), false, false)
                            .expect("failed to emit bundle")
                            .code;

                        let output_dir = project_dir
                            .clone()
                            .join("target")
                            .join("spack-tests")
                            .join(&dir_name);

                        create_dir_all(&output_dir).unwrap();

                        let tmp_dir =
                            tempdir_in(&output_dir).expect("failed to create a temp directory");
                        create_dir_all(&tmp_dir).unwrap();

                        let output_path = tmp_dir.path().join(format!("input.test.js"));

                        let mut output_file =
                            File::create(&output_path).expect("failed to create output file");

                        output_file
                            .write_all(code.as_bytes())
                            .expect("failed to write output to file");

                        let status = Command::new("jest")
                            .args(&["--testMatch", &format!("{}", output_path.display())])
                            .status()
                            .expect("failed to run jest");
                        if status.success() {
                            return Ok(());
                        }
                    }

                    Ok(())
                })
                .expect("failed to bundle");
            }
        });
    }

    Ok(())
}

#[test]
fn webpack() {
    let args: Vec<_> = env::args().collect();
    let mut tests = Vec::new();
    load(&mut tests).unwrap();
    test_main(&args, tests, Some(Options::new()));
}
