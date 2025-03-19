# Superscript WASM Module

This is the JS (WASM) runner for [Superscript expression language](https://github.com/superwall/Superscript).
The evaluator can call host environment functions and compute dynamic properties while evaluating expressions.

## Setup

First, import the module:
`import * as wasm from "@superwall/superscript";`

Next, create a WasmHostContext class to allow the expression evaluator to call the host environment (your JS)
and compute the dynamic properties, i.e. `platform.daysSinceEvent("event_name")`.

```typescript
/**
* @param name - The name of the computed property or function being invoked.
* @param args - arguments for the function.
* @returns a resolved value.
* */
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
```


Then create an instance of the `WasmHostContext` and provide it together with the arguments to
`wasm.evaluateWithContext(arguments, wasmHostContext)`.

```javascript
async function main() {
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

        const result = await evaluateWithContext(input, context);
}
```

