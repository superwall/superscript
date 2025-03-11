import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';
import SuperscriptParserComponent from "./SuperscriptParser";

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <SuperscriptParserComponent />
  </React.StrictMode>
);