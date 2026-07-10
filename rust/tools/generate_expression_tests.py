#!/usr/bin/env python3
"""Generate expression tests with expected results from the reference Python implementation.

Each "session" is a fresh user_namespace; steps run in order so definitions can be
tested. Results are serialized structurally (not via repr) so the Rust side can
compare with numeric tolerance.

Run from the repo root:
    poetry run python rust/tools/generate_expression_tests.py

Output: rust/prefig-core/tests/expression_tests.json (regenerate whenever
prefig/core/user_namespace.py or math_utilities.py changes).
"""

import importlib
import json
import sys
from pathlib import Path

import numpy as np

REPO_ROOT = Path(__file__).resolve().parents[2]
OUT = REPO_ROOT / "rust" / "prefig-core" / "tests" / "expression_tests.json"

# Each entry: (session_name, [steps]).
# step = ("eval", expr) | ("eval", expr, tol) | ("define", expr) | ("error", expr)
CORPUS = [
    ("integers-and-floats", [
        ("eval", "2"),
        ("eval", "0.5"),
        ("eval", ".5"),
        ("eval", "5."),
        ("eval", "1.e5"),
        ("eval", "1e-6"),
        ("eval", "2.5E-3"),
        ("eval", "1_000"),
    ]),
    ("arithmetic", [
        ("eval", "3 + 4 * 2"),
        ("eval", "(3 + 4) * 2"),
        ("eval", "7 // 2"),
        ("eval", "-7 // 2"),
        ("eval", "7 % 3"),
        ("eval", "-7 % 3"),
        ("eval", "7 % -3"),
        ("eval", "2**-3"),
        ("eval", "-2**2"),
        ("eval", "2**3**2"),
        ("eval", "--3"),
        ("eval", "+-3"),
        ("eval", "2^5"),            # ^ -> ** substitution
        ("eval", "10 / 4"),
    ]),
    ("constants-and-math", [
        ("eval", "pi", 1e-15),
        ("eval", "e", 1e-15),
        ("eval", "sin(pi/4)", 1e-15),
        ("eval", "cos(0)"),
        ("eval", "tan(pi/4)", 1e-15),
        ("eval", "atan2(1, 1)", 1e-15),
        ("eval", "exp(1)", 1e-15),
        ("eval", "log(e)", 1e-15),
        ("eval", "ln(e)", 1e-15),
        ("eval", "log10(100)", 1e-12),
        ("eval", "log2(8)", 1e-12),
        ("eval", "sqrt(16)"),
        ("eval", "abs(-3.5)"),
        ("eval", "floor(2.7)"),
        ("eval", "ceil(2.1)"),
        ("eval", "max(1, 5, 3)"),
        ("eval", "min(1, 5, 3)"),
        ("eval", "round(2.5)"),     # banker's rounding -> 2
        ("eval", "round(1.5)"),     # -> 2
        ("eval", "round(0.5)"),     # -> 0
        ("eval", "round(2.675, 2)"),
        ("eval", "sec(0)"),
        ("eval", "csc(pi/2)", 1e-15),
        ("eval", "cot(pi/4)", 1e-12),
        ("eval", "degrees(pi)", 1e-12),
        ("eval", "radians(180)", 1e-12),
        ("eval", "choose(5, 2)"),
        ("eval", "factorial(5)"),
        ("eval", "sinh(1)", 1e-15),
        ("eval", "cosh(1)", 1e-15),
        ("eval", "tanh(1)", 1e-15),
        ("eval", "asin(1)", 1e-15),
        ("eval", "acos(0)", 1e-15),
        ("eval", "atan(1)", 1e-15),
        ("eval", "inf"),
        ("eval", "-inf"),
    ]),
    ("vectors", [
        ("eval", "(1, 2)"),
        ("eval", "(1,)"),
        ("eval", "()"),
        ("eval", "[1, 2, 3]"),
        ("eval", "[]"),
        ("eval", "(1, 2) + (3, 4)"),
        ("eval", "(1, 2) - (3, 4)"),
        ("eval", "2 * (1, 2)"),
        ("eval", "(1, 2) * 2"),
        ("eval", "(1, 2) / 2"),
        ("eval", "(2, 4) * (3, 5)"),
        ("eval", "-(1, 2)"),
        ("eval", "(1, 2, 3) ** 2"),
        ("eval", "1, 2"),           # bare top-level tuple
        ("eval", "(1, 2,)"),
        ("eval", "[1, 2,]"),
        ("eval", "[(1, 2), (3, 4)]"),
        ("eval", "[(1, 2), (3, 4)] + (10, 20)"),   # broadcasting
        ("eval", "((1, 2), (3, 4, 5))"),           # ragged: inhomogeneous fallback
    ]),
    ("vector-functions", [
        ("eval", "dot((1, 2), (3, 4))"),
        ("eval", "length((3, 4))"),
        ("eval", "distance((0, 0), (3, 4))"),
        ("eval", "normalize((3, 4))", 1e-15),
        ("eval", "midpoint((0, 0), (2, 4))"),
        ("eval", "angle((1, 1))", 1e-12),
        ("eval", "angle((1, 1), 'rad')", 1e-15),
        ("eval", "rotate((1, 0), pi/2)", 1e-15),
        ("eval", "roll([(1, 2), (3, 4), (5, 6)])"),
        ("eval", "append((1, 2), 3)"),
        ("eval", "zip_lists((1, 2), (3, 4))"),
        ("eval", "evaluate_bezier(((0,0), (1,1), (2,0)), 0.5)", 1e-15),
        ("eval", "evaluate_bezier(((0,0), (1,1), (2,1), (3,0)), 0.25)", 1e-15),
        ("eval", "chi_oo(0, 1, 0.5)"),
        ("eval", "chi_oo(0, 1, 0)"),
        ("eval", "chi_cc(0, 1, 0)"),
        ("eval", "chi_co(0, 1, 1)"),
        ("eval", "chi_oc(0, 1, 1)"),
    ]),
    ("names-and-definitions", [
        ("define", "a = 5"),
        ("eval", "a + 3"),
        ("define", "p = (1, 2)"),
        ("eval", "p + (1, 1)"),
        ("eval", "(*p, 1)"),
        ("define", "f(x) = x**2 + 3*x + 1"),
        ("eval", "f(5)"),
        ("eval", "3 * f(4) + 1"),
        ("eval", "f(f(1))"),
        ("define", "g(t, y) = t - y"),
        ("eval", "g(0.5, 1)"),
        ("define", "h(x) = f(x) + a"),
        ("eval", "h(2)"),
        ("eval", "f"),              # bare function name -> function value
        ("define", "b = a * 2"),
        ("eval", "b"),
        ("eval", "deriv(f, 1)", 1e-6),
        ("define", "c = f(2)"),
        ("eval", "c"),
    ]),
    ("indexing", [
        ("define", "v = (10, 20, 30)"),
        ("eval", "v[0]"),
        ("eval", "v[-1]"),
        ("define", "k = 1"),
        ("eval", "v[k]"),
        ("eval", "v[k + 1]"),
        ("define", "m = ((1, 2), (3, 4))"),
        ("eval", "m[1]"),
        ("eval", "m[1][0]"),
        ("eval", "m[1, 0]"),        # tuple subscript -> 2-D numpy index
        ("eval", "v[0] + v[1]"),
    ]),
    ("dicts", [
        ("eval", "{}"),
        ("eval", "{'a': 1, 'b': 2}"),
        ("eval", "{'a': 'x', 3: 'y', }"),
        ("define", "d = {'a': 10, 'b': 20}"),
        ("eval", "d['a']"),
    ]),
    ("strings-and-colors", [
        ("eval", "'deg'"),
        ("eval", '"hello"'),
        ("eval", "#ff0000"),        # color literal passthrough
        ("eval", "  #abc"),
        ("eval", "rgb(255, 0, 0)"),
        ("eval", "rgb(25 * 10, 0, 2**3)"),
    ]),
    ("bools", [
        ("eval", "True"),
        ("eval", "False"),
    ]),
    ("euler", [
        ("define", "f(t, y) = y"),
        ("eval", "eulers_method(f, 0, 1, 1, 4)", 1e-12),
    ]),
    ("errors", [
        ("error", "x < 3"),
        ("error", "unknown_name"),
        ("error", "unknown_fn(3)"),
        ("error", "[i for i in (1, 2)]"),
        ("error", "lambda x: x"),
        ("error", "3 +"),
        ("error", "'unclosed"),
    ]),
]


def to_jsonable(v, seen_depth=0):
    if seen_depth > 12:
        raise ValueError("value too deep")
    if isinstance(v, (bool, np.bool_)):
        return {"t": "bool", "v": bool(v)}
    if isinstance(v, (int, np.integer)):
        return {"t": "num", "v": float(v)}
    if isinstance(v, (float, np.floating)):
        if np.isinf(v):
            return {"t": "num", "v": "inf" if v > 0 else "-inf"}
        return {"t": "num", "v": float(v)}
    if isinstance(v, str):
        return {"t": "str", "v": v}
    if isinstance(v, dict):
        return {"t": "dict", "v": {str(k): to_jsonable(x, seen_depth + 1) for k, x in v.items()}}
    if isinstance(v, np.ndarray):
        return {"t": "array", "v": [to_jsonable(x, seen_depth + 1) for x in v]}
    if isinstance(v, (list, tuple)):
        return {"t": "array", "v": [to_jsonable(x, seen_depth + 1) for x in v]}
    if callable(v):
        return {"t": "function"}
    raise ValueError(f"unhandled type {type(v)}")


def main():
    sys.path.insert(0, str(REPO_ROOT))
    from prefig.core import user_namespace as un

    sessions = []
    for name, steps in CORPUS:
        importlib.reload(un)
        out_steps = []
        for step in steps:
            op, expr = step[0], step[1]
            tol = step[2] if len(step) > 2 else None
            entry = {"op": op, "input": expr}
            if tol is not None:
                entry["tol"] = tol
            if op == "define":
                un.define(expr)
            elif op == "eval":
                result = un.valid_eval(expr)
                entry["expect"] = to_jsonable(result)
            elif op == "error":
                try:
                    un.valid_eval(expr)
                    raise AssertionError(f"expected error for {expr!r} but it evaluated")
                except AssertionError:
                    raise
                except Exception:
                    pass
            out_steps.append(entry)
        sessions.append({"name": name, "steps": out_steps})

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"sessions": sessions}, indent=1))
    n = sum(len(s["steps"]) for s in sessions)
    print(f"wrote {OUT} ({len(sessions)} sessions, {n} steps)")


if __name__ == "__main__":
    main()
