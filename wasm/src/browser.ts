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
 * base64-inlined wasm bytes. Needs no bundler wasm/asset support at all, so
 * it works under Bun, esbuild, and any bundler that treats `.wasm` imports
 * as plain file assets (where the bundler path throws
 * "wasm.__wbindgen_start is not a function" at import time).
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
    wasmModulePromise ??= loadBundlerModule().catch(() => loadInlineModule());
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
