import type { 
    SuperscriptHostContext, 
    ExecutionContext,
    UIntValue,
    IntValue,
    FloatValue,
    StringValue,
    BoolValue,
    ListValue,
    MapValue,
    BytesValue,
    TimestampValue,
    FunctionValue,
    NullValue,
    ValueType,
    PassableValue
} from './types';

// Import the WASM module generated for Node.js
import wasmModule from '../target/node/superscript.js';

export async function evaluateWithContext(
    input: ExecutionContext,
    context: SuperscriptHostContext
): Promise<string> {
    const hostContext = {
        computed_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            const result = context.computed_property(name, parsedArgs);
            // Ensure the result is properly formatted for serialization
            let res = JSON.stringify(result);
            console.log("Computed property result in node", res);
            return res;
        },
        device_property: (name: string, args: string) => {
            const parsedArgs = JSON.parse(args);
            const result = context.device_property(name, parsedArgs);
            // Ensure the result is properly formatted for serialization
            let res = JSON.stringify(result);
            console.log("Device property result in node", res);
            return res;
        }
    }
    const inputJson = JSON.stringify(input);
    return await wasmModule.evaluate_with_context(inputJson, hostContext);
}

// Re-export for CommonJS compatibility
export default {
    evaluateWithContext
};

export type {
    UIntValue,
    IntValue,
    FloatValue,
    StringValue,
    BoolValue,
    ListValue,
    MapValue,
    BytesValue,
    TimestampValue,
    FunctionValue,
    NullValue,
    ValueType,
    SuperscriptHostContext as WasmHostContext,
    ExecutionContext,
    PassableValue
}; 