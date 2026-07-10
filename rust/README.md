# PreFigure in Rust

A Rust version of [PreFigure](https://prefigure.org), ported from the Python
implementation in [`../prefig/`](../prefig/). The Python version remains the
reference; this port follows it module by module so the two stay in sync
(see [PORTING.md](PORTING.md) for what is done and what isn't).

The main goal is a small, fast WebAssembly build so that websites (the
PreFigure playground, [DoenetML](https://github.com/Doenet/DoenetML)) can build
diagrams in the browser without downloading the much larger Python stack.

## What's here

| Directory | Contents |
|---|---|
| `prefig-core/` | The library: expression evaluator now, drawing pipeline in progress |
| `prefig-wasm/` | WebAssembly bindings for browsers and Node |
| `prefig-cli/` | The `prefig` command-line program |
| `tools/` | Scripts that generate test data from the Python version |

## Requirements

- Rust (edition 2021 or later) — <https://rustup.rs>
- For the WebAssembly build: [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) and Node
- For regenerating test data: the Python version installed at the repo root (`poetry install`)

## Build and test

```sh
cd rust
cargo build            # build everything
cargo test             # run all tests
```

Try the command line:

```sh
cargo run -p prefig-cli -- eval "(1,2) + (3,4)"
```

## WebAssembly

```sh
cd rust/prefig-wasm
npm install
npm test               # compiles to WebAssembly, then runs the Node tests
```

The compiled package lands in `prefig-wasm/pkg/`.

## Test data

The tests compare this port against output from the Python version:

- `prefig-core/tests/expression_tests.json` — expressions with the results
  Python produces. Regenerate with
  `poetry run python rust/tools/generate_expression_tests.py`.
- `prefig-core/tests/expected_svgs/` — SVGs that Python builds from the
  diagrams in `prefig-core/tests/example_diagrams/`. Regenerate with
  `rust/tools/generate_expected_svgs.sh <path to a prefigure-docs checkout>`.

Both are checked in, so running the tests does not require Python. Regenerate
them whenever the Python version changes behavior.

## Design documents

- [`../RUST_PORT_OUTLINE.md`](../RUST_PORT_OUTLINE.md) — the full plan for the port
- [PORTING.md](PORTING.md) — per-module status and rules for staying in sync
- [PLAYGROUND_PLAN.md](PLAYGROUND_PLAN.md) — plan for using the WebAssembly build on the website
