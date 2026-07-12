import { expose } from "comlink";
import { PreFigureCompiler } from "./compiler";
import { PreFigureWasmCompiler } from "./compiler-wasm";

// The Python-via-Pyodide compiler (current default).
const compiler = new PreFigureCompiler();

// The Rust-via-WebAssembly compiler (drop-in alternative; see
// rust/PLAYGROUND_PLAN.md). Selected by the main thread when the user opts in.
const wasmCompiler = new PreFigureWasmCompiler();

const add = (a: number, b: number) => a + b;

export const api = {
    compiler,
    wasmCompiler,
    add,
};

expose(api);
