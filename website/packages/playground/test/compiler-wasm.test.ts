import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
    version,
    set_host_api,
    build_from_string,
} from "@prefigure/prefig-wasm";

// Directory of Python-built example diagrams (shared with the Rust tests).
const examplesDir = join(
    dirname(fileURLToPath(import.meta.url)),
    "../../../../tests/examples",
);

// A deterministic stand-in for PrefigBrowserApi. Real MathJax/SRE would render
// math; here labels get fixed-size placeholders so the test is hermetic. The
// drawing (grids, axes, curves, points) does not depend on the host.
const mockHostApi = {
    measure_text: (text: string) => [text.length * 8, 10, 3],
    translate_text: (text: string) => text,
    processMath: () =>
        `<svg xmlns="http://www.w3.org/2000/svg" width="1ex" height="1ex" style="vertical-align: 0ex"><defs/></svg>`,
    processBraille: () => "⠠",
};

function readExample(rel: string): string {
    return readFileSync(join(examplesDir, rel), "utf-8");
}

describe("prefig-wasm compiler", () => {
    it("reports a version", () => {
        expect(version()).toMatch(/^\d+\.\d+\.\d+$/);
    });

    it("compiles example diagrams to non-trivial SVG", () => {
        set_host_api(mockHostApi);
        // Examples that render without needing real math rendering to be valid.
        const cases: { file: string; width: string }[] = [
            { file: "hand_crafted/tangent.xml", width: "310" },
            { file: "hand_crafted/roots_of_unity.xml", width: "310" },
            { file: "extracted_from_docs/polar-grid-1.xml", width: "310" },
            { file: "hand_crafted/implicit.xml", width: "310" },
            // boolean <shape> ops need the `shapes` feature in the wasm build;
            // this guards against it silently disappearing from the feature set
            { file: "extracted_from_docs/shape_difference.xml", width: "310" },
        ];
        for (const { file, width } of cases) {
            const { svg } = build_from_string("svg", readExample(file));
            expect(svg, file).toMatch(/^<svg/);
            expect(svg, file).toContain(`width="${width}"`);
            // real drawing content, not just an empty canvas
            expect(svg.length, file).toBeGreaterThan(1000);
            expect(svg, file).toMatch(/<(path|line|circle)/);
        }
    });

    it("computes boolean shape operations (shapes feature)", () => {
        set_host_api(mockHostApi);
        const { svg } = build_from_string(
            "svg",
            readExample("extracted_from_docs/shape_difference.xml"),
        );
        // The A-minus-B region is a path computed by the boolean op and filled
        // magenta. Without the `shapes` feature in the wasm build, no such
        // path is emitted at all (the op logs an error and bails).
        expect(svg).toMatch(/<path(?=[^>]*\bd="M )(?=[^>]*fill="magenta")/);
    });

    it("generates default speech annotations when the source has none", () => {
        // In the pyodide environment, a diagram without <annotations> gets an
        // auto-generated speech tree (diagram_to_speech), matching Python.
        set_host_api(mockHostApi);
        const { annotations } = build_from_string(
            "svg",
            `<diagram dimensions="(100,100)"><coordinates bbox="[-1,-1,1,1]"><grid/></coordinates></diagram>`,
        );
        expect(annotations).toContain("speech2=");
        expect(annotations).toContain("A grid element");
    });

    it("throws a clear error for sourceless input", () => {
        set_host_api(mockHostApi);
        expect(() => build_from_string("svg", "<nope/>")).toThrow(/diagram/);
    });
});
