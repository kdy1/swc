use super::Plan;
use crate::bundler::tests::{suite, Tester};
use std::collections::{HashMap, HashSet};
use swc_common::FileName;

#[track_caller]
fn assert_normal(t: &mut Tester, p: &Plan, entry: &str, deps: &[&str]) {
    assert_normal_transitive(t, p, entry, deps, &[]);
}

#[track_caller]
fn assert_normal_transitive(
    t: &mut Tester,
    p: &Plan,
    entry: &str,
    deps: &[&str],
    transitive_deps: &[&str],
) {
    if deps.is_empty() {
        if let Some(v) = p.normal.get(&t.id(&format!("{}.js", entry))) {
            assert_eq!(v.chunks, vec![], "Should be empty");
        }

        return;
    }

    assert_eq!(
        p.normal[&t.id(&format!("{}.js", entry))]
            .chunks
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
        deps.into_iter()
            .map(|s| format!("{}.js", s))
            .map(|s| t.id(&s))
            .collect::<HashSet<_>>(),
        "Should merge {:?}",
        deps
    );

    assert_eq!(
        p.normal[&t.id(&format!("{}.js", entry))]
            .transitive_chunks
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
        transitive_deps
            .into_iter()
            .map(|s| format!("{}.js", s))
            .map(|s| t.id(&s))
            .collect::<HashSet<_>>(),
        "Should merge {:?} transitivly",
        transitive_deps
    )
}

fn assert_circular(t: &mut Tester, p: &Plan, entry: &str, members: &[&str]) {
    assert_eq!(
        p.circular[&t.id(&format!("{}.js", entry))]
            .chunks
            .iter()
            .cloned()
            .collect::<HashSet<_>>(),
        members
            .into_iter()
            .map(|s| format!("{}.js", s))
            .map(|s| t.id(&s))
            .collect::<HashSet<_>>(),
    );
}

#[test]
fn concurrency_001() {
    suite()
        .file(
            "main.js",
            "
            export { b } from './b';
            export { a } from './a';
            ",
        )
        .file(
            "a.js",
            "
            import { b } from './b';
            export const a = '1';
            console.log(b);
            ",
        )
        .file(
            "b.js",
            "
            export const b = '1'
            ",
        )
        .run(|t| {
            let module = t
                .bundler
                .load_transformed(&FileName::Real("main.js".into()))?
                .unwrap();
            let mut entries = HashMap::default();
            entries.insert("main.js".to_string(), module);

            let p = t.bundler.determine_entries(entries)?;

            assert_eq!(p.circular.len(), 0);

            assert_normal(t, &p, "main", &["a"]);
            assert_normal(t, &p, "a", &["b"]);
            assert_normal(t, &p, "b", &[]);

            Ok(())
        });
}

#[test]
fn concurrency_002() {
    suite()
        .file(
            "main.js",
            "
            export { a } from './a';
            export { b } from './b';
            ",
        )
        .file(
            "a.js",
            "
            import { b } from './b';
            export const a = '1';
            console.log(b);
            ",
        )
        .file(
            "b.js",
            "
            export const b = '1'
            ",
        )
        .run(|t| {
            let module = t
                .bundler
                .load_transformed(&FileName::Real("main.js".into()))?
                .unwrap();
            let mut entries = HashMap::default();
            entries.insert("main.js".to_string(), module);

            let p = t.bundler.determine_entries(entries)?;

            assert_eq!(p.circular.len(), 0);

            assert_normal(t, &p, "main", &["a"]);
            assert_normal(t, &p, "a", &["b"]);
            assert_normal(t, &p, "b", &[]);

            Ok(())
        });
}

#[test]
fn concurrency_003() {
    suite()
        .file(
            "main.js",
            "
            import { A } from './a';
            import { B } from './b';

            console.log(A, B);
            ",
        )
        .file(
            "a.js",
            "
            import { B } from './b';

            export class A extends B { }
            ",
        )
        .file(
            "b.js",
            "
            export class B { }
            ",
        )
        .run(|t| {
            let module = t
                .bundler
                .load_transformed(&FileName::Real("main.js".into()))?
                .unwrap();
            let mut entries = HashMap::default();
            entries.insert("main.js".to_string(), module);

            let p = t.bundler.determine_entries(entries)?;

            assert_eq!(p.circular.len(), 0);
            assert_eq!(p.normal.len(), 2);
            assert_normal(t, &p, "main", &["a"]);
            assert_normal(t, &p, "a", &["b"]);
            assert_normal(t, &p, "b", &[]);

            Ok(())
        });
}

#[test]
fn circular_001() {
    suite()
        .file(
            "main.js",
            "
                import { A } from './a';
                import { B } from './b';

                console.log(A, B);
            ",
        )
        .file(
            "a.js",
            "
            import { B } from './b'

            export class A {
                method() {
                    return new B();
                }
            }
        ",
        )
        .file(
            "b.js",
            "
            import { A } from './a';

            export class B extends A {
            }
            ",
        )
        .run(|t| {
            let module = t
                .bundler
                .load_transformed(&FileName::Real("main.js".into()))?
                .unwrap();
            let mut entries = HashMap::default();
            entries.insert("main.js".to_string(), module.clone());

            let p = t.bundler.determine_entries(entries)?;

            assert_circular(t, &p, "a", &["b"]);
            assert_normal(t, &p, "main", &["a"]);
            assert_normal(t, &p, "a", &[]);
            assert_normal(t, &p, "b", &[]);

            Ok(())
        });
}

#[test]
fn transitive_001() {
    suite()
        .file(
            "main.js",
            "
                import './a';
                import './b';
                ",
        )
        .file("a.js", "import './common';")
        .file("b.js", "import './common';")
        .file("common.js", r#"console.log('foo')"#)
        .run(|t| {
            let module = t
                .bundler
                .load_transformed(&FileName::Real("main.js".into()))?
                .unwrap();
            let mut entries = HashMap::default();
            entries.insert("main.js".to_string(), module);

            let p = t.bundler.determine_entries(entries)?;

            dbg!(&p);

            assert_eq!(p.circular.len(), 0);
            assert_normal_transitive(t, &p, "main", &["a", "b"], &["common"]);
            assert_normal_transitive(t, &p, "a", &[], &[]);
            assert_normal_transitive(t, &p, "b", &[], &[]);

            Ok(())
        });
}

#[test]
fn transitive_002() {
    suite()
        .file(
            "main.js",
            "
                import './a';
                import './b';
                import './c';
                import './d';
                ",
        )
        .file(
            "a.js",
            "
                import './common1';
                import './common2';
                ",
        )
        .file(
            "b.js",
            "
                import './common2';
                import './common1';
                ",
        )
        .file(
            "c.js",
            "
                import './common3';
                import './common2';
                ",
        )
        .file(
            "d.js",
            "
                import './common1';
                import './common4';
                ",
        )
        .file("common1.js", r#"console.log(1)"#)
        .file("common2.js", r#"console.log(2)"#)
        .file("common3.js", r#"console.log(3)"#)
        .file("common4.js", r#"console.log(4)"#)
        .run(|t| {
            let module = t
                .bundler
                .load_transformed(&FileName::Real("main.js".into()))?
                .unwrap();
            let mut entries = HashMap::default();
            entries.insert("main.js".to_string(), module);

            let p = t.bundler.determine_entries(entries)?;

            dbg!(&p);

            assert_eq!(p.circular.len(), 0);
            assert_normal_transitive(
                t,
                &p,
                "main",
                &["a", "b", "c", "d"],
                &["common1", "common2"],
            );
            assert_normal_transitive(t, &p, "a", &[], &[]);
            assert_normal_transitive(t, &p, "b", &[], &[]);
            assert_normal_transitive(t, &p, "b", &[], &["common3"]);
            assert_normal_transitive(t, &p, "b", &[], &["common4"]);

            Ok(())
        });
}
