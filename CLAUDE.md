# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Superscript runtime library that evaluates CEL (Common Expression Language) expressions in mobile applications. It's built in Rust with bindings for Android (via JNI), iOS (via Swift/Objective-C), and WASM (for browser/Node.js).

## Core Architecture

### Main Components

- **src/lib.rs**: Main Rust library with CEL evaluation logic and platform interfaces
- **src/models.rs**: Data structures for serialization/deserialization between host platforms and Rust
- **src/ast.rs**: AST-related structures for expression parsing
- **src/cel.udl**: UniFFI definition file for generating platform bindings
- **wasm/**: WebAssembly wrapper for browser/Node.js environments
- **examples/**: Example implementations for different platforms

### Key Concepts

- **HostContext**: Trait that defines callbacks for dynamic property resolution from host platforms
- **PassableValue**: Enum for JSON-serializable values that can cross FFI boundaries
- **ExecutionContext**: Input structure containing variables, expressions, and platform callbacks
- **Dynamic Properties**: Platform-specific functions callable from CEL expressions (e.g., `computed.daysSince("app_launch")`)

## Build Commands

### Android
```bash
./build_android.sh
```
- Builds for multiple Android architectures using cross-compilation
- Generates JNI libraries and Kotlin bindings via UniFFI
- Output: `target/android/jniLibs/` and `target/android/java/uniffi/cel/`

### iOS
```bash
./build_ios.sh
```
- Builds XCFramework for iOS, macOS, watchOS, and visionOS
- Generates Swift bindings via UniFFI
- Output: `target/xcframeworks/libcel.xcframework` and `target/ios/`

### WASM
```bash
./build_wasm.sh
```
- Builds WASM module for browser and Node.js
- Builds TypeScript/JavaScript wrappers
- Output: `wasm/target/browser/` and `wasm/target/node/`

### Testing
```bash
# Main Rust tests
cargo test

# Node.js example tests
cd examples/node/tests && bun test

# WASM tests
cd wasm && npm test
```

## Development Workflow

1. **Core Logic Changes**: Modify `src/lib.rs` and related files
2. **Platform Binding Changes**: Update `src/cel.udl` and rebuild bindings
3. **WASM Changes**: Work in `wasm/src/` directory
4. **Testing**: Run tests in respective directories before committing

## Key Files to Understand

- **src/lib.rs:61-143**: Main evaluation functions (`evaluate_with_context`, `evaluate_ast_with_context`, `evaluate_ast`)
- **src/models.rs:23-46**: `PassableValue` enum defining the type system
- **wasm/src/lib.rs**: WASM-specific host context adapter
- **examples/**: Platform-specific usage examples

## Important Notes

- UniFFI doesn't support recursive enums, so JSON serialization is used for complex data structures
- The library supports both compiled CEL programs and AST evaluation
- Platform callbacks are handled differently for WASM (synchronous) vs native (asynchronous)
- Test files contain extensive examples of usage patterns