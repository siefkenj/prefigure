# @prefigure/prefig-wasm

The Rust port of PreFigure (`rust/prefig-core`) compiled to WebAssembly.

Build with `npm run build` — produces:
- `pkg-node/` (Node target) used by tests
- `pkg-web/` (browser target) used by the playground

Both export `version()`, `set_host_api(api)`, `build_from_string(mode, source)`,
and an `Evaluator` class. `set_host_api` takes the playground's
`PrefigBrowserApi` (math rendering, braille, text measurement).
