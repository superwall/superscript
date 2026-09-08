# Superscript WASM Module

This is the JS (WASM) runner for [Superscript expression language](https://github.com/superwall/Superscript).
The evaluator can call host environment functions and compute dynamic properties while evaluating expressions.

## Entries

- `@superwall/superscript/node` — Node/Bun. Loads wasm via `fs`.
- `@superwall/superscript/browser` — browsers. Tries wasm-pack `--target bundler` first (`import` of the `.wasm` file). If that throws at runtime (Bun, esbuild with `--loader:.wasm=file`), it falls back to a `--target web` build initialised from base64-inlined bytes, with no network.

Default esbuild and Next.js webpack still fail **at build time** on the `.wasm` import. Those need `--loader:.wasm=file` or `experiments.asyncWebAssembly: true`.

## Optional CDN fallback

The browser entry does **not** fetch wasm from the network unless you opt in before a **load attempt**. The flags are re-read at the start of each attempt (including the post-cooldown retry after a total miss), so setting them from an error handler still takes effect. After a successful load they are not read again.

```js
// Exact-version file on jsDelivr (must match this package's version).
globalThis.SUPERWALL_SUPERSCRIPT_WASM_CDN = true;

// Or a self-hosted copy. Wins if both are set.
globalThis.SUPERWALL_SUPERSCRIPT_WASM_URL =
  "https://your.cdn.example/superscript_bg.wasm";
```

jsDelivr URL shape: `https://cdn.jsdelivr.net/npm/@superwall/superscript@<version>/dist/target/web/superscript_bg.wasm`.

CSP: allow the origin you actually fetch in `connect-src` (for the default, `https://cdn.jsdelivr.net`). Setting neither flag issues no request and needs no CSP change.

## Setup

First, import the matching entry:

```ts
import { evaluateWithContext } from "@superwall/superscript/browser";
// Node/Bun: import { evaluateWithContext } from "@superwall/superscript/node";
```

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


Then create an instance of the host context and pass it with the arguments to
`evaluateWithContext(input, hostContext)`.

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

