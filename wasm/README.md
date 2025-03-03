# Superscript WASM Module

This is the JS (WASM) runner for [Superscript expression language](https://github.com/superwall/Superscript).
The evaluator can call host environment functions and compute dynamic properties while evaluating expressions.

## Setup

First, import the module:
`import * as wasm from "@superwall/superscript";`

Next, create a WasmHostContext class to allow the expression evaluator to call the host environment (your JS)
and compute the dynamic properties, i.e. `platform.daysSinceEvent("event_name")`.
```javascript
/**
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
```


Then create an instance of the `WasmHostContext` and provide it together with the arguments to
`wasm.evaluateWithContext(arguments, wasmHostContext)`.

```javascript
async function main() {
       const context = new WasmHostContext();

       const input = {
           //Available variables
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
           computed: {
               //Computed values definitions
               daysSinceEvent: [{
                   type: "string",
                   value: "event_name"
               }]
           },
           device: {
               //Device value definitions
               daysSinceEvent: [{
                   type: "string",
                   value: "event_name"
               }]
           },
           expression: 'computed.daysSinceEvent("test") == user.some_value'
       };

       const inputJson = JSON.stringify(input);
       const result = await wasm.evaluate_with_context(inputJson, context);
}
```

