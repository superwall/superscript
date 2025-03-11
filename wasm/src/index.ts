import type { SuperscriptHostContext, ExecutionContext, PassableValue } from './types';

declare const wasm_bindgen: any;

let wasmModule: any;

export async function initialize(): Promise<void> {
    wasmModule = await wasm_bindgen();
}

export async function evaluateWithContext(
    input: ExecutionContext,
    context: SuperscriptHostContext
): Promise<string> {
    if (!wasmModule) {
        await initialize();
    }
    const hostContext = {
        computed_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            let res  = JSON.stringify(context.computed_property(name, parsedArgs))
            console.log("Computed property result in index", res);
            return res;
        },
        device_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            let res  = JSON.stringify(context.device_property(name, parsedArgs))
            console.log("Device property result in index", res);
            return res;
        }
    }
    const inputJson = JSON.stringify(input);
    return await wasmModule.evaluate_with_context(inputJson, hostContext);
}

export type {
    SuperscriptHostContext,
    ValueType,
    ExecutionContext,
    PassableValue,
    WasmModule
} from './types';