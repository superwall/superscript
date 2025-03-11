const path = require('path');
const webpack = require('webpack');

module.exports = function override(config, env) {
    // Add WASM support
    config.experiments = {
        ...config.experiments,
        asyncWebAssembly: true,
        syncWebAssembly: true
    };

    // Add .wasm to the list of extensions
    config.resolve.extensions.push('.wasm');

    // Add rule for WASM files
    config.module.rules.push({
        test: /\.wasm$/,
        type: 'webassembly/async',
    });

    // Disable webpack's default handling of WASM
    config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        crypto: false,
    };

    return config;
};