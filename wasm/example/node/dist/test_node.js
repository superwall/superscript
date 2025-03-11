import pkg from '@superwall/superscript';
const { evaluateWithContext } = pkg;
class TestHostContext {
    computed_property(name, args) {
        console.log(`computed_property called with name: ${name}, args: ${JSON.stringify(args)}`);
        const parsedArgs = args;
        if (name === "randomUserValue") {
            const toReturn = {
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
    device_property(name, args) {
        console.log(`device_property called with name: ${name}, args: ${JSON.stringify(args)}`);
        const parsedArgs = args;
        if (name === "daysSinceEvent") {
            const toReturn = {
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
        }
        catch (error) {
            console.error("Evaluation error:", error);
            console.error("Error details:", error.stack);
        }
    }
    catch (error) {
        console.error("Initialization error:", error);
        console.error("Error details:", error.stack);
    }
}
console.log("Node.js environment detected");
main().catch((error) => {
    console.error(error);
    process.exit(1);
});
