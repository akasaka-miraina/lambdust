/**
 * TypeScript definitions for Lambdust WebAssembly bindings
 * 
 * This file provides type definitions for the Lambdust Scheme interpreter
 * when used in JavaScript/TypeScript environments via WebAssembly.
 */

export interface LambdustError {
  message: string;
  type: 'SyntaxError' | 'RuntimeError' | 'TypeError' | 'ArityError';
}

export interface SchemeValue {
  type: 'number' | 'string' | 'boolean' | 'symbol' | 'list' | 'vector' | 'procedure' | 'void';
  value?: any;
}

export interface MemoryStats {
  totalAllocated: number;
  peakUsage: number;
  allocationCount: number;
}

/**
 * WebAssembly Lambdust Scheme interpreter
 */
export class WasmLambdustInterpreter {
  /**
   * Create a new interpreter instance
   */
  constructor();
  
  /**
   * Evaluate Scheme code and return the result as a string
   * @param code Scheme code to evaluate
   * @returns Result string or null if error
   */
  eval(code: string): string | null;
  
  /**
   * Evaluate Scheme code and return the result as a JavaScript value
   * @param code Scheme code to evaluate
   * @returns JavaScript representation of the result
   * @throws Error if evaluation fails
   */
  eval_js(code: string): any;
  
  /**
   * Get the last error message
   * @returns Error message or null if no error
   */
  get_last_error(): string | null;
  
  /**
   * Register a JavaScript function as a Scheme procedure
   * @param name Name of the procedure in Scheme
   * @param func JavaScript function to register
   */
  register_js_function(name: string, func: Function): void;
  
  /**
   * Load and evaluate a Scheme program from a string
   * @param program Scheme program code
   * @returns Result string
   * @throws Error if evaluation fails
   */
  load_program(program: string): string;
  
  /**
   * Reset the interpreter to its initial state
   */
  reset(): void;
  
  /**
   * Get interpreter version
   * @returns Version string
   */
  static version(): string;
  
  /**
   * Check if the interpreter is in a valid state
   * @returns True if healthy
   */
  is_healthy(): boolean;
  
  /**
   * Free the interpreter resources (should be called when done)
   */
  free(): void;
}

/**
 * Utility functions for WebAssembly environments
 */
export namespace utils {
  /**
   * Log a message to the browser console
   * @param message Message to log
   */
  export function log(message: string): void;
  
  /**
   * Log an error to the browser console
   * @param message Error message to log
   */
  export function error(message: string): void;
  
  /**
   * Get the current timestamp (useful for benchmarking)
   * @returns Current timestamp in milliseconds
   */
  export function now(): number;
  
  /**
   * Check if running in a browser environment
   * @returns True if in browser
   */
  export function is_browser(): boolean;
  
  /**
   * Check if running in Node.js environment
   * @returns True if in Node.js
   */
  export function is_nodejs(): boolean;
}

/**
 * Promise-based wrapper for the WebAssembly interpreter
 */
export class AsyncLambdustInterpreter {
  private interpreter: WasmLambdustInterpreter;
  
  constructor();
  
  /**
   * Evaluate Scheme code asynchronously
   * @param code Scheme code to evaluate
   * @returns Promise resolving to the result
   */
  eval(code: string): Promise<string>;
  
  /**
   * Evaluate Scheme code and return JavaScript value asynchronously
   * @param code Scheme code to evaluate
   * @returns Promise resolving to JavaScript value
   */
  evalJs(code: string): Promise<any>;
  
  /**
   * Register a JavaScript function asynchronously
   * @param name Function name in Scheme
   * @param func JavaScript function
   * @returns Promise resolving when registration is complete
   */
  registerFunction(name: string, func: Function): Promise<void>;
  
  /**
   * Load a Scheme program from URL
   * @param url URL to load program from
   * @returns Promise resolving to the result
   */
  loadFromUrl(url: string): Promise<string>;
  
  /**
   * Destroy the interpreter and free resources
   */
  destroy(): void;
}

/**
 * High-level API for common Scheme operations
 */
export class SchemeAPI {
  private interpreter: WasmLambdustInterpreter;
  
  constructor();
  
  /**
   * Define a variable in the global environment
   * @param name Variable name
   * @param value Variable value
   */
  define(name: string, value: any): void;
  
  /**
   * Call a Scheme function with arguments
   * @param functionName Function name
   * @param args Function arguments
   * @returns Function result
   */
  call(functionName: string, ...args: any[]): any;
  
  /**
   * Load a standard library module
   * @param moduleName Module name (e.g., 'srfi-1', 'srfi-13')
   */
  loadModule(moduleName: string): void;
  
  /**
   * Get available procedures
   * @returns Array of procedure names
   */
  getProcedures(): string[];
  
  /**
   * Check if a procedure exists
   * @param name Procedure name
   * @returns True if exists
   */
  hasProcedure(name: string): boolean;
}

/**
 * Initialize the Lambdust WebAssembly module
 * @param wasmPath Path to the .wasm file (optional)
 * @returns Promise resolving when initialization is complete
 */
export function init(wasmPath?: string): Promise<void>;

/**
 * Default export - the main interpreter class
 */
export default WasmLambdustInterpreter;