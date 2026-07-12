import { prefigBrowserApi } from "./compat-api";

/**
 * Compiles a PreFigure document using the Rust port compiled to WebAssembly,
 * a drop-in alternative to `PreFigureCompiler` (which uses Python via Pyodide).
 *
 * The WebAssembly module delegates math rendering, braille, and text
 * measurement to the same `prefigBrowserApi` object the Python version uses,
 * so no other browser-side code changes.
 *
 * The `@prefigure/prefig-wasm` package is produced from `rust/prefig-wasm`
 * with `wasm-pack build --target web` (see rust/PLAYGROUND_PLAN.md).
 */
export class PreFigureWasmCompiler {
    private wasm: typeof import("@prefigure/prefig-wasm") | null = null;
    private initPromise: Promise<void> | null = null;

    /** Safe to call multiple times. */
    async init(): Promise<void> {
        if (this.wasm) {
            return;
        }
        if (this.initPromise) {
            return this.initPromise;
        }
        this.initPromise = (async () => {
            const mod = await import("@prefigure/prefig-wasm");
            // `--target web` builds export a default init() that loads the .wasm
            if (typeof (mod as any).default === "function") {
                await (mod as any).default();
            }
            // MathJax / speech-rule-engine finish loading asynchronously
            await prefigBrowserApi.initFinished;
            mod.set_host_api(prefigBrowserApi);
            this.wasm = mod;
        })();
        return this.initPromise;
    }

    /** The version of the Rust prefig package that is loaded. */
    version(): string {
        if (!this.wasm) {
            throw new Error("Compiler not initialized");
        }
        return this.wasm.version();
    }

    /** Compile PreFigure source, returning the SVG and any annotations. */
    async compile(
        mode: "svg" | "tactile",
        source: string,
    ): Promise<{ svg: string; annotations: string }> {
        if (!this.wasm) {
            throw new Error("Compiler not initialized");
        }
        const result = this.wasm.build_from_string(mode, source) as {
            svg: string;
            annotations: string | null;
        };
        return { svg: result.svg, annotations: result.annotations ?? "" };
    }
}
