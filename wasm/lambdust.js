/**
 * Lambdust WebAssembly JavaScript bindings
 * 
 * This module provides a high-level JavaScript API for the Lambdust
 * Scheme interpreter running in WebAssembly.
 */

import * as wasm from './lambdust_bg.wasm';
import { WasmLambdustInterpreter, utils } from './lambdust_bg.js';

/**
 * Promise-based wrapper for the WebAssembly interpreter
 */
export class AsyncLambdustInterpreter {
  constructor() {
    this.interpreter = new WasmLambdustInterpreter();
  }
  
  /**
   * Evaluate Scheme code asynchronously
   */
  async eval(code) {
    return new Promise((resolve, reject) => {
      try {
        const result = this.interpreter.eval(code);
        if (result === null) {
          const error = this.interpreter.get_last_error();
          reject(new Error(error || 'Unknown evaluation error'));
        } else {
          resolve(result);
        }
      } catch (error) {
        reject(error);
      }
    });
  }
  
  /**
   * Evaluate Scheme code and return JavaScript value asynchronously
   */
  async evalJs(code) {
    return new Promise((resolve, reject) => {
      try {
        const result = this.interpreter.eval_js(code);
        resolve(result);
      } catch (error) {
        reject(error);
      }
    });
  }
  
  /**
   * Register a JavaScript function asynchronously
   */
  async registerFunction(name, func) {
    return new Promise((resolve, reject) => {
      try {
        this.interpreter.register_js_function(name, func);
        resolve();
      } catch (error) {
        reject(error);
      }
    });
  }
  
  /**
   * Load a Scheme program from URL
   */
  async loadFromUrl(url) {
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
      }
      const code = await response.text();
      return await this.eval(code);
    } catch (error) {
      throw new Error(`Failed to load from URL ${url}: ${error.message}`);
    }
  }
  
  /**
   * Destroy the interpreter and free resources
   */
  destroy() {
    if (this.interpreter) {
      this.interpreter.free();
      this.interpreter = null;
    }
  }
}

/**
 * High-level API for common Scheme operations
 */
export class SchemeAPI {
  constructor() {
    this.interpreter = new WasmLambdustInterpreter();
    this.procedures = new Set();
    this._initializeStandardLibrary();
  }
  
  /**
   * Define a variable in the global environment
   */
  define(name, value) {
    let schemeValue;
    if (typeof value === 'number') {
      schemeValue = value.toString();
    } else if (typeof value === 'string') {
      schemeValue = `"${value.replace(/"/g, '\\"')}"`;
    } else if (typeof value === 'boolean') {
      schemeValue = value ? '#t' : '#f';
    } else if (Array.isArray(value)) {
      const elements = value.map(v => this._valueToScheme(v)).join(' ');
      schemeValue = `'(${elements})`;
    } else {
      schemeValue = `"${String(value)}"`;
    }
    
    const result = this.interpreter.eval(`(define ${name} ${schemeValue})`);
    if (result === null) {
      throw new Error(this.interpreter.get_last_error() || 'Failed to define variable');
    }
  }
  
  /**
   * Call a Scheme function with arguments
   */
  call(functionName, ...args) {
    const schemeArgs = args.map(arg => this._valueToScheme(arg)).join(' ');
    const code = `(${functionName} ${schemeArgs})`;
    
    try {
      return this.interpreter.eval_js(code);
    } catch (error) {
      throw new Error(`Failed to call ${functionName}: ${error.message}`);
    }
  }
  
  /**
   * Load a standard library module
   */
  loadModule(moduleName) {
    const moduleMap = {
      'srfi-1': '(import (srfi 1))',
      'srfi-13': '(import (srfi 13))',
      'srfi-69': '(import (srfi 69))',
      // Add more modules as needed
    };
    
    const importCode = moduleMap[moduleName];
    if (!importCode) {
      throw new Error(`Unknown module: ${moduleName}`);
    }
    
    const result = this.interpreter.eval(importCode);
    if (result === null) {
      throw new Error(this.interpreter.get_last_error() || `Failed to load module ${moduleName}`);
    }
  }
  
  /**
   * Get available procedures
   */
  getProcedures() {
    return Array.from(this.procedures);
  }
  
  /**
   * Check if a procedure exists
   */
  hasProcedure(name) {
    return this.procedures.has(name);
  }
  
  /**
   * Convert JavaScript value to Scheme representation
   */
  _valueToScheme(value) {
    if (typeof value === 'number') {
      return value.toString();
    } else if (typeof value === 'string') {
      return `"${value.replace(/"/g, '\\"')}"`;
    } else if (typeof value === 'boolean') {
      return value ? '#t' : '#f';
    } else if (Array.isArray(value)) {
      const elements = value.map(v => this._valueToScheme(v)).join(' ');
      return `'(${elements})`;
    } else if (value === null || value === undefined) {
      return "'()";
    } else {
      return `"${String(value)}"`;
    }
  }
  
  /**
   * Initialize standard library procedures
   */
  _initializeStandardLibrary() {
    // Standard procedures
    const standardProcs = [
      // Arithmetic
      '+', '-', '*', '/', 'quotient', 'remainder', 'modulo',
      'abs', 'floor', 'ceiling', 'sqrt', 'expt', 'min', 'max',
      
      // Comparison
      '=', '<', '>', '<=', '>=', 'eq?', 'eqv?', 'equal?',
      
      // Type predicates
      'number?', 'integer?', 'real?', 'string?', 'symbol?', 'boolean?',
      'list?', 'vector?', 'procedure?', 'null?', 'pair?',
      
      // List operations
      'car', 'cdr', 'cons', 'list', 'append', 'reverse', 'length',
      'list-ref', 'list-set!', 'list->vector', 'list->string',
      
      // String operations
      'string=?', 'string<?', 'string>?', 'string<=?', 'string>=?',
      'string-length', 'string-ref', 'string-set!', 'make-string',
      'string->list', 'string->number', 'string-append',
      
      // Vector operations
      'vector', 'make-vector', 'vector-length', 'vector-ref', 'vector-set!',
      'vector->list', 'list->vector',
      
      // Control flow
      'if', 'cond', 'case', 'and', 'or', 'not',
      'lambda', 'define', 'let', 'let*', 'letrec',
      
      // Higher-order functions
      'apply', 'map', 'for-each', 'fold', 'fold-right', 'filter',
      
      // I/O
      'display', 'newline', 'write', 'read',
    ];
    
    standardProcs.forEach(proc => this.procedures.add(proc));
  }
}

/**
 * Performance monitoring utilities
 */
export class PerformanceMonitor {
  constructor(interpreter) {
    this.interpreter = interpreter;
    this.benchmarks = new Map();
  }
  
  /**
   * Benchmark Scheme code execution
   */
  async benchmark(name, code, iterations = 1000) {
    const times = [];
    
    for (let i = 0; i < iterations; i++) {
      const start = utils.now();
      
      try {
        await this.interpreter.eval(code);
        const end = utils.now();
        times.push(end - start);
      } catch (error) {
        utils.error(`Benchmark error in iteration ${i}: ${error.message}`);
        break;
      }
    }
    
    if (times.length === 0) {
      throw new Error('No successful benchmark runs');
    }
    
    const stats = {
      name,
      iterations: times.length,
      total: times.reduce((a, b) => a + b, 0),
      average: times.reduce((a, b) => a + b, 0) / times.length,
      min: Math.min(...times),
      max: Math.max(...times),
      median: times.sort((a, b) => a - b)[Math.floor(times.length / 2)]
    };
    
    this.benchmarks.set(name, stats);
    return stats;
  }
  
  /**
   * Get benchmark results
   */
  getResults(name) {
    return name ? this.benchmarks.get(name) : Object.fromEntries(this.benchmarks);
  }
  
  /**
   * Clear benchmark results
   */
  clear() {
    this.benchmarks.clear();
  }
}

/**
 * Error handling utilities
 */
export class ErrorHandler {
  constructor() {
    this.errorLog = [];
    this.maxLogSize = 100;
  }
  
  /**
   * Handle and log an error
   */
  handle(error, context = '') {
    const errorEntry = {
      timestamp: new Date().toISOString(),
      message: error.message || String(error),
      context,
      stack: error.stack
    };
    
    this.errorLog.push(errorEntry);
    
    // Keep log size manageable
    if (this.errorLog.length > this.maxLogSize) {
      this.errorLog.shift();
    }
    
    utils.error(`[${context}] ${errorEntry.message}`);
    return errorEntry;
  }
  
  /**
   * Get error log
   */
  getLog() {
    return [...this.errorLog];
  }
  
  /**
   * Clear error log
   */
  clearLog() {
    this.errorLog.length = 0;
  }
}

/**
 * Module loader for dynamic imports
 */
export class ModuleLoader {
  constructor(interpreter) {
    this.interpreter = interpreter;
    this.loadedModules = new Set();
  }
  
  /**
   * Load a module from URL
   */
  async loadModule(url, name) {
    if (this.loadedModules.has(name)) {
      return; // Already loaded
    }
    
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      
      const code = await response.text();
      const result = this.interpreter.eval(code);
      
      if (result === null) {
        throw new Error(this.interpreter.get_last_error() || 'Module evaluation failed');
      }
      
      this.loadedModules.add(name);
      utils.log(`Module ${name} loaded successfully`);
      
    } catch (error) {
      throw new Error(`Failed to load module ${name} from ${url}: ${error.message}`);
    }
  }
  
  /**
   * Check if module is loaded
   */
  isLoaded(name) {
    return this.loadedModules.has(name);
  }
  
  /**
   * Get list of loaded modules
   */
  getLoadedModules() {
    return Array.from(this.loadedModules);
  }
}

// Export the main interpreter and utilities
export { WasmLambdustInterpreter, utils };
export default WasmLambdustInterpreter;