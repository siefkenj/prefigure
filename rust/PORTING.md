# Porting status: Python → Rust

The Rust port shadows the reference Python implementation (see
`RUST_PORT_OUTLINE.md` at the repo root). This file is the sync contract:
one row per Python module, the Rust module that mirrors it, and the Python
commit it was last verified against. **When you change a `prefig/` module,
update the matching Rust module and this table.**

Last full sync: Python @ `d0ac23a` (version 0.7.0).

| Python module | Rust module | Status | Notes |
|---|---|---|---|
| `core/user_namespace.py` | `evaluator/` (mod, parse, interp, builtins) | ✅ ported | Instance-based `ExpressionContext` instead of module globals (outline §4.1). Parser is a dedicated PEG grammar (outline §16), not Python's `ast`. Missing: `derivative()` registration, `enter_function`, ODE break/delta machinery (`find_breaks`, `measure_de_jump`) — land with `diffeqs.rs`. |
| `core/math_utilities.py` | `evaluator/builtins.rs` | 🟡 partial | Everything except the diagram-dependent functions (`intersect`, `solve`, `proj_2d`, `line_intersection`, `filter`, `grad`, `delta`) — they need the `EvalEnv` handle (outline §4.1) or `Diagram`. Move to `core/math_utilities.rs` when `Diagram` exists. |
| `core/calculus.py` | `core/calculus.rs` | ✅ ported | Richardson-extrapolated derivative. |
| `core/parse.py` | — | ⬜ next (M1) | |
| `core/diagram.py` | — | ⬜ next (M1) | |
| `core/CTM.py` | — | ⬜ next (M1) | |
| `core/utilities.py` | — | ⬜ next (M1) | |
| `core/tags.py` | — | ⬜ M1 | |
| everything else in `core/` | — | ⬜ M1–M4 | See outline §13 milestones. |
| `engine.py` | — | ⬜ M1 | `build_from_string` unblocks the `rust_output_matches_python` test. |
| `cli.py` | — | ⬜ M6 | |

## Test suites (TDD ground truth)

| Suite | Source of truth | Regenerate with |
|---|---|---|
| `tests/parser.rs` | outline §16.5 corner-case table | hand-maintained |
| `tests/expression_tests.rs` + `tests/expression_tests.json` | Python `user_namespace` (147 steps, 12 sessions) | `poetry run python rust/tools/generate_expression_tests.py` |
| `tests/expected_svgs.rs` + `tests/expected_svgs/{repo,docs}/*.svg` | Python-built SVGs: 8 repo examples + 29 diagrams from [prefigure-docs](https://github.com/davidaustinm/prefigure-docs) (GPL, same author) | `rust/tools/generate_expected_svgs.sh <prefigure-docs checkout>` |

Regenerate these files whenever the corresponding Python modules change;
they are checked in so CI needs no Python.

## Behavioral findings pinned by the test data (don't "fix" these)

- `m[1, 0]` is numpy **fancy row-indexing** (rows 1 and 0), not element
  `[1][0]` — Python's TransformList wraps the index tuple in `np.array`.
- `[(1,2),(3,4)] + (10,20)` broadcasts over rows (trailing-dimension
  alignment), not element-by-element zip.
- `round()` is banker's rounding **on the true decimal value**:
  `round(2.675, 2) == 2.67`. Implemented via format-then-parse; a
  multiply-by-10^n scale would give 2.68.
- `valid_eval("  #abc")` returns the string **with leading whitespace**.
- `rgb(...)` components are evaluated then truncated toward zero (`int()`).
- Python `%` and `//` follow the divisor's sign: `-7 // 2 == -4`, `-7 % 3 == 2`.
