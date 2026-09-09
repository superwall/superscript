// Post-build check for the /browser fallback surface. No bundler involved:
// atob-decode the inline module, init the web-target glue, evaluate one
// known expression. Also asserts the generated VERSION matches package.json
// and that `cdnWasmUrl()` in browser.ts produces the jsDelivr URL (or null
// when the consumer has not opted in).

import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = join(import.meta.dir, '..');
const fileUrl = (rel: string) => pathToFileURL(join(root, rel)).href;

const pkg = (await Bun.file(join(root, 'package.json')).json()) as {
    name: string;
    version: string;
};

const { VERSION } = (await import(fileUrl('dist/esm/version.js'))) as {
    VERSION: string;
};

if (VERSION !== pkg.version) {
    throw new Error(
        `generated VERSION ${JSON.stringify(VERSION)} != package.json ${JSON.stringify(pkg.version)}`
    );
}

const g = globalThis as {
    SUPERWALL_SUPERSCRIPT_WASM_CDN?: boolean;
    SUPERWALL_SUPERSCRIPT_WASM_URL?: string;
};
const { cdnWasmUrl } = (await import(fileUrl('dist/esm/browser.js'))) as {
    cdnWasmUrl: () => string | null;
};

delete g.SUPERWALL_SUPERSCRIPT_WASM_CDN;
delete g.SUPERWALL_SUPERSCRIPT_WASM_URL;
if (cdnWasmUrl() !== null) {
    throw new Error(`cdnWasmUrl() must be null when neither opt-in flag is set; got ${cdnWasmUrl()}`);
}

g.SUPERWALL_SUPERSCRIPT_WASM_CDN = true;
const jsdelivr = `https://cdn.jsdelivr.net/npm/${pkg.name}@${pkg.version}/dist/target/web/superscript_bg.wasm`;
if (cdnWasmUrl() !== jsdelivr) {
    throw new Error(`cdnWasmUrl() with CDN flag: ${cdnWasmUrl()} != ${jsdelivr}`);
}

g.SUPERWALL_SUPERSCRIPT_WASM_URL = 'https://example.test/superscript_bg.wasm';
if (cdnWasmUrl() !== 'https://example.test/superscript_bg.wasm') {
    throw new Error(`cdnWasmUrl() URL override lost to CDN flag: ${cdnWasmUrl()}`);
}
delete g.SUPERWALL_SUPERSCRIPT_WASM_CDN;
delete g.SUPERWALL_SUPERSCRIPT_WASM_URL;

const glue = (await import(fileUrl('dist/target/web/superscript.js'))) as {
    default: (init: { module_or_path: BufferSource }) => Promise<unknown>;
    evaluate_with_context: (
        input: string,
        host: {
            computed_property: (name: string, args: string) => string;
            device_property: (name: string, args: string) => string;
        },
    ) => Promise<string>;
};
const inline = (await import(fileUrl('dist/target/web/superscript_bg_inline.js'))) as {
    wasmBase64: string;
};

const binary = Uint8Array.from(atob(inline.wasmBase64), (c) => c.charCodeAt(0));
const wasmFile = new Uint8Array(
    await Bun.file(join(root, 'dist/target/web/superscript_bg.wasm')).arrayBuffer(),
);
if (binary.byteLength !== wasmFile.byteLength) {
    throw new Error(
        `inline wasm length ${binary.byteLength} != superscript_bg.wasm ${wasmFile.byteLength}`
    );
}
for (let i = 0; i < binary.byteLength; i++) {
    if (binary[i] !== wasmFile[i]) {
        throw new Error(`inline wasm bytes diverge from superscript_bg.wasm at offset ${i}`);
    }
}

await glue.default({ module_or_path: binary });

const input = {
    variables: {
        map: {
            device: {
                type: 'map',
                value: { activeEntitlements: { type: 'list', value: [] } },
            },
            params: {
                type: 'map',
                value: {
                    event_name: { type: 'string', value: 'test_embed_redirect' },
                },
            },
            user: { type: 'map', value: {} },
        },
    },
    expression:
        '(size(device.activeEntitlements) == 0) && (params.event_name == "test_embed_redirect")',
    computed: {},
    device: { activeEntitlements: [] },
};
const host = {
    computed_property: () => JSON.stringify({ type: 'null', value: null }),
    device_property: () => JSON.stringify({ type: 'null', value: null }),
};

const result = await glue.evaluate_with_context(JSON.stringify(input), host);
const parsed = JSON.parse(result) as {
    Ok?: { type?: string; value?: unknown };
};
if (parsed.Ok?.type !== 'bool' || parsed.Ok.value !== true) {
    throw new Error(`unexpected eval result: ${result}`);
}

console.log(`test-browser-loader: ok (VERSION=${VERSION}, inline ${binary.byteLength} bytes)`);
