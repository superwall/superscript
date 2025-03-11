import React, {useState, useCallback, useEffect, useRef} from 'react';
import {JsonEditor} from 'json-edit-react';
import * as wasm from '@superwall/superscript';
import Split from "react-split";
import Editor from "@monaco-editor/react";

const initialJson = {
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
        someComputedValue: [{
            type: "string",
            value: "event_name"
        }]
    },
    device: {
        daysSinceEvent: [{
            type: "string",
            value: "event_name"
        }]
    },
    expression: 'device.daysSinceEvent("test") == user.some_value'
};

const defaultPlatformCode = `/**
 * An example of a WasmHostContext implementation.
 * This contract allows the expression evaluator to call the host environment (your JS/TS)
 * and compute the dynamic properties, i.e. \`platform.daysSinceEvent("event_name")\`.
 * 
 * @method computed_property
 * @param {string} name - The name of the computed property.
 * @param {string} args - The JSON serialized list of arguments for the computed property.
 * @returns {string} - A JSON string representing the computed property value.
 * 
 * @method device_property
 * @param {string} name - The name of the device property.
 * @param {string} args - The JSON serialized list of arguments for the device property.
 * @returns {string} - A JSON string representing the device property value.
 */
class WasmHostContext {
  
  computed_property(name, args) {
    console.log("computed_property called with name:", name, "args:", args);
    
    // Important: args must be a string that can be parsed as JSON
    // The Rust side expects a string that it will parse
    if (name === "someComputedValue") {
      let toReturn = {
        type: "uint",
        value: 7
      };
      console.log("Computed property will return", toReturn);
      return toReturn;
    }
    
    console.error("Computed property not defined");
    return {
      type: "string",
      value: "Computed property " + name
    };
  }

  device_property(name, args) {
    console.log("device_property called with name:", name, "args:", args);
    
    // Important: args must be a string that can be parsed as JSON
    // The Rust side expects a string that it will parse
    if (name === "daysSinceEvent") {
      let toReturn = {
        type: "uint",
        value: 7
      };
      console.log("Device property will return", toReturn);
      return toReturn;
    }
    
    console.error("Device property not defined");
    return {
      type: "string",
      value: "Device property " + name
    };
  }
}
`;

const SuperscriptParserComponent = () => {
    // Add CSS animation for the error toast
    useEffect(() => {
        // Create a style element
        const styleEl = document.createElement('style');
        // Define the animation
        styleEl.innerHTML = `
            @keyframes slideIn {
                0% {
                    transform: translateY(-20px);
                    opacity: 0;
                }
                100% {
                    transform: translateY(0);
                    opacity: 1;
                }
            }
            
            @keyframes shake {
                0%, 100% {
                    transform: translateX(0);
                }
                10%, 30%, 50%, 70%, 90% {
                    transform: translateX(-5px);
                }
                20%, 40%, 60%, 80% {
                    transform: translateX(5px);
                }
            }
        `;
        // Append the style element to the head
        document.head.appendChild(styleEl);
        
        // Clean up
        return () => {
            document.head.removeChild(styleEl);
        };
    }, []);

    const [json, setJson] = useState(initialJson);
    const [platformCode, setPlatformCode] = useState(defaultPlatformCode);
    const [result, setResult] = useState(null);
    const [error, setError] = useState(null);
    const editorRef = useRef(null);
    const handleEditorDidMount = (editor, monaco) => {
        editorRef.current = editor;
    };

    useEffect(() => {
        const initWasm = async () => {
            try {
                // Initialize the WASM module and log when it's ready
                console.log("Initializing WASM module...");
                console.log("WASM module before initialization:", wasm);
                await wasm;
                console.log("WASM module after initialization:", wasm);
                console.log("WASM module initialized successfully");
                
                // Check if the evaluate_with_context function exists
            } catch (err) {
                console.error("WASM initialization error:", err);
                setError('Failed to initialize WASM module: ' + err.message);
            }
        };
        initWasm();
    }, []);

    const evaluateExpression = async () => {
        try {
            const code = editorRef.current.getValue();
            console.log("Evaluating with code:", code);
            
            // Here we need to evaluate the code as a class definition to get the WasmHostContext instance.
            let wasmHostContext;
            try {
                const WasmHostContextClass = new Function(`${code}; return WasmHostContext;`)();
                wasmHostContext = new WasmHostContextClass();
            } catch (classError) {
                console.warn("Failed to evaluate as class:", classError);
                throw new Error(`Error parsing platform code: ${classError.message}`);
            }

            console.log("Created host context:", wasmHostContext);

            console.log("Will evaluate with context:", json, wasmHostContext);
            
            console.log("Calling WASM evaluate_with_context function...");
            try {
                let res = await wasm.evaluateWithContext(json, wasmHostContext);
                console.log("Result:", res);
                setResult(res);
                setError(null);
            } catch (wasmError) {
                throw new Error(`Error evaluating expression: ${wasmError.message}`);
            }
        } catch (err) {
            console.error("Evaluation error:", err);
            setError(err.message || "Unknown error occurred during evaluation");
            setResult(null);
        }
    };

    return (
        <div style={styles.container}>
            <div style={styles.toolbar}>
                <h1 style={styles.title}>Superscript Parser</h1>
                <button
                    style={styles.button}
                    onClick={evaluateExpression}
                >
                    Evaluate Expression
                </button>
            </div>
            {error && (
                <div style={styles.errorToast}>
                    <div style={styles.errorIcon}>⚠️</div>
                    <div style={styles.errorMessage}>{error}</div>
                    <button 
                        style={styles.errorCloseButton}
                        onClick={() => setError(null)}
                    >
                        ×
                    </button>
                </div>
            )}
            <Split
                style={styles.splitContainer}
                sizes={[50, 40, 10]}
                minSize={100}
                expandToMin={false}
                gutterSize={10}
                gutterAlign="center"
                snapOffset={30}
                dragInterval={1}
                direction="horizontal"
                cursor="col-resize"
            >
                <div style={styles.pane}>
                    <h2 style={styles.paneTitle}>Platform Code</h2>
                    <Editor
                        height="90%"
                        defaultLanguage="typescript"
                        defaultValue={platformCode}
                        theme="vs-dark"
                        options={{
                            lineNumbers: 'off',
                            minimap: { enabled: false },
                            fontSize: 14,
                        }}
                        onMount={handleEditorDidMount}
                    />                </div>
                <div style={styles.pane}>
                    <h2 style={styles.paneTitle}>JSON Editor</h2>
                    <JsonEditor
                        data={json}
                        setData={setJson}
                        onUpdate={({newData}) => {
                            setJson(newData);
                        }}
                        collapse={false}
                        enableClipboard={true}
                        showCollectionCount={true}
                    />
                </div>
                <div style={styles.pane}>
                    <h2 style={styles.paneTitle}>Result</h2>
                    {error && (
                        <div style={styles.resultError}>
                            <div style={styles.resultErrorTitle}>Error:</div>
                            <div style={styles.resultErrorMessage}>{error}</div>
                        </div>
                    )}
                    {result !== null && !error && (
                        <div style={styles.resultSuccess}>
                            <div style={styles.resultSuccessTitle}>Success:</div>
                            {typeof result === 'string' ? (
                                <div>
                                    <div style={styles.resultLabel}>Raw Result:</div>
                                    <pre style={styles.result}>{result}</pre>
                                    
                                    {(() => {
                                        try {
                                            const parsedJson = JSON.parse(result);
                                            return (
                                                <div>
                                                    <div style={styles.resultLabel}>Parsed JSON:</div>
                                                    <pre style={styles.resultJson}>
                                                        {JSON.stringify(parsedJson, null, 2)}
                                                    </pre>
                                                </div>
                                            );
                                        } catch (e) {
                                            return null;
                                        }
                                    })()}
                                </div>
                            ) : (
                                <pre style={styles.result}>{JSON.stringify(result, null, 2)}</pre>
                            )}
                        </div>
                    )}
                </div>
            </Split>
        </div>
    );
};

const styles = {
    container: {
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        backgroundColor: '#0A192F',
        color: '#E6F1FF',
    },
    toolbar: {
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        padding: '10px 20px',
        backgroundColor: '#172A45',
        borderBottom: '1px solid #2D3B4F',
    },
    title: {
        fontSize: '24px',
        fontWeight: 'bold',
        margin: 0,
        color: '#64FFDA',
    },
    errorToast: {
        display: 'flex',
        alignItems: 'center',
        backgroundColor: '#FF5555',
        color: 'white',
        padding: '10px 20px',
        borderRadius: '4px',
        margin: '10px 20px',
        boxShadow: '0 2px 10px rgba(0, 0, 0, 0.2)',
        animation: 'slideIn 0.3s ease-out, shake 0.5s ease-in-out 0.3s',
    },
    errorIcon: {
        marginRight: '10px',
        fontSize: '20px',
    },
    errorMessage: {
        flexGrow: 1,
        fontWeight: 'bold',
    },
    errorCloseButton: {
        background: 'none',
        border: 'none',
        color: 'white',
        fontSize: '20px',
        cursor: 'pointer',
        padding: '0 5px',
    },
    button: {
        backgroundColor: '#64FFDA',
        border: 'none',
        color: '#0A192F',
        padding: '10px 20px',
        textAlign: 'center',
        textDecoration: 'none',
        display: 'inline-block',
        fontSize: '16px',
        margin: '4px 2px',
        cursor: 'pointer',
        borderRadius: '4px',
        transition: 'background-color 0.3s ease',
    },
    splitContainer: {
        display: 'flex',
        flexGrow: 1,
    },
    pane: {
        display: 'flex',
        flexDirection: 'column',
        padding: '20px',
        overflow: 'auto',
        backgroundColor: '#1E2A3A',
    },
    paneTitle: {
        fontSize: '18px',
        fontWeight: 'bold',
        marginBottom: '15px',
        color: '#64FFDA',
    },
    textarea: {
        width: '100%',
        height: 'calc(100% - 40px)',
        padding: '10px',
        border: '1px solid #2D3B4F',
        borderRadius: '4px',
        resize: 'none',
        backgroundColor: '#2A3A4A',
        color: '#E6F1FF',
        fontSize: '14px',
        lineHeight: '1.5',
    },
    error: {
        color: '#FF6B6B',
        marginBottom: '10px',
        padding: '10px',
        backgroundColor: 'rgba(255, 107, 107, 0.1)',
        borderRadius: '4px',
    },
    resultError: {
        backgroundColor: 'rgba(255, 85, 85, 0.2)',
        border: '1px solid #FF5555',
        borderRadius: '4px',
        padding: '10px',
        margin: '10px 0',
        color: '#FF5555',
        fontFamily: 'monospace',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
    },
    resultErrorTitle: {
        fontWeight: 'bold',
        marginBottom: '5px',
        fontSize: '16px',
    },
    resultErrorMessage: {
        fontSize: '14px',
    },
    resultSuccess: {
        backgroundColor: 'rgba(100, 255, 218, 0.2)',
        border: '1px solid #64FFDA',
        borderRadius: '4px',
        padding: '10px',
        margin: '10px 0',
        maxHeight: 'calc(100vh - 200px)',
        overflow: 'auto',
    },
    resultSuccessTitle: {
        fontWeight: 'bold',
        marginBottom: '5px',
        fontSize: '16px',
        color: '#64FFDA',
    },
    resultLabel: {
        fontWeight: 'bold',
        marginTop: '10px',
        marginBottom: '5px',
        fontSize: '14px',
        color: '#E6F1FF',
    },
    result: {
        margin: 0,
        color: '#E6F1FF',
        fontFamily: 'monospace',
        fontSize: '14px',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        backgroundColor: 'rgba(42, 58, 74, 0.5)',
        padding: '8px',
        borderRadius: '4px',
        maxWidth: '100%',
        overflow: 'auto',
    },
    resultJson: {
        margin: 0,
        color: '#E6F1FF',
        fontFamily: 'monospace',
        fontSize: '14px',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-word',
        backgroundColor: 'rgba(42, 58, 74, 0.5)',
        padding: '8px',
        borderRadius: '4px',
        maxWidth: '100%',
        overflow: 'auto'
    },
};
export default SuperscriptParserComponent;