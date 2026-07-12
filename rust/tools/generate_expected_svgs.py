#!/usr/bin/env python3
"""Generate expected SVGs from the reference Python implementation.

Builds every example diagram with the PreFigure Python package in the
"pretext" environment (so <read>/<image> resolve files under data/) and writes
the SVG the Rust port is checked against.

Usage (from the repo root):
    poetry run python rust/tools/generate_expected_svgs.py [DOCS_CHECKOUT]

DOCS_CHECKOUT is a checkout of https://github.com/davidaustinm/prefigure-docs;
its source/prefigure/*.xml diagrams (and source/data/) are built too when given.

Outputs, under rust/prefig-core/tests/:
    example_diagrams/{repo,docs}/*.xml   sources that build to non-trivial SVG
    example_diagrams/docs/data/          data files for <read>/<image>
    expected_svgs/{repo,docs}/*.svg      the Python-built SVGs
    expected_svgs/manifest.json          what built and what was skipped
"""

import json
import os
import shutil
import sys
from pathlib import Path

import lxml.etree as ET

REPO_ROOT = Path(__file__).resolve().parents[2]
TESTS = REPO_ROOT / "rust" / "prefig-core" / "tests"
EXAMPLES = TESTS / "example_diagrams"
EXPECTED = TESTS / "expected_svgs"

# SVGs with only <defs> (or less) mean the diagram did not really build (e.g. a
# missing data file); we skip those rather than check in an empty golden.
MIN_MEANINGFUL_CHILDREN = 2


def build_one(xml_path: Path, environment="pretext"):
    """Build one source file, returning (svg, annotations) or None on failure."""
    from prefig.core import parse, user_namespace
    import importlib

    importlib.reload(user_namespace)
    ns = {"pf": "https://prefigure.org"}
    tree = ET.parse(str(xml_path))
    diagrams = tree.xpath("//pf:diagram", namespaces=ns) + tree.xpath("//diagram")
    if not diagrams:
        return None
    diagram = diagrams[0]
    for elem in diagram.getiterator():
        if not isinstance(elem, (ET._Comment, ET._ProcessingInstruction)):
            elem.tag = ET.QName(elem).localname
    parse.check_duplicate_handles(diagram, set())
    result = parse.mk_diagram(
        diagram,
        "svg",
        None,  # publication
        xml_path.stem,  # filename → id prefix
        False,  # suppress caption
        None,  # diagram number
        environment,
        return_string=True,
    )
    if result is None:
        return None
    return result


def meaningful(svg: str) -> bool:
    try:
        root = ET.fromstring(svg.encode("utf-8"))
    except Exception:
        return False
    return len(root) >= MIN_MEANINGFUL_CHILDREN


def build_category(src_dir: Path, category: str, data_dir: Path | None):
    out_examples = EXAMPLES / category
    out_expected = EXPECTED / category
    out_examples.mkdir(parents=True, exist_ok=True)
    out_expected.mkdir(parents=True, exist_ok=True)

    # data files are resolved relative to the working directory in the
    # pretext environment; run from the category example dir
    if data_dir and data_dir.is_dir():
        dest = out_examples / "data"
        if dest.exists():
            shutil.rmtree(dest)
        shutil.copytree(data_dir, dest)

    built, skipped = [], []
    prev_cwd = Path.cwd()
    os.chdir(out_examples)
    try:
        for xml_path in sorted(src_dir.glob("*.xml")):
            name = xml_path.stem
            try:
                result = build_one(xml_path)
            except Exception as e:  # noqa: BLE001 - report and continue
                skipped.append((name, f"error: {e}"))
                continue
            if result is None or not meaningful(result[0]):
                skipped.append((name, "empty or trivial output"))
                continue
            svg, annotations = result
            shutil.copy(xml_path, out_examples / f"{name}.xml")
            (out_expected / f"{name}.svg").write_text(svg)
            if annotations:
                (out_expected / f"{name}.xml").write_text(annotations)
            built.append(name)
    finally:
        os.chdir(prev_cwd)
    return built, skipped


def main():
    sys.path.insert(0, str(REPO_ROOT))

    manifest = {}
    repo_src = REPO_ROOT / "prefig" / "resources" / "examples"
    built, skipped = build_category(repo_src, "repo", None)
    manifest["repo"] = {"built": built, "skipped": skipped}

    docs_data = None
    if len(sys.argv) > 1:
        docs = Path(sys.argv[1])
        docs_src = docs / "source" / "prefigure"
        docs_data = docs / "source" / "data"
        if docs_src.is_dir():
            built, skipped = build_category(docs_src, "docs", docs_data)
            manifest["docs"] = {"built": built, "skipped": skipped}

    # synthetic examples exercising elements the docs set doesn't cover
    synth_src = REPO_ROOT / "rust" / "tools" / "synthetic_examples"
    if synth_src.is_dir():
        built, skipped = build_category(synth_src, "synth", docs_data)
        manifest["synth"] = {"built": built, "skipped": skipped}

    EXPECTED.mkdir(parents=True, exist_ok=True)
    (EXPECTED / "manifest.json").write_text(json.dumps(manifest, indent=1))
    counts = {k: (len(v["built"]), len(v["skipped"])) for k, v in manifest.items()}
    print(f"expected-SVG manifest (built, skipped): {counts}")


if __name__ == "__main__":
    main()
