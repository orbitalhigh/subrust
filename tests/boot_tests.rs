
mod common;

use common::boot_diff;

#[test]
fn seed_corpus() {
    let dir = std::path::Path::new("tests/seed");
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("tests/seed")
        .filter_map(|e| {
            let p = e.ok()?.path();
            if p.extension()? == "rs" {
                Some(p.file_stem()?.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    assert!(names.len() >= 10, "corpus went missing?");

    for name in &names {
        let src = std::fs::read_to_string(dir.join(format!("{name}.rs"))).unwrap();
        let input = std::fs::read(dir.join(format!("{name}.in"))).unwrap_or_default();
        let expect_trap = name.starts_with("trap_");

        let (out, trapped) = boot_diff(name, &src, &input);

        assert_eq!(
            trapped, expect_trap,
            "{name}: trap expectation (got {trapped})"
        );
        if let Ok(golden) = std::fs::read(dir.join(format!("{name}.out"))) {
            assert_eq!(
                out, golden,
                "{name}: golden mismatch\n got: {:?}",
                String::from_utf8_lossy(&out)
            );
        }
    }
    println!("seed corpus: {} programs differentially verified", names.len());
}
