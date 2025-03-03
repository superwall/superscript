# Supercel WASM Module

This is (WASM) runner for Superscript expression language.
The evaluator can call host environment functions and compute dynamic properties.

## Getting Started

### Setup
- Install the prerequisites
- Run `rustup target add wasm32-unknown-unknown` to add the WASM target

### Building the Project

To build the project, you need to:

- Run `./build_wasm.sh`

**OR**

- Build the WASM project for the first time: `cargo build --lib --target wasm32-unknown-unknown`

Then use:
```bash
npm run build
```

This will generate targets in the `.target/` directory
* `./target/browser` for browser environments
* `./target/node` for Node.js environments


### Running the Project

For **browsers**:

- Open `./example/browser/` and run `npm install ../../target/browser && npm run start`

For **node**:
- Open `./example/` and run `node test_node.js`


