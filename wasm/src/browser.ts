import type { SuperscriptHostContext, ExecutionContext } from './types';

// Dynamic import for the browser WASM module
async function loadWasmModule() {
    const wasm = await import('../target/browser/superscript.js');
    return wasm;
}

let wasmModule: any = null;

export async function evaluateWithContext(
    input: ExecutionContext,
    context: SuperscriptHostContext
): Promise<string> {
    if (!wasmModule) {
        wasmModule = await loadWasmModule();
    }
    
    const hostContext = {
        computed_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            let res  = JSON.stringify(context.computed_property(name, parsedArgs))
            console.log("Computed property result in browser", res);
            return res;
        },
        device_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            let res  = JSON.stringify(context.device_property(name, parsedArgs))
            console.log("Device property result in browser", res);
            return res;
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