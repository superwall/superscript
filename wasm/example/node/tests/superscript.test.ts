import pkg from '@superwall/superscript';
const { evaluateWithContext } = pkg;
import type { ExecutionContext, PassableValue } from '@superwall/superscript';
import { SuperscriptHostContext } from '@superwall/superscript';
import assert from 'assert';

/**
 * Test host context implementation that handles both computed and device properties
 * for all possible value types.
 */
class TestHostContext implements SuperscriptHostContext {
  computed_property(name: string, args: [PassableValue]): PassableValue {
    console.log(`computed_property called with name: ${name}, args:`, JSON.stringify(args));
    
    // Handle different property types based on the name
    switch (name) {
      case "stringProperty":
        return {
          type: "string" as const,
          value: "test-string-value"
        };
      
      case "intProperty":
        return {
          type: "int" as const,
          value: -42
        };
      
      case "uintProperty":
        return {
          type: "uint" as const,
          value: 42
        };
      
      case "floatProperty":
        return {
          type: "float" as const,
          value: 3.14159
        };
      
      case "boolProperty":
        return {
          type: "bool" as const,
          value: true
        };
      
      case "listProperty":
        return {
          type: "list" as const,
          value: [
            { type: "string" as const, value: "item1" },
            { type: "uint" as const, value: 2 }
          ]
        };
      
      case "mapProperty":
        return {
          type: "map" as const,
          value: {
            key1: { type: "string" as const, value: "value1" },
            key2: { type: "uint" as const, value: 2 }
          }
        };
      
      case "bytesProperty":
        return {
          type: "bytes" as const,
          value: [72, 101, 108, 108, 111] // "Hello" in ASCII
        };
      
      case "timestampProperty":
        return {
          type: "timestamp" as const,
          value: Date.now()
        };
      
      case "functionProperty":
        return {
          type: "function" as const,
          value: ["testFunction", { type: "string" as const, value: "arg" }]
        };
      
      case "nullProperty":
        return {
          type: "null" as const,
          value: null
        };
      
      default:
        console.error(`Computed property not defined: ${name}`);
        return {
          type: "string" as const,
          value: `Unknown computed property: ${name}`
        };
    }
  }

  device_property(name: string, args: [PassableValue]): PassableValue {
    console.log(`device_property called with name: ${name}, args:`, JSON.stringify(args));
    
    // Handle different property types based on the name
    switch (name) {
      case "stringDeviceProperty":
        return {
          type: "string" as const,
          value: "device-string-value"
        };
      
      case "intDeviceProperty":
        return {
          type: "int" as const,
          value: -100
        };
      
      case "uintDeviceProperty":
        return {
          type: "uint" as const,
          value: 100
        };
      
      case "floatDeviceProperty":
        return {
          type: "float" as const,
          value: 2.71828
        };
      
      case "boolDeviceProperty":
        return {
          type: "bool" as const,
          value: false
        };
      
      case "listDeviceProperty":
        return {
          type: "list" as const,
          value: [
            { type: "string" as const, value: "device-item1" },
            { type: "uint" as const, value: 200 }
          ]
        };
      
      case "mapDeviceProperty":
        return {
          type: "map" as const,
          value: {
            deviceKey1: { type: "string" as const, value: "device-value1" },
            deviceKey2: { type: "uint" as const, value: 200 }
          }
        };
      
      case "bytesDeviceProperty":
        return {
          type: "bytes" as const,
          value: [87, 111, 114, 108, 100] // "World" in ASCII
        };
      
      case "timestampDeviceProperty":
        return {
          type: "timestamp" as const,
          value: Date.now() - 86400000 // Yesterday
        };
      
      case "functionDeviceProperty":
        return {
          type: "function" as const,
          value: ["deviceFunction", { type: "string" as const, value: "device-arg" }]
        };
      
      case "nullDeviceProperty":
        return {
          type: "null" as const,
          value: null
        };
      
      default:
        console.error(`Device property not defined: ${name}`);
        return {
          type: "string" as const,
          value: `Unknown device property: ${name}`
        };
    }
  }
}

/**
 * Helper function to run a test with a specific expression and context
 */
async function runTest(expression: string, context: SuperscriptHostContext, testName: string): Promise<void> {
  const input: ExecutionContext = {
    variables: {
      map: {
        user: {
          type: "map" as const,
          value: {
            stringVal: { type: "string" as const, value: "test-string-value" },
            intVal: { type: "int" as const, value: -42 },
            uintVal: { type: "uint" as const, value: 42 },
            floatVal: { type: "float" as const, value: 3.14159 },
            boolVal: { type: "bool" as const, value: true },
            listVal: { 
              type: "list" as const, 
              value: [
                { type: "string" as const, value: "item1" },
                { type: "uint" as const, value: 2 }
              ] 
            },
            mapVal: { 
              type: "map" as const, 
              value: {
                key1: { type: "string" as const, value: "value1" },
                key2: { type: "uint" as const, value: 2 }
              } 
            },
            bytesVal: { type: "bytes" as const, value: [72, 101, 108, 108, 111] },
            timestampVal: { type: "timestamp" as const, value: Date.now() },
            functionVal: { type: "function" as const, value: ["testFunction", { type: "string" as const, value: "arg" }] },
            nullVal: { type: "Null" as const, value: null }
          }
        }
      }
    },
    computed: {
      stringProperty: [{ type: "string" as const, value: "arg" }],
      intProperty: [{ type: "string" as const, value: "arg" }],
      uintProperty: [{ type: "string" as const, value: "arg" }],
      floatProperty: [{ type: "string" as const, value: "arg" }],
      boolProperty: [{ type: "string" as const, value: "arg" }],
      listProperty: [{ type: "string" as const, value: "arg" }],
      mapProperty: [{ type: "string" as const, value: "arg" }],
      bytesProperty: [{ type: "string" as const, value: "arg" }],
      timestampProperty: [{ type: "string" as const, value: "arg" }],
      functionProperty: [{ type: "string" as const, value: "arg" }],
      nullProperty: [{ type: "string" as const, value: "arg" }]
    },
    device: {
      stringDeviceProperty: [{ type: "string" as const, value: "arg" }],
      intDeviceProperty: [{ type: "string" as const, value: "arg" }],
      uintDeviceProperty: [{ type: "string" as const, value: "arg" }],
      floatDeviceProperty: [{ type: "string" as const, value: "arg" }],
      boolDeviceProperty: [{ type: "string" as const, value: "arg" }],
      listDeviceProperty: [{ type: "string" as const, value: "arg" }],
      mapDeviceProperty: [{ type: "string" as const, value: "arg" }],
      bytesDeviceProperty: [{ type: "string" as const, value: "arg" }],
      timestampDeviceProperty: [{ type: "string" as const, value: "arg" }],
      functionDeviceProperty: [{ type: "string" as const, value: "arg" }],
      nullDeviceProperty: [{ type: "string" as const, value: "arg" }]
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
    } catch (error) {
      console.error("❌ Tests failed:", error);
      // Don't call process.exit() here as it interferes with Jest
    }
  }
  
  runAllTests().catch(error => {
    console.error("Test execution failed:", error);
    // Don't call process.exit() here as it interferes with Jest
  });
} 