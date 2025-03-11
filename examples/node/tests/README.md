# Superscript Tests

This directory contains comprehensive tests for the Superscript module, testing all possible value types and both device and computed properties.

## Test Coverage

The tests cover:

1. All possible value types from the `types.ts` file:
   - String
   - Int
   - UInt
   - Float
   - Bool
   - List
   - Map
   - Bytes
   - Timestamp
   - Function
   - Null

2. Testing both device and computed properties with all types

3. Each evaluation returns the expected result: `{"Ok":{"type":"bool","value":true}}`

## Running the Tests

To run the tests:

1. Install dependencies:
   ```bash
   npm install
   ```

2. Run the tests:
   ```bash
   npm test
   ```

Alternatively, you can run the test file directly:

```bash
node --experimental-vm-modules dist/superscript.test.js
```

## Test Structure

- `superscript.test.ts`: The main test file that tests all value types and properties
- `TestHostContext`: A class that implements the `SuperscriptHostContext` interface and handles both computed and device properties for all possible value types

## Adding New Tests

To add new tests, add new test cases to the `runAllTests` function in `superscript.test.ts`. 