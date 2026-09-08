import type { SuperscriptHostContext, ExecutionContext } from './types';
import { VERSION } from './version';

/** Minimal shape of the wasm-bindgen glue module we call into. */
interface WasmExports {
    evaluate_with_context(input: string, context: unknown): Promise<string>;
}

type CdnGlobals = {
    SUPERWALL_SUPERSCRIPT_WASM_URL?: unknown;
    SUPERWALL_SUPERSCRIPT_WASM_CDN?: unknown;
};

/** Opt-in CDN wasm URL. `SUPERWALL_SUPERSCRIPT_WASM_URL` (self-hosted) wins
 *  over `SUPERWALL_SUPERSCRIPT_WASM_CDN === true` (pinned jsDelivr). Read at
 *  the start of each load attempt (including the post-cooldown retry after
 *  a total miss); a late opt-in from an error handler still takes effect.
 *  After a successful load they are not read again. Returns null when
 *  neither is set — the CDN path is then skipped. */
function cdnWasmUrl(): string | null {
    const g = globalThis as CdnGlobals;
    if (typeof g.SUPERWALL_SUPERSCRIPT_WASM_URL === 'string' && g.SUPERWALL_SUPERSCRIPT_WASM_URL.length > 0) {
        return g.SUPERWALL_SUPERSCRIPT_WASM_URL;
    }
    if (g.SUPERWALL_SUPERSCRIPT_WASM_CDN === true) {
        return `https://cdn.jsdelivr.net/npm/@superwall/superscript@${VERSION}/dist/target/web/superscript_bg.wasm`;
    }
    return null;
}

let wasmModulePromise: Promise<WasmExports> | null = null;
/** Last total-failure, used to fail-fast during the cooldown instead of
 *  re-running every path (and the CDN fetch) on each `evaluateWithContext`. */
let lastFailure: { error: unknown; at: number } | null = null;
const RETRY_COOLDOWN_MS = 10_000;

/** Decoded inline wasm bytes, cached independently of whether
 *  `glue.default(...)` then succeeds — a retry must not redo `atob` over
 *  the ~1.5 M-char blob. Dropped after a successful init: the overall
 *  load is then memoized in `wasmModulePromise` and this path is never
 *  re-entered. */
let inlineWasmBytesPromise: Promise<Uint8Array> | null = null;

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
 * Decode the base64-inlined wasm once. Cached on the promise so a retry
 * after a failed `glue.default` does not redo `atob`; the cache is
 * dropped after a successful init.
 */
function loadInlineWasmBytes(): Promise<Uint8Array> {
    inlineWasmBytesPromise ??= import('../target/web/superscript_bg_inline.js')
        .then((inline) =>
            Uint8Array.from(atob(inline.wasmBase64), (c) => c.charCodeAt(0)),
        )
        .catch((error) => {
            // Don't cache a failed import (missing/mangled chunk) —
            // a later retry after cooldown should try again.
            inlineWasmBytesPromise = null;
            throw error;
        });
    return inlineWasmBytesPromise;
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
    const [glue, binary] = await Promise.all([
        import('../target/web/superscript.js'),
        loadInlineWasmBytes(),
    ]);
    await glue.default({ module_or_path: binary });
    // Init succeeded, so the overall load is memoized and this path is
    // never re-entered — drop the ~1.15 MB decode cache.
    inlineWasmBytesPromise = null;
    return glue as unknown as WasmExports;
}

/**
 * Opt-in last-resort path: fetch wasm bytes from a URL and initialise the
 * local `--target web` glue with them. Only registered when the consumer
 * set `SUPERWALL_SUPERSCRIPT_WASM_CDN` or `SUPERWALL_SUPERSCRIPT_WASM_URL`
 * before the first eval — see wasm/README.md.
 */
async function loadCdnModule(url: string): Promise<WasmExports> {
    const glue = await import('../target/web/superscript.js');
    // Pass the URL string — wasm-bindgen's web-target init fetches it.
    // (`fetch()` here would skip MIME/streaming checks the glue already does.)
    await glue.default({ module_or_path: url });
    return glue as unknown as WasmExports;
}

/**
 * Try each load path in order. Keeps every failure cause — bundler wasm
 * handling fails in enough surprising ways that losing the earlier errors
 * would make field reports undiagnosable.
 */
async function tryLoadPaths(): Promise<WasmExports> {
    const failures: { path: string; error: unknown }[] = [];
    const paths: [string, () => Promise<WasmExports>][] = [
        ['bundler target', loadBundlerModule],
        ['inline web target', loadInlineModule],
    ];
    const cdnUrl = cdnWasmUrl();
    if (cdnUrl !== null) {
        paths.push(['cdn web target', () => loadCdnModule(cdnUrl)]);
    }
    for (const [path, load] of paths) {
        try {
            return await load();
        } catch (error) {
            failures.push({ path, error });
        }
    }
    const detail = failures
        .map((f) => `${f.path}: ${String(f.error)}`)
        .join('; ');
    const hint =
        cdnUrl === null
            ? '; set globalThis.SUPERWALL_SUPERSCRIPT_WASM_CDN = true or SUPERWALL_SUPERSCRIPT_WASM_URL before the next load attempt for a network fallback'
            : '';
    const error = new Error(
        `superscript: all wasm load paths failed — ${detail}${hint}`
    );
    (error as Error & { failures: unknown }).failures = failures;
    throw error;
}

function loadWasmModule(): Promise<WasmExports> {
    if (wasmModulePromise) return wasmModulePromise;

    // A transient failure (offline during the CDN fetch) should be
    // retryable, but a persistently failing environment (CSP blocking
    // connect-src, mangled inline chunk, still-offline) must not issue a
    // fresh ~1.15 MB fetch + full base64 decode on every evaluation.
    // Fail fast for RETRY_COOLDOWN_MS, then allow a single new attempt.
    if (lastFailure && Date.now() - lastFailure.at < RETRY_COOLDOWN_MS) {
        return Promise.reject(lastFailure.error);
    }

    wasmModulePromise = tryLoadPaths().then(
        (mod) => {
            lastFailure = null;
            return mod;
        },
        (error) => {
            lastFailure = { error, at: Date.now() };
            wasmModulePromise = null;
            throw error;
        },
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

export { cdnWasmUrl };
