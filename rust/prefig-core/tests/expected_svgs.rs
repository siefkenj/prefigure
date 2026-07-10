//! SVG-output parity tests (RUST_PORT_OUTLINE.md §12.3).
//!
//! `tests/expected_svgs/{repo,docs}/*.svg` are built by the reference Python
//! implementation (regenerate with rust/tools/generate_expected_svgs.sh); the matching
//! sources live in `tests/example_diagrams/`. Once the Rust pipeline exists,
//! `rust_output_matches_python` builds each source and compares against the Python-produced SVG with
//! numeric tolerance. Until then it is #[ignore]d, but the comparator itself
//! is exercised by the self-check tests below.

mod svg_compare;

use std::fs;
use std::path::{Path, PathBuf};

fn expected_svgs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/expected_svgs")
}

fn expected_svg_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for category in ["repo", "docs"] {
        let dir = expected_svgs_dir().join(category);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "svg") {
                    files.push(entry.path());
                }
            }
        }
    }
    files.sort();
    files
}

/// The comparator must accept every expected SVG compared against itself.
#[test]
fn comparator_selfcheck_accepts_identical() {
    let files = expected_svg_files();
    assert!(
        files.len() >= 37,
        "expected the generated SVGs to be present, found {}",
        files.len()
    );
    for path in files {
        let svg = fs::read_to_string(&path).unwrap();
        let diffs = svg_compare::compare(&svg, &svg, 1e-4);
        assert!(
            diffs.is_empty(),
            "{} does not match itself: {:?}",
            path.display(),
            diffs
        );
    }
}

/// ...and must reject clearly different documents.
#[test]
fn comparator_selfcheck_rejects_different() {
    let files = expected_svg_files();
    let a = fs::read_to_string(&files[0]).unwrap();
    let b = fs::read_to_string(files.last().unwrap()).unwrap();
    assert!(
        !svg_compare::compare(&a, &b, 1e-4).is_empty(),
        "comparator failed to distinguish {} from {}",
        files[0].display(),
        files.last().unwrap().display()
    );
}

/// ...and must tolerate sub-tolerance numeric jitter but flag real drift.
#[test]
fn comparator_selfcheck_numeric_tolerance() {
    let svg = r#"<svg width="310"><path d="M 5.0 305.0 L 47.9 5.0"/></svg>"#;
    let jittered = r#"<svg width="310"><path d="M 5.00003 305.0 L 47.9 5.0"/></svg>"#;
    let drifted = r#"<svg width="310"><path d="M 5.2 305.0 L 47.9 5.0"/></svg>"#;
    assert!(svg_compare::compare(svg, jittered, 1e-4).is_empty());
    assert!(!svg_compare::compare(svg, drifted, 1e-4).is_empty());
}

/// The real parity test: build every example source with the Rust pipeline and
/// compare against the SVG that Python produced.
#[test]
#[ignore = "enable once engine::build_from_string exists (outline milestone M1+)"]
fn rust_output_matches_python() {
    unimplemented!(
        "for each tests/example_diagrams/{{repo,docs}}/NAME.xml: \
         run the Rust pipeline and svg_compare against tests/expected_svgs/.../NAME.svg"
    );
}
