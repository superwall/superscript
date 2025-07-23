## Superscript Runtime

[![Coverage Status](https://img.shields.io/badge/coverage-75.48%25-orange.svg)](./cobertura.xml)

This is the Superscript runtime library. Superscript is an expression language that builds upon CEL with enhanced null-safety, host integration, and mobile-optimized features.

The library can be used to evaluate Superscript expressions, with support for:
- Dynamic property resolution from host platform
- Built-in null-safety transformations  
- Type normalization and coercion
- WebAssembly (WASM) deployment

## Installation

To build the library, you'll need to install:

1. Rust (https://www.rust-lang.org/tools/install)
2. Docker (for cross) (https://docs.docker.com/get-docker/)
3. cross (https://github.com/cross-rs/cross)

## Building

To build the library, run:

```shell
./build_android.sh
```

(note: for the first run you will need to `chmod +x build.sh` and wait a bit until the docker images are downloaded)

This will:

- Clear the previously built jniLibs
- Build the library using cross for Defined Android images (add a new image in the script if needed).
- Copy the generated library to the `jniLibs` folder.
- Use UniFFI to generate the JNI bindings and a `cel.kt` file at `./src/uniffi/cel/cel.kt`.
- Copy the necessary files to the `./target/android/` folder.

## Usage

The library defines three methods exposed to the host platform, which you can use depending on the type of
expression you want to evaluate:

```idl
 // Evaluates a Superscript expression with provided variables and platform callbacks
 string evaluate_with_context(string definition, HostContext context);
 
 // Evaluates a Superscript AST expression with provided variables, platform callbacks
 string evaluate_ast_with_context(string definition, HostContext context);
 
 // Evaluates a pure Superscript AST expression
 string evaluate_ast(string ast);
 
 // Parses a Superscript expression into an AST
 string parse_to_ast(string expression);
```

The `HostContext` object is a callback interface allowing us to invoke host (iOS/Android) functions from our Rust code.
It provides two functions:
- `computed_property(name: String, args: String, callback: ResultCallback)` - For computed properties/functions
- `device_property(name: String, args: String, callback: ResultCallback)` - For device properties/functions

The functions pass in the name and the args (if required, serialized as JSON) of the dynamic function/property to invoke, and use a callback to return the result asynchronously.



### Android

To use the library in your Android application, you need to:
- Copy the `jniLibs` folder from `./target/android` to Android project's `superwall/src/main` folder.
- Copy the `cel.kt` file from `./src/uniffi/cel/cel.kt` to your Android project's `superwall/src/main/java/com/superwall/uniffi/cel/` folder.


The library exposes a single function currently:
`fn evaluate_with_context(definition: String, ctx: HostContext) -> String`

This function takes in a JSON containing the variables to be used and the expression to evaluate and returns the result.
The JSON is required to be in shape of `ExecutionContext`, which is defined as:

```json
{
  "variables": {
    // Map of variables that can be used in the expression
    // The key is the variable name, and the value is the variable value wrapped together with a type discriminator
    "map" : {
      "foo": {"type": "int", "value": 100},
      "some_property": {"type": "string", "value": "true"},
      "numbers": {
        "type" : "list",
        "value" : [
          {"type": "int", "value": 1},
          {"type": "int", "value": 2},
          {"type": "int", "value": 3}
        ]
      }
    }
  },
  // Host-exposed functions for computed properties
  "computed": {
    "daysSince": [{"type": "string", "value": "event_name"}],
    "some_property": []
  },
  // Host-exposed functions for device properties  
  "device": {
    "daysSince": [{"type": "string", "value": "event_name"}],
    "batteryLevel": []
  },
  // The Superscript expression to evaluate
  "expression": "device.daysSince(\"app_launch\") > 3.0 && computed.some_property == true"
}
```

## Key Features

### Null-Safe Evaluation
The library automatically transforms expressions to be null-safe:
- **Property access**: `obj.property` becomes `has(obj.property) ? obj.property : null`
- **Function calls**: `device.function()` becomes `hasFn("device.function") ? device.function() : false`

### Built-in Functions
Supported functions are defined in the `SUPPORTED_FUNCTIONS` constant:
- `maybe` - Null coalescing operator
- `toString`, `toBool`, `toInt`, `toFloat` - Type conversion extension functions
- `has` - Checks if a property exists
- `hasFn` - Checks if a function is available

### Host Integration
The `HostContext` provides async callbacks to resolve dynamic properties:
- `computed_property(name, args, callback)` - For computed functions
- `device_property(name, args, callback)` - For device functions

Results are returned as JSON-serialized `PassableValue` objects.

### Variable Normalization
The library automatically normalizes string values to their appropriate types:
- `"true"/"false"` → `Bool`
- Numeric strings → `Int`/`UInt`/`Float`
- Works recursively on nested objects and arrays

## Documentation

For a detailed explanation of the expression evaluation process, see [interpretation-flow.md](interpretation-flow.md).

### iOS

To use the library in your iOS application, you need to:

1. Make the build script executable:
- `chmod +x ./build_ios.sh`
2. Run the build script:
- `./build_ios.sh`
3. Get the resulting XCframework from the `./target/xcframeworks/` folder and add it to your iOS project together 
with generated swift files from `./target/ios`


This should give you a `HostContext` protocol:
```swift
public protocol HostContextProtocol : AnyObject {
    func computedProperty(name: String, args: String, callback: ResultCallback)
    func deviceProperty(name: String, args: String, callback: ResultCallback)
}
```

And a  `evaluateWithContext` method you can invoke:
```swift
public func evaluateWithContext(definition: String, context: HostContext) -> String
```


## Updating

When updating the library, you need to pay attention to uniffi bindings and ensure they match the signature of the library functions.
While it is tempting to migrate the library to use uniffi for the entire library, we still need to use JSON
for the input and output since UniFFI does not support recursive enums yet (such as PassableValue).
For that, track this [issue](https://github.com/mozilla/uniffi-rs/issues/396) for updates.
