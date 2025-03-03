import React, {useState, useCallback, useEffect, useRef} from 'react';
import {JsonEditor} from 'json-edit-react';
import * as wasm from 'supercel-wasm';
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
    device: {
        daysSinceEvent: [{
            type: "string",
            value: "event_name"
        }]
    },
    computed: {
        daysSinceEvent: [{
            type: "string",
            value: "event_name"
        }]
    },
    expression: 'device.daysSinceEvent("test") == user.some_value'
};

const defaultPlatformCode = `/**
 * An example of a WasmHostContext implementation from @file src/lib.rs.
 * This contract allows the expression evaluator to call the host environment (your JS)
 * and compute the dynamic properties, i.e. \`platform.daysSinceEvent("event_name")\`.
 *
 * @param name - The name of the computed property or function being invoked.
 * @param args - JSON string of the arguments for the function.
 * @returns JSON-serialized string of the computed property value.
 * */
class WasmHostContext {
  computed_property(name, args) {
        console.log(\`computed_property called with name: \${name}, args: \${args}\`);
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
                value: \`Computed property \${name} with args \${args}\`
        });
  }

  device_property(name, args) {
      console.log(\`computed_property called with name: \${name}, args: \${args}\`);
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
          value: \`Computed property \${name} with args \${args}\`
      });
  }
}
`;

const CelParserComponent = () => {


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
                await wasm;
                await wasm.start();
            } catch (err) {
                setError('Failed to initialize WASM module: ' + err.message);
            }
        };
        initWasm();
    }, []);

    const handleJsonChange = useCallback((newJson) => {
        setJson(newJson);
    }, []);

    const handlePlatformCodeChange = (event) => {
        setPlatformCode(event.target.value);
    };

    const evaluateAll = () => {
        evaluateExpression()
    }
    const evaluateExpression = async () => {
        try {
            const code = editorRef.current.getValue();
            // Create a new WasmHostContext instance with the code from Editor
            const WasmHostContextClass = eval(`(${code})`);
            const wasmHostContext = new WasmHostContextClass();

            console.log("Will evaluate with context:", json, wasmHostContext)
            console.log("Wasm functions:", wasm.evaluate_with_context)
            let res = await wasm.evaluate_with_context(JSON.stringify(json), wasmHostContext)
            console.log("Result:", res);
            setResult(res);
            setError(null);
        } catch (err) {
            console.log("Error:", err.message)
            setError(err.message);
            setResult(null);
        }
    };

    return (
        <div style={styles.container}>
            <div style={styles.toolbar}>
                <h1 style={styles.title}>CEL Parser</h1>
                <button
                    style={styles.button}
                    onClick={evaluateExpression}
                >
                    Evaluate Expression
                </button>
            </div>
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
                        defaultLanguage="javascript"
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
                        theme="githubDark"
                        collapse={false}
                        enableClipboard={true}
                        showCollectionCount={true}
                    />
                </div>
                <div style={styles.pane}>
                    <h2 style={styles.paneTitle}>Result</h2>
                    {error && (
                        <div style={styles.error}>{error}</div>
                    )}
                    {result !== null && (
                        <pre style={styles.result}>{JSON.stringify(result, null, 2)}</pre>
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
    result: {
        backgroundColor: '#2A3A4A',
        padding: '15px',
        borderRadius: '4px',
        whiteSpace: 'pre-wrap',
        wordBreak: 'break-all',
        fontSize: '14px',
        lineHeight: '1.5',
        color: '#E6F1FF',
    },
};
export default CelParserComponent;