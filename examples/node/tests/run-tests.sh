#!/bin/bash

# Install dependencies
npm install

# Build the TypeScript files
npm run build

# Run the tests with Jest
npm test

# Or run directly without Jest
# node --experimental-vm-modules dist/superscript.test.js 