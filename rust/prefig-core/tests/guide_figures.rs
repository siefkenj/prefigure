//! Smoke test: every figure from the PreFigure Guide must build without
//! panicking.
//!
//! The figures are vendored under `tests/guide_figures/` (extracted from
//! <https://github.com/davidaustinm/prefigure-docs>, `source/code/*.xml` and
//! `assets/images/*.xml`). Each is run through the SVG and tactile build
//! pipelines inside `catch_unwind`; the test fails listing every figure that
//! panics. A graceful `Err` is acceptable -- some guide figures are meant to be
//! embedded in a PreTeXt document and get their dimensions from that wrapper,
//! so standalone they legitimately report an error rather than crash. We only
//! guard against panics (index-out-of-bounds, `unwrap` on `None`, etc.).
//!
//! Labels are rendered with fixed stub services so the test needs neither Node
//! (MathJax) nor cairo, yet still exercises the label-layout code paths.

use prefig_core::core::label_tools::{
    BrailleTranslator, FontData, LabelState, MathLabel, MathLabels, TextMeasurements,
};
use prefig_core::engine::build_from_string;
use prefig_core::xml;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};

// --- Fixed stub label services (no Node/cairo required) ---------------------

struct StubMath;
impl MathLabels for StubMath {
    fn add_macros(&mut self, _macros: &str) {}
    fn register_math_label(&mut self, _id: &str, _text: &str) {}
    fn process_math_labels(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn get_math_label(&self, _id: &str) -> Option<MathLabel> {
        // A well-formed MathJax-like placeholder: `ex`-unit width/height/style
        // and a <defs>, so the real label-insertion path runs (ex->px, glyph
        // id prefixing) instead of being skipped.
        let svg = xml::parse_str(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="1.5ex" height="1.5ex" viewBox="0 -1 1.5 1.5" style="vertical-align: -0.25ex"><defs></defs><g></g></svg>"#,
        )
        .ok()?;
        Some(MathLabel::Svg(svg))
    }
}

struct StubText;
impl TextMeasurements for StubText {
    fn measure_text(&self, text: &str, font: &FontData) -> Option<[f64; 3]> {
        let w = text.chars().count() as f64 * font.size * 0.5;
        Some([w, font.size * 0.75, font.size * 0.25])
    }
}

struct StubBraille;
impl BrailleTranslator for StubBraille {
    fn initialized(&self) -> bool {
        true
    }
    fn translate(&self, text: &str, _typeform: &[u8]) -> Option<String> {
        Some(text.chars().map(|_| '\u{283F}').collect())
    }
}

fn stub_labels() -> LabelState {
    LabelState {
        math: Box::new(StubMath),
        text: Box::new(StubText),
        braille: Box::new(StubBraille),
    }
}

// ----------------------------------------------------------------------------

fn collect_xml(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_xml(&path, out);
        } else if path.extension().is_some_and(|x| x == "xml") {
            out.push(path);
        }
    }
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>")
        .to_string()
}

#[test]
fn guide_figures_build_without_panicking() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/guide_figures");
    let mut figures = Vec::new();
    collect_xml(&root, &mut figures);
    figures.sort();
    assert!(
        figures.len() >= 130,
        "expected the vendored guide figures under {}, found only {}",
        root.display(),
        figures.len()
    );

    // Silence the default per-panic stderr dump; catch_unwind still reports the
    // panic to us. Restored afterwards.
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    let mut panicked: Vec<String> = Vec::new();
    let mut graceful_errors = 0usize;
    let mut built = 0usize;

    for path in &figures {
        let source = std::fs::read_to_string(path).expect("read figure");
        let name = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();

        for format in ["svg", "tactile"] {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                build_from_string(format, &source, "pf_cli", stub_labels())
            }));
            match result {
                Err(payload) => panicked.push(format!("{name} [{format}]: {}", panic_message(&*payload))),
                Ok(Ok(_)) => built += 1,
                Ok(Err(_)) => graceful_errors += 1,
            }
        }
    }

    panic::set_hook(prev_hook);

    eprintln!(
        "guide figures: {} files x 2 formats => {built} built, {graceful_errors} graceful errors, {} panics",
        figures.len(),
        panicked.len(),
    );

    assert!(
        panicked.is_empty(),
        "{} guide figure build(s) panicked:\n{}",
        panicked.len(),
        panicked.join("\n"),
    );
}
