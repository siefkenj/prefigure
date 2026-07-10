// Node tests for the WebAssembly build.
// Run with `npm test` (builds the wasm first) or `npm run test:only`.

import { describe, it, expect } from "vitest";
import { version, Evaluator, build_from_string } from "../pkg/prefig_wasm.js";

describe("version", () => {
    it("reports the crate version", () => {
        expect(version()).toMatch(/^\d+\.\d+\.\d+$/);
    });
});

describe("Evaluator", () => {
    it("evaluates arithmetic", () => {
        const ev = new Evaluator();
        expect(ev.evaluate("3 + 4 * 2")).toBe(11);
        expect(ev.evaluate("2^5")).toBe(32); // ^ means exponent
        expect(ev.evaluate("-7 // 2")).toBe(-4); // Python-style floor division
    });

    it("evaluates vectors as arrays", () => {
        const ev = new Evaluator();
        expect(ev.evaluate("(1, 2) + (3, 4)")).toEqual([4, 6]);
        expect(ev.evaluate("midpoint((0,0), (2,4))")).toEqual([1, 2]);
    });

    it("remembers definitions", () => {
        const ev = new Evaluator();
        ev.define("a = 5");
        ev.define("f(x) = x^2 + a");
        expect(ev.evaluate("f(3)")).toBe(14);
    });

    it("keeps definitions separate between instances", () => {
        const first = new Evaluator();
        first.define("a = 5");
        const second = new Evaluator();
        expect(() => second.evaluate("a")).toThrow(/Unrecognized name/);
    });

    it("returns dictionaries as objects and strings as strings", () => {
        const ev = new Evaluator();
        expect(ev.evaluate("{'color': 'red', 'width': 2}")).toEqual({
            color: "red",
            width: 2,
        });
        expect(ev.evaluate("#ff0000")).toBe("#ff0000");
        expect(ev.evaluate("rgb(255, 0, 0)")).toBe("rgb(255,0,0)");
    });

    it("rejects expressions the Python version rejects", () => {
        const ev = new Evaluator();
        expect(() => ev.evaluate("x < 3")).toThrow();
        expect(() => ev.evaluate("__import__('os')")).toThrow();
    });
});

describe("build_from_string", () => {
    it("is not implemented yet and says so", () => {
        expect(() => build_from_string("svg", "<diagram/>")).toThrow(
            /not implemented yet/,
        );
    });
});
