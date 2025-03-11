import pkg from '@superwall/superscript/node';

const { evaluateWithContext } = pkg;
import type { ExecutionContext, SuperscriptHostContext, PassableValue } from '@superwall/superscript/node';

class TestHostContext implements SuperscriptHostContext {
    computed_property(name: string, args: [PassableValue]): PassableValue {
        console.log(`computed_property called with name: ${name}, args: ${JSON.stringify(args)}`);
        const parsedArgs = args;
        if (name === "randomUserValue") {
            const toReturn: PassableValue = {
                type: 'uint',
                value: 7
            };
            console.log("Computed property will return", toReturn);
            return toReturn;
        }
        console.error("Computed property not defined");
        return {
            type: 'string',
            value: `Computed property ${name} with args ${JSON.stringify(args)}`
        };
    }

    device_property(name: string, args: [PassableValue]): PassableValue {
        console.log(`device_property called with name: ${name}, args: ${JSON.stringify(args)}`);
        const parsedArgs = args;
        if (name === "daysSinceEvent") {
            const toReturn: PassableValue = {
                type: 'uint',
                value: 7
            };
            console.log("Device property will return", toReturn);
            return toReturn;
        }
        console.error("Device property not defined");
        return {
            type: 'string',
            value: `Device property ${name} with args ${JSON.stringify(args)}`
        };
    }
}

async function main() {
    try {
        console.log("TS Node example - WASM module initialized successfully");

        const context = new TestHostContext();

        const input: ExecutionContext = {
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
                randomUserValue: [{
                    type: "string",
                    value: "event_name"
                }]
            },
            expression: 'computed.randomUserValue("test") == user.some_value'
        };

        try {
            const result = await evaluateWithContext(input, context);
            console.log("Evaluation result:", result);
        } catch (error) {
            console.error("Evaluation error:", error);
            console.error("Error details:", (error as Error).stack);
        }

    } catch (error) {
        console.error("Initialization error:", error);
        console.error("Error details:", (error as Error).stack);
    }
}

console.log("Node.js environment detected");
main().catch((error) => {
    console.error(error);
    process.exit(1);
}); 