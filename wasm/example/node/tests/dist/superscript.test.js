import pkg from '@superwall/superscript';
const { evaluateWithContext } = pkg;
import assert from 'assert';
/**
 * Test host context implementation that handles both computed and device properties
 * for all possible value types.
 */
class TestHostContext {
    computed_property(name, args) {
        console.log(`computed_property called with name: ${name}, args:`, JSON.stringify(args));
        // Handle different property types based on the name
        switch (name) {
            case "stringProperty":
                return {
                    type: "string",
                    value: "test-string-value"
                };
            case "intProperty":
                return {
                    type: "int",
                    value: -42
                };
            case "uintProperty":
                return {
                    type: "uint",
                    value: 42
                };
            case "floatProperty":
                return {
                    type: "float",
                    value: 3.14159
                };
            case "boolProperty":
                return {
                    type: "bool",
                    value: true
                };
            case "listProperty":
                return {
                    type: "list",
                    value: [
                        { type: "string", value: "item1" },
                        { type: "uint", value: 2 }
                    ]
                };
            case "mapProperty":
                return {
                    type: "map",
                    value: {
                        key1: { type: "string", value: "value1" },
                        key2: { type: "uint", value: 2 }
                    }
                };
            case "bytesProperty":
                return {
                    type: "bytes",
                    value: [72, 101, 108, 108, 111] // "Hello" in ASCII
                };
            case "timestampProperty":
                return {
                    type: "timestamp",
                    value: Date.now()
                };
            case "functionProperty":
                return {
                    type: "function",
                    value: ["testFunction", { type: "string", value: "arg" }]
                };
            case "nullProperty":
                return {
                    type: "null",
                    value: null
                };
            default:
                console.error(`Computed property not defined: ${name}`);
                return {
                    type: "string",
                    value: `Unknown computed property: ${name}`
                };
        }
    }
    device_property(name, args) {
        console.log(`device_property called with name: ${name}, args:`, JSON.stringify(args));
        // Handle different property types based on the name
        switch (name) {
            case "stringDeviceProperty":
                return {
                    type: "string",
                    value: "device-string-value"
                };
            case "intDeviceProperty":
                return {
                    type: "int",
                    value: -100
                };
            case "uintDeviceProperty":
                return {
                    type: "uint",
                    value: 100
                };
            case "floatDeviceProperty":
                return {
                    type: "float",
                    value: 2.71828
                };
            case "boolDeviceProperty":
                return {
                    type: "bool",
                    value: false
                };
            case "listDeviceProperty":
                return {
                    type: "list",
                    value: [
                        { type: "string", value: "device-item1" },
                        { type: "uint", value: 200 }
                    ]
                };
            case "mapDeviceProperty":
                return {
                    type: "map",
                    value: {
                        deviceKey1: { type: "string", value: "device-value1" },
                        deviceKey2: { type: "uint", value: 200 }
                    }
                };
            case "bytesDeviceProperty":
                return {
                    type: "bytes",
                    value: [87, 111, 114, 108, 100] // "World" in ASCII
                };
            case "timestampDeviceProperty":
                return {
                    type: "timestamp",
                    value: Date.now() - 86400000 // Yesterday
                };
            case "functionDeviceProperty":
                return {
                    type: "function",
                    value: ["deviceFunction", { type: "string", value: "device-arg" }]
                };
            case "nullDeviceProperty":
                return {
                    type: "null",
                    value: null
                };
            default:
                console.error(`Device property not defined: ${name}`);
                return {
                    type: "string",
                    value: `Unknown device property: ${name}`
                };
        }
    }
}
/**
 * Helper function to run a test with a specific expression and context
 */
async function runTest(expression, context, testName) {
    const input = {
        variables: {
            map: {
                user: {
                    type: "map",
                    value: {
                        stringVal: { type: "string", value: "test-string-value" },
                        intVal: { type: "int", value: -42 },
                        uintVal: { type: "uint", value: 42 },
                        floatVal: { type: "float", value: 3.14159 },
                        boolVal: { type: "bool", value: true },
                        listVal: {
                            type: "list",
                            value: [
                                { type: "string", value: "item1" },
                                { type: "uint", value: 2 }
                            ]
                        },
                        mapVal: {
                            type: "map",
                            value: {
                                key1: { type: "string", value: "value1" },
                                key2: { type: "uint", value: 2 }
                            }
                        },
                        bytesVal: { type: "bytes", value: [72, 101, 108, 108, 111] },
                        timestampVal: { type: "timestamp", value: Date.now() },
                        functionVal: { type: "function", value: ["testFunction", { type: "string", value: "arg" }] },
                        nullVal: { type: "Null", value: null }
                    }
                }
            }
        },
        computed: {
            stringProperty: [{ type: "string", value: "arg" }],
            intProperty: [{ type: "string", value: "arg" }],
            uintProperty: [{ type: "string", value: "arg" }],
            floatProperty: [{ type: "string", value: "arg" }],
            boolProperty: [{ type: "string", value: "arg" }],
            listProperty: [{ type: "string", value: "arg" }],
            mapProperty: [{ type: "string", value: "arg" }],
            bytesProperty: [{ type: "string", value: "arg" }],
            timestampProperty: [{ type: "string", value: "arg" }],
            functionProperty: [{ type: "string", value: "arg" }],
            nullProperty: [{ type: "string", value: "arg" }]
        },
        device: {
            stringDeviceProperty: [{ type: "string", value: "arg" }],
            intDeviceProperty: [{ type: "string", value: "arg" }],
            uintDeviceProperty: [{ type: "string", value: "arg" }],
            floatDeviceProperty: [{ type: "string", value: "arg" }],
            boolDeviceProperty: [{ type: "string", value: "arg" }],
            listDeviceProperty: [{ type: "string", value: "arg" }],
            mapDeviceProperty: [{ type: "string", value: "arg" }],
            bytesDeviceProperty: [{ type: "string", value: "arg" }],
            timestampDeviceProperty: [{ type: "string", value: "arg" }],
            functionDeviceProperty: [{ type: "string", value: "arg" }],
            nullDeviceProperty: [{ type: "string", value: "arg" }]
        },
        expression: expression
    };
    console.log(`Running test: ${testName}`);
    const result = await evaluateWithContext(input, context);
    console.log(`Test result for ${testName}:`, result);
    // Check if the result matches the expected format
    assert.strictEqual(result, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}", `Test failed: ${testName}`);
    console.log(`✅ Test passed: ${testName}`);
}
// Jest test suite
describe('Superscript Expression Evaluation', () => {
    const context = new TestHostContext();
    // Test computed properties
    test('Computed String Type', async () => {
        await runTest('computed.stringProperty("arg") == user.stringVal', context, "Computed String Type");
    });
    test('Computed Int Type', async () => {
        await runTest('computed.intProperty("arg") == user.intVal', context, "Computed Int Type");
    });
    test('Computed UInt Type', async () => {
        await runTest('computed.uintProperty("arg") == user.uintVal', context, "Computed UInt Type");
    });
    test('Computed Float Type', async () => {
        await runTest('computed.floatProperty("arg") == user.floatVal', context, "Computed Float Type");
    });
    test('Computed Bool Type', async () => {
        await runTest('computed.boolProperty("arg") == user.boolVal', context, "Computed Bool Type");
    });
    // Test device properties
    test('Device String Type', async () => {
        await runTest('device.stringDeviceProperty("arg") == "device-string-value"', context, "Device String Type");
    });
    test('Device Int Type', async () => {
        await runTest('device.intDeviceProperty("arg") == -100', context, "Device Int Type");
    });
    test('Device UInt Type', async () => {
        await runTest('device.uintDeviceProperty("arg") == 100', context, "Device UInt Type");
    });
    test('Device Float Type', async () => {
        await runTest('device.floatDeviceProperty("arg") == 2.71828', context, "Device Float Type");
    });
    test('Device Bool Type', async () => {
        await runTest('device.boolDeviceProperty("arg") == false', context, "Device Bool Type");
    });
    // Test complex types
    test('Map Has Key', async () => {
        await runTest('has(user.mapVal, "key1")', context, "Map Has Key");
    });
    test('List Size', async () => {
        await runTest('size(user.listVal) == 2', context, "List Size");
    });
    // Test null handling
    test('Null Value', async () => {
        await runTest('user.nullVal == null', context, "Null Value");
    });
    test('Computed Null Property', async () => {
        await runTest('computed.nullProperty("arg") == null', context, "Computed Null Property");
    });
    test('Device Null Property', async () => {
        await runTest('device.nullDeviceProperty("arg") == null', context, "Device Null Property");
    });
});
// For direct execution (not through Jest)
if (typeof require !== 'undefined' && require.main === module) {
    const context = new TestHostContext();
    async function runAllTests() {
        try {
            console.log("Starting Superscript tests...");
            // Test all value types with computed properties
            await runTest('computed.stringProperty("arg") == user.stringVal', context, "Computed String Type");
            await runTest('computed.intProperty("arg") == user.intVal', context, "Computed Int Type");
            await runTest('computed.uintProperty("arg") == user.uintVal', context, "Computed UInt Type");
            await runTest('computed.floatProperty("arg") == user.floatVal', context, "Computed Float Type");
            await runTest('computed.boolProperty("arg") == user.boolVal', context, "Computed Bool Type");
            // Test all value types with device properties
            await runTest('device.stringDeviceProperty("arg") == "device-string-value"', context, "Device String Type");
            await runTest('device.intDeviceProperty("arg") == -100', context, "Device Int Type");
            await runTest('device.uintDeviceProperty("arg") == 100', context, "Device UInt Type");
            await runTest('device.floatDeviceProperty("arg") == 2.71828', context, "Device Float Type");
            await runTest('device.boolDeviceProperty("arg") == false', context, "Device Bool Type");
            // Test complex types
            await runTest('has(user.mapVal, "key1")', context, "Map Has Key");
            await runTest('size(user.listVal) == 2', context, "List Size");
            // Test null handling
            await runTest('user.nullVal == null', context, "Null Value");
            await runTest('computed.nullProperty("arg") == null', context, "Computed Null Property");
            await runTest('device.nullDeviceProperty("arg") == null', context, "Device Null Property");
            console.log("✅ All tests completed successfully!");
        }
        catch (error) {
            console.error("❌ Tests failed:", error);
            // Don't call process.exit() here as it interferes with Jest
        }
    }
    runAllTests().catch(error => {
        console.error("Test execution failed:", error);
        // Don't call process.exit() here as it interferes with Jest
    });
}
