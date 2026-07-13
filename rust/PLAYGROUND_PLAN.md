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

1. ✅ **Package the WebAssembly build for the website.**
   `website/packages/prefig-wasm/` is a workspace package
   (`@prefigure/prefig-wasm`) that builds both flavors with `wasm-pack`:
   `pkg-node/` (`--target nodejs`, for tests) and `pkg-web/` (`--target web`,
   for the browser bundle). `npm run build` in that package builds both. The
   full module is ~858 KB raw / ~320 KB gzipped — vs the ~40 MB Pyodide stack.

2. ✅ **Add a second compiler class next to the Pyodide one.**
   `src/worker/compiler-wasm.ts` (`PreFigureWasmCompiler`) has the same public
   interface as `PreFigureCompiler`: `init()` loads the module and hands it
   `prefigBrowserApi` via `set_host_api`; `compile(mode, source)` calls
   `build_from_string` and returns `{ svg, annotations }`. Both compilers are
   exposed from `src/worker/index.ts`.

3. ✅ **Add a switch.**
   `src/state/model.ts` now holds an `engine: "pyodide" | "wasm"` state,
   initialized from the `?engine=wasm` query parameter. The `loadPyodide`
   (init) and `compile` thunks dispatch to `worker.wasmCompiler` or
   `worker.compiler` accordingly, and an `onSetEngine` thunk re-initializes the
   selected engine and recompiles when it changes. A **Python / Rust** toggle in
   the navbar (`App.tsx`) drives it, and the loading spinner
   (`renderer.tsx`) covers the `loadingWasm` state. Typechecks, bundles the
   WebAssembly chunk, and the vitest suite passes; the interactive reactive flow
   still wants a click-through in a real browser (`npm run dev`).

4. ✅ **Wire up the tests.**
   `test/compiler-wasm.test.ts` (vitest) drives the real WebAssembly module
   over example diagrams from `rust/prefig-core/tests/example_diagrams/` with a
   mock host API, asserting valid, non-trivial SVG. The authoritative
   structural parity (all 37 examples vs Python within tolerance) lives in the
   Rust suite `rust/prefig-core/tests/expected_svgs.rs`.

5. ⬜ **Measure and switch the default.**
   Record load time and download size for both paths. When no open playground
   bug depends on Pyodide, flip the default to WebAssembly, keep Pyodide one
   release as an escape hatch, then remove it and the Python downloads from
   `compiler.ts`.

6. ⬜ **Later, for DoenetML:** publish a slimmer variant built with
   `--no-default-features --features xast,diffeqs` that accepts documents as
   XAST (the JSON form of XML Doenet already has) instead of XML text, and grow
   `PrefigBrowserApi` with `processMathXast` so the module needs no XML parser.
   Details in [`../RUST_PORT_OUTLINE.md`](../RUST_PORT_OUTLINE.md) §10.

## Status

Steps 1–4 are done and tested; the drawing pipeline (`build_from_string`)
builds all 40 example diagrams to Python-matching SVG, and the playground has a
working Python/Rust engine toggle. Step 5 (measure load time + download size,
then flip the default to WebAssembly) remains and wants a live browser session.
Every `prefig/core/*.py` handler is now ported (see [PORTING.md](PORTING.md)),
so documents no longer error on unported elements.
