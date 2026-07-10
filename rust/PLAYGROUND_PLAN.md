# Plan: using the WebAssembly build in the Playground website

The playground (`website/packages/playground`) currently builds diagrams by
running the *Python* version of PreFigure inside the browser with Pyodide.
That means every visitor downloads Pyodide plus Python packages (lxml, numpy,
scipy, shapely, networkx, and the prefig wheel) — tens of megabytes — before
the first diagram appears. The WebAssembly build of the Rust port replaces all
of that with one file, currently ~86 KB compressed (it will grow as the
drawing pipeline lands, but stays in the hundreds-of-KB range).

## What stays the same

The browser-side support code needs no changes:

- **`src/worker/compat-api.ts`** (`PrefigBrowserApi`) already provides the four
  services PreFigure needs from the browser: rendering math with MathJax
  (`processMath`), converting math to braille (`processBraille`), translating
  text to braille (`translate_text`), and measuring text
  (`measure_text`). The Rust WebAssembly module calls the same object with the
  same method names the Python version uses today.
- The worker architecture, the editor, and the UI around compilation.
- The result shape: `compile()` keeps returning `{ svg, annotations }`.

## Steps

1. **Package the WebAssembly build for the website.**
   Build with `wasm-pack build --target web` (the Node tests use
   `--target nodejs`; the website needs the browser flavor). Add the output as
   a workspace package, e.g. `website/packages/prefig-wasm`, or publish to npm
   as `@prefigure/prefig-wasm` and depend on it normally.

2. **Add a second compiler class next to the Pyodide one.**
   In `src/worker/`, add `compiler-wasm.ts` with the same public interface as
   `PreFigureCompiler` in `compiler.ts`:
   - `init()`: load and instantiate the WebAssembly module (one `await import`
     plus one call), then hand it the `prefigBrowserApi` object.
   - `compile(mode, source)`: call the module's `build_from_string(mode,
     source)` and return `{ svg, annotations }`.
   Keeping both classes lets the playground switch per session while the port
   matures; the Pyodide path is deleted at the end.

3. **Add a switch.**
   A query parameter or settings toggle (default: Pyodide at first) selects
   which compiler the worker constructs. This enables side-by-side comparison
   on real documents and an easy fallback if a diagram exposes a gap in the
   port.

4. **Wire up the tests.**
   The playground already runs vitest. Add a test that compiles each source in
   `rust/prefig-core/tests/example_diagrams/` with the WebAssembly compiler and
   compares the SVG against `rust/prefig-core/tests/expected_svgs/` (same
   comparison the Rust tests use: identical structure, numbers within
   tolerance).

5. **Measure and switch the default.**
   Record load time and download size for both paths. When the example
   diagrams all match and no open playground bug depends on Pyodide, flip the
   default to WebAssembly. Keep the Pyodide path for one release as an escape
   hatch, then remove it and the Python package downloads from `compiler.ts`.

6. **Later, for DoenetML:** publish a second, slimmer package variant built
   with `--no-default-features --features xast,diffeqs,network,shapes` that
   accepts documents as XAST (the JSON form of XML that Doenet already has)
   instead of XML text, and grow `PrefigBrowserApi` with `processMathXast` so
   the module needs no XML parser at all. Details in
   [`../RUST_PORT_OUTLINE.md`](../RUST_PORT_OUTLINE.md) §10.

## Blockers

Steps 2–5 need `build_from_string` to actually build diagrams, which is
milestones M1–M3 of the port (drawing pipeline, then labels): see
[PORTING.md](PORTING.md). Step 1 can happen now — the evaluator-only module is
already buildable and tested — and is a good way to settle the packaging and
worker-loading questions early.
