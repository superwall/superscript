import type { SuperscriptHostContext, ExecutionContext } from './types';

/** Minimal shape of the wasm-bindgen glue module we call into. */
interface WasmExports {
    evaluate_with_context(input: string, context: unknown): Promise<string>;
}

let wasmModulePromise: Promise<WasmExports> | null = null;

/**
 * Primary path: wasm-pack `--target bundler` output. It does
 * `import * as wasm from './superscript_bg.wasm'`, which only works in
 * bundlers with WebAssembly ESM integration (webpack `asyncWebAssembly`,
 * vite-plugin-wasm, Rollup wasm plugin, …).
 */
async function loadBundlerModule(): Promise<WasmExports> {
    return await import('../target/browser/superscript.js');
}

/**
 * Fallback path: wasm-pack `--target web` output initialised from
 * base64-inlined wasm bytes. Covers bundlers that resolve `.wasm` imports as
 * plain file assets — Bun out of the box, esbuild with `--loader:.wasm=file`
 * — where the bundler path throws "wasm.__wbindgen_start is not a function"
 * at import time. Bundlers with no `.wasm` handling at all (default esbuild,
 * Next.js' default webpack config) still fail at *build* time before either
 * path can run; those need `--loader:.wasm=file` resp.
 * `experiments.asyncWebAssembly: true` in the consumer config.
 *
 * Both modules are behind dynamic imports, so wasm-capable bundlers put the
 * inline chunk in a separate lazily-loaded chunk that is never fetched on
 * the happy path.
 */
async function loadInlineModule(): Promise<WasmExports> {
    const [glue, inline] = await Promise.all([
        import('../target/web/superscript.js'),
        import('../target/web/superscript_bg_inline.js'),
    ]);
    const binary = Uint8Array.from(atob(inline.wasmBase64), (c) =>
        c.charCodeAt(0)
    );
    await glue.default({ module_or_path: binary });
    return glue as unknown as WasmExports;
}

function loadWasmModule(): Promise<WasmExports> {
    // Keep both failure causes: the memoized rejection is all future callers
    // ever see, and bundler wasm handling fails in enough surprising ways
    // that losing the primary error would make field reports undiagnosable.
    wasmModulePromise ??= loadBundlerModule().catch((bundlerError) =>
        loadInlineModule().catch((inlineError) => {
            const error = new Error(
                `superscript: both wasm load paths failed — bundler target: ${String(
                    bundlerError
                )}; inline web target: ${String(inlineError)}`
            );
            (error as Error & {
                bundlerError: unknown;
                inlineError: unknown;
            }).bundlerError = bundlerError;
            (error as Error & {
                bundlerError: unknown;
                inlineError: unknown;
            }).inlineError = inlineError;
            throw error;
        })
    );
    return wasmModulePromise;
}

export async function evaluateWithContext(
    input: ExecutionContext,
    context: SuperscriptHostContext
): Promise<string> {
    const wasmModule = await loadWasmModule();

    const hostContext = {
        computed_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            return JSON.stringify(context.computed_property(name, parsedArgs));
        },
        device_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            return JSON.stringify(context.device_property(name, parsedArgs));
        }
    }
    const inputJson = JSON.stringify(input);
    return await wasmModule.evaluate_with_context(inputJson, hostContext);
}

export type {
    SuperscriptHostContext as WasmHostContext,
    ExecutionContext,
    ValueType,
} from './types';
