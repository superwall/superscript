
import wasm from '@superwall/superscript';
const { evaluate_with_context, ExecutionContext } = wasm;

/**
 * An example of a WasmHostContext implementation from @file src/lib.rs.
 * This contract allows the expression evaluator to call the host environment (your JS)
 * and compute the dynamic properties, i.e. `platform.daysSinceEvent("event_name")`.
 *
 * @param name - The name of the computed property or function being invoked.
 * @param args - JSON string of the arguments for the function.
 * @returns JSON-serialized string of the computed property value.
 * */
class WasmHostContext {
    computed_property(name, args) {
        console.log(`computed_property called with name: ${name}, args: ${args}`);
        const parsedArgs = JSON.parse(args);
        if (name === "daysSinceEvent") {
            let toReturn =  JSON.stringify({
                type: "uint",
                value: 7
            });
            console.log("Computed property will return", toReturn);
            return toReturn
        }
        console.error("Computed property not defined")
        return JSON.stringify({
            type: "string",
            value: `Computed property ${name} with args ${args}`
        });
    }

    device_property(name, args) {
        console.log(`computed_property called with name: ${name}, args: ${args}`);
        const parsedArgs = JSON.parse(args);
        if (name === "daysSinceEvent") {
            let toReturn =  JSON.stringify({
                type: "uint",
                value: 7
            });
            console.log("Computed property will return", toReturn);
            return toReturn
        }
        console.error("Computed property not defined")
        return JSON.stringify({
            type: "string",
            value: `Computed property ${name} with args ${args}`
        });
    }

}

/**
 * Entry point for testing the WASM module.
 * */
async function main() {
    try {
        console.log("WASM module initialized successfully");

        const context = new WasmHostContext();

        const input = {
            variables: {
                map: {
                    user: {
                        type: "map",
                        value: {
                            should_display: {
                                type: "bool",
                                value: true
                            },
                            some_value: {
                                type: "uint",
                                value: 7
                            }
                        }
                    }
                }
            },
            device: {
                daysSinceEvent: [{
                    type: "string",
                    value: "event_name"
                }]
            },
            computed: {
                daysSinceEvent: [{
                    type: "string",
                    value: "event_name"
                }]
            },
            expression: 'device.daysSinceEvent("test") == user.some_value'
        };

        const inputJson = JSON.stringify(input);
        console.log("Input JSON:", inputJson);

        try {
            const result = await evaluate_with_context(inputJson, context);
            console.log("Evaluation result:", result);
        } catch (error) {
            console.error("Evaluation error:", error);
            console.error("Error details:", error.stack);
        }

        // AST evaluation

    } catch (error) {
        console.error("Initialization error:", error);
        console.error("Error details:", error.stack);
    }
}

// Check if we're in a browser environment
console.log("Node.js environment detected")
// Node.js environment
main().catch((error) => {
    console.error(error);
    process.exit(1);
});
