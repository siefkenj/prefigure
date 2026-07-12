#!/usr/bin/env bash
# Build every example diagram with the reference Python implementation and
# save the resulting SVGs; the Rust test suite compares its own output to them.
#
# Usage: rust/tools/generate_expected_svgs.sh [DOCS_EXAMPLES_DIR]
#   DOCS_EXAMPLES_DIR: a checkout of https://github.com/davidaustinm/prefigure-docs
#                      (its source/prefigure/*.xml diagrams are built too, if given)
#
# Outputs:
#   rust/prefig-core/tests/example_diagrams/{repo,docs}/*.xml    (sources)
#   rust/prefig-core/tests/expected_svgs/{repo,docs}/*.svg       (Python-built SVGs)
#   rust/prefig-core/tests/expected_svgs/manifest.json           (pass/fail record)
set -u
cd "$(dirname "$0")/../.."   # repo root

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
FIX=rust/prefig-core/tests/example_diagrams
GOLD=rust/prefig-core/tests/expected_svgs
mkdir -p "$FIX/repo" "$GOLD/repo"

build_dir () {  # $1 = src dir with xml files, $2 = category (repo|docs)
    local src=$1 cat=$2
    mkdir -p "$WORK/$cat" "$FIX/$cat" "$GOLD/$cat"
    cp "$src"/*.xml "$WORK/$cat/"
    # <read> data files resolve to data/ under the build cwd (pretext environment)
    if [ -d "$src/../data" ]; then
        cp -r "$src/../data" "$WORK/$cat/data"
        # the Rust parity test needs the same data files
        rm -rf "$FIX/$cat/data" && cp -r "$src/../data" "$FIX/$cat/data"
    fi
    PREFIG_BIN="$PWD/.venv/bin/prefig"
    ls "$WORK/$cat"/*.xml | xargs -P 4 -I{} bash -c '
        f={}; name=$(basename "$f" .xml)
        cd "$(dirname "$f")"
        if timeout 120 "'"$PREFIG_BIN"'" build "$name.xml" >/dev/null 2>&1 \
           && [ -s "output/$name.svg" ]; then
            echo "PASS $name"
        else
            echo "FAIL $name"
        fi
    ' | sort > "$WORK/$cat.results"
    while read -r status name; do
        if [ "$status" = PASS ]; then
            cp "$WORK/$cat/$name.xml" "$FIX/$cat/$name.xml"
            cp "$WORK/$cat/output/$name.svg" "$GOLD/$cat/$name.svg"
        fi
    done < "$WORK/$cat.results"
}

build_dir prefig/resources/examples repo
if [ $# -ge 1 ] && [ -d "$1/source/prefigure" ]; then
    build_dir "$1/source/prefigure" docs
fi

python3 - "$WORK" <<'PY'
import json, sys, pathlib
work = pathlib.Path(sys.argv[1])
manifest = {}
for res in work.glob("*.results"):
    cat = res.stem
    entries = [line.split() for line in res.read_text().splitlines() if line.strip()]
    manifest[cat] = {
        "pass": sorted(n for s, n in entries if s == "PASS"),
        "fail": sorted(n for s, n in entries if s == "FAIL"),
    }
out = pathlib.Path("rust/prefig-core/tests/expected_svgs/manifest.json")
out.write_text(json.dumps(manifest, indent=1))
counts = {k: (len(v["pass"]), len(v["fail"])) for k, v in manifest.items()}
print("expected-SVG manifest:", counts)
PY
