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

3. ⬜ **Add a switch.** *(remaining — needs live browser verification)*
   `src/state/model.ts` constructs and drives the compiler through the Comlink
   worker. To toggle: read a query param (e.g. `?engine=wasm`), and in the
   `loadPyodide`/`compile` thunks call `worker.wasmCompiler.init()` /
   `worker.wasmCompiler.compile(mode, source)` instead of the Pyodide one when
   selected. Left for a browser session because the reactive worker flow can't
   be verified headlessly; the worker already exposes `wasmCompiler`.

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

Steps 1, 2, and 4 are done and tested; the drawing pipeline (`build_from_string`)
builds all 37 example diagrams to Python-matching SVG. Step 3 (the reactive
`model.ts` toggle) and step 5 (measure + flip default) remain and want a live
browser session. Handlers not yet ported (boolean `<shape>` ops, automatic
`<network>` layout, `<read>`, `<histogram>`/`<scatter>`, tactile labels) are
tracked in [PORTING.md](PORTING.md); a document using one currently errors on
that element, which is the moment to port it.
