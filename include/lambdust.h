/**
 * @file lambdust.h
 * @brief C/C++ Header for Lambdust Scheme Interpreter
 * 
 * This header provides a C-compatible interface for embedding the Lambdust
 * Scheme interpreter in C and C++ applications. It declares all the FFI
 * functions available for context management, code evaluation, function
 * registration, and memory management.
 * 
 * @version 1.0.0
 * @author Lambdust Development Team
 */

#ifndef LAMBDUST_H
#define LAMBDUST_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

/**
 * @brief Error codes returned by Lambdust FFI functions
 * 
 * These error codes provide detailed information about the success or failure
 * of Lambdust operations. Always check return values for error conditions.
 */
typedef enum {
    /** Operation completed successfully */
    LAMBDUST_SUCCESS = 0,
    /** General error occurred */
    LAMBDUST_ERROR = 1,
    /** Invalid argument provided */
    LAMBDUST_INVALID_ARGUMENT = 2,
    /** Null pointer encountered */
    LAMBDUST_NULL_POINTER = 3,
    /** Memory allocation failure */
    LAMBDUST_MEMORY_ERROR = 4,
    /** Scheme code evaluation error */
    LAMBDUST_EVALUATION_ERROR = 5,
    /** Type error in Scheme operation */
    LAMBDUST_TYPE_ERROR = 6,
    /** Wrong number of arguments (arity error) */
    LAMBDUST_ARITY_ERROR = 7,
    /** Runtime error during execution */
    LAMBDUST_RUNTIME_ERROR = 8,
    /** Panic occurred (should be rare) */
    LAMBDUST_PANIC_ERROR = 9
} LambdustErrorCode;

/**
 * @brief Opaque handle to a Lambdust interpreter context
 * 
 * This structure represents a Lambdust interpreter instance. It should be
 * treated as opaque - never access its members directly. Use the provided
 * functions to interact with the context.
 */
typedef struct LambdustContext LambdustContext;

/**
 * @brief Signature for host functions callable from Scheme
 * 
 * Host functions registered with Lambdust must conform to this signature.
 * They receive arguments as an array of C strings and return a result
 * string via an output parameter.
 * 
 * @param argc Number of arguments provided
 * @param argv Array of argument strings (null-terminated)
 * @param result Output parameter for result string (caller must allocate)
 * @return Error code indicating success or failure
 * 
 * @note The result string, if allocated, must be freed by the caller using
 *       the standard C library free() function or lambdust_free_string().
 * 
 * @example
 * @code
 * int my_host_function(int argc, const char* const* argv, char** result) {
 *     if (argc != 2) {
 *         return LAMBDUST_ARITY_ERROR;
 *     }
 *     
 *     int a = atoi(argv[0]);
 *     int b = atoi(argv[1]);
 *     int sum = a + b;
 *     
 *     *result = malloc(32);
 *     snprintf(*result, 32, "%d", sum);
 *     
 *     return LAMBDUST_SUCCESS;
 * }
 * @endcode
 */
typedef int (*LambdustHostFunction)(
    int argc,
    const char* const* argv,
    char** result
);

/**
 * @brief Create a new Lambdust interpreter context
 * 
 * Creates and initializes a new Lambdust interpreter context with default
 * settings. The context includes a complete Scheme environment with all
 * built-in functions and special forms.
 * 
 * @return Pointer to new context, or NULL on failure
 * 
 * @note The returned context must be destroyed with lambdust_destroy_context()
 *       to prevent memory leaks.
 * 
 * @example
 * @code
 * LambdustContext* ctx = lambdust_create_context();
 * if (ctx == NULL) {
 *     fprintf(stderr, "Failed to create Lambdust context\n");
 *     return -1;
 * }
 * @endcode
 */
LambdustContext* lambdust_create_context(void);

/**
 * @brief Destroy a Lambdust context and free associated memory
 * 
 * Properly destroys a Lambdust context and releases all associated resources.
 * After calling this function, the context pointer should not be used.
 * 
 * @param context Context to destroy (may be NULL)
 * 
 * @note This function is safe to call with a NULL pointer.
 * 
 * @example
 * @code
 * lambdust_destroy_context(ctx);
 * ctx = NULL; // Good practice to avoid accidental reuse
 * @endcode
 */
void lambdust_destroy_context(LambdustContext* context);

/**
 * @brief Evaluate Scheme code and return the result
 * 
 * Evaluates the provided Scheme code string and returns the result as a
 * newly allocated C string. The result string should be freed using
 * lambdust_free_string() when no longer needed.
 * 
 * @param context Valid Lambdust context
 * @param code Null-terminated Scheme code string
 * @param result Output parameter for result string (allocated by function)
 * @return Error code indicating success or failure
 * 
 * @note The result string is allocated by this function and must be freed
 *       by the caller using lambdust_free_string().
 * 
 * @example
 * @code
 * char* result = NULL;
 * int error = lambdust_eval(ctx, "(+ 1 2 3)", &result);
 * if (error == LAMBDUST_SUCCESS) {
 *     printf("Result: %s\n", result);
 *     lambdust_free_string(result);
 * } else {
 *     fprintf(stderr, "Evaluation failed with error %d\n", error);
 * }
 * @endcode
 */
int lambdust_eval(
    LambdustContext* context,
    const char* code,
    char** result
);

/**
 * @brief Register a host function that can be called from Scheme
 * 
 * Registers a C function that can be called from Scheme code. The function
 * will be available in the Scheme environment under the specified name.
 * 
 * @param context Valid Lambdust context
 * @param name Function name as it will appear in Scheme (null-terminated)
 * @param func Host function implementation
 * @return Error code indicating success or failure
 * 
 * @example
 * @code
 * int error = lambdust_register_function(ctx, "host-add", my_add_function);
 * if (error == LAMBDUST_SUCCESS) {
 *     // Function can now be called from Scheme as (host-add "10" "20")
 * }
 * @endcode
 */
int lambdust_register_function(
    LambdustContext* context,
    const char* name,
    LambdustHostFunction func
);

/**
 * @brief Get the last error message from a context
 * 
 * Retrieves a human-readable description of the last error that occurred
 * in the given context. The returned string is owned by the context and
 * should not be freed by the caller.
 * 
 * @param context Valid Lambdust context
 * @return Error message string, or NULL if no error occurred
 * 
 * @note The returned string is owned by the context and becomes invalid
 *       when the context is destroyed or when another operation occurs.
 * 
 * @example
 * @code
 * char* result = NULL;
 * int error = lambdust_eval(ctx, "(invalid-syntax", &result);
 * if (error != LAMBDUST_SUCCESS) {
 *     const char* err_msg = lambdust_get_last_error(ctx);
 *     if (err_msg) {
 *         fprintf(stderr, "Error: %s\n", err_msg);
 *     }
 * }
 * @endcode
 */
const char* lambdust_get_last_error(LambdustContext* context);

/**
 * @brief Free a string allocated by Lambdust
 * 
 * Frees memory allocated by Lambdust functions such as lambdust_eval().
 * This function should be used instead of the standard free() for strings
 * returned by Lambdust to ensure proper memory management.
 * 
 * @param str_ptr String pointer to free (may be NULL)
 * 
 * @note This function is safe to call with a NULL pointer.
 * 
 * @example
 * @code
 * char* result = NULL;
 * if (lambdust_eval(ctx, "(+ 1 2)", &result) == LAMBDUST_SUCCESS) {
 *     printf("Result: %s\n", result);
 *     lambdust_free_string(result); // Proper cleanup
 * }
 * @endcode
 */
void lambdust_free_string(char* str_ptr);

/**
 * @brief Call a Scheme function from C
 * 
 * Calls a function defined in the Scheme environment with the provided
 * arguments. This allows C code to invoke Scheme functions and retrieve
 * their results.
 * 
 * @param context Valid Lambdust context
 * @param function_name Name of the Scheme function to call
 * @param argc Number of arguments to pass
 * @param argv Array of argument strings (null-terminated)
 * @param result Output parameter for result string (allocated by function)
 * @return Error code indicating success or failure
 * 
 * @note The result string is allocated by this function and must be freed
 *       by the caller using lambdust_free_string().
 * 
 * @example
 * @code
 * // First define a function in Scheme
 * lambdust_eval(ctx, "(define (greet name) (string-append \"Hello, \" name \"!\"))", NULL);
 * 
 * // Then call it from C
 * const char* args[] = {"Alice"};
 * char* result = NULL;
 * int error = lambdust_call_function(ctx, "greet", 1, args, &result);
 * if (error == LAMBDUST_SUCCESS) {
 *     printf("Greeting: %s\n", result); // "Hello, Alice!"
 *     lambdust_free_string(result);
 * }
 * @endcode
 */
int lambdust_call_function(
    LambdustContext* context,
    const char* function_name,
    int argc,
    const char* const* argv,
    char** result
);

/**
 * @brief Get version information for Lambdust
 * 
 * Returns a string containing version information for the Lambdust library.
 * The returned string is statically allocated and should not be freed.
 * 
 * @return Version string (e.g., "Lambdust 1.0.0")
 * 
 * @example
 * @code
 * printf("Using %s\n", lambdust_get_version());
 * @endcode
 */
const char* lambdust_get_version(void);

/**
 * @brief Check if Lambdust library is properly initialized
 * 
 * Performs a basic sanity check to ensure the Lambdust library is working
 * correctly. This can be useful for debugging integration issues.
 * 
 * @return 1 if library is working correctly, 0 otherwise
 * 
 * @example
 * @code
 * if (!lambdust_check_library()) {
 *     fprintf(stderr, "Lambdust library is not functioning correctly\n");
 *     return -1;
 * }
 * @endcode
 */
int lambdust_check_library(void);

#ifdef __cplusplus
}
#endif

/*
 * =============================================================================
 * USAGE EXAMPLES
 * =============================================================================
 */

#ifdef LAMBDUST_INCLUDE_EXAMPLES

/**
 * @example basic_usage.c
 * 
 * Basic usage example showing context creation, evaluation, and cleanup:
 * 
 * @code
 * #include "lambdust.h"
 * #include <stdio.h>
 * #include <stdlib.h>
 * 
 * int main() {
 *     // Create context
 *     LambdustContext* ctx = lambdust_create_context();
 *     if (!ctx) {
 *         fprintf(stderr, "Failed to create context\n");
 *         return 1;
 *     }
 *     
 *     // Evaluate some Scheme code
 *     char* result = NULL;
 *     int error = lambdust_eval(ctx, "(+ (* 2 3) (* 4 5))", &result);
 *     
 *     if (error == LAMBDUST_SUCCESS) {
 *         printf("Result: %s\n", result);
 *         lambdust_free_string(result);
 *     } else {
 *         const char* err_msg = lambdust_get_last_error(ctx);
 *         fprintf(stderr, "Error: %s\n", err_msg ? err_msg : "Unknown error");
 *     }
 *     
 *     // Cleanup
 *     lambdust_destroy_context(ctx);
 *     return 0;
 * }
 * @endcode
 */

/**
 * @example host_functions.c
 * 
 * Example showing how to register and use host functions:
 * 
 * @code
 * #include "lambdust.h"
 * #include <stdio.h>
 * #include <stdlib.h>
 * #include <string.h>
 * 
 * // Host function that concatenates two strings
 * int host_concat(int argc, const char* const* argv, char** result) {
 *     if (argc != 2) {
 *         return LAMBDUST_ARITY_ERROR;
 *     }
 *     
 *     size_t len1 = strlen(argv[0]);
 *     size_t len2 = strlen(argv[1]);
 *     *result = malloc(len1 + len2 + 1);
 *     
 *     if (!*result) {
 *         return LAMBDUST_MEMORY_ERROR;
 *     }
 *     
 *     strcpy(*result, argv[0]);
 *     strcat(*result, argv[1]);
 *     
 *     return LAMBDUST_SUCCESS;
 * }
 * 
 * int main() {
 *     LambdustContext* ctx = lambdust_create_context();
 *     if (!ctx) return 1;
 *     
 *     // Register host function
 *     int error = lambdust_register_function(ctx, "string-concat", host_concat);
 *     if (error != LAMBDUST_SUCCESS) {
 *         fprintf(stderr, "Failed to register function\n");
 *         lambdust_destroy_context(ctx);
 *         return 1;
 *     }
 *     
 *     // Use the function from Scheme
 *     char* result = NULL;
 *     error = lambdust_eval(ctx, "(string-concat \"Hello, \" \"World!\")", &result);
 *     
 *     if (error == LAMBDUST_SUCCESS) {
 *         printf("Concatenated: %s\n", result);
 *         lambdust_free_string(result);
 *     }
 *     
 *     lambdust_destroy_context(ctx);
 *     return 0;
 * }
 * @endcode
 */

/**
 * @example cpp_wrapper.cpp
 * 
 * C++ wrapper class example:
 * 
 * @code
 * #include "lambdust.h"
 * #include <string>
 * #include <stdexcept>
 * #include <memory>
 * 
 * class LambdustInterpreter {
 * private:
 *     LambdustContext* ctx_;
 *     
 * public:
 *     LambdustInterpreter() : ctx_(lambdust_create_context()) {
 *         if (!ctx_) {
 *             throw std::runtime_error("Failed to create Lambdust context");
 *         }
 *     }
 *     
 *     ~LambdustInterpreter() {
 *         if (ctx_) {
 *             lambdust_destroy_context(ctx_);
 *         }
 *     }
 *     
 *     // Non-copyable
 *     LambdustInterpreter(const LambdustInterpreter&) = delete;
 *     LambdustInterpreter& operator=(const LambdustInterpreter&) = delete;
 *     
 *     std::string eval(const std::string& code) {
 *         char* result = nullptr;
 *         int error = lambdust_eval(ctx_, code.c_str(), &result);
 *         
 *         if (error != LAMBDUST_SUCCESS) {
 *             const char* err_msg = lambdust_get_last_error(ctx_);
 *             throw std::runtime_error(err_msg ? err_msg : "Evaluation failed");
 *         }
 *         
 *         std::string str_result(result);
 *         lambdust_free_string(result);
 *         return str_result;
 *     }
 *     
 *     void registerFunction(const std::string& name, LambdustHostFunction func) {
 *         int error = lambdust_register_function(ctx_, name.c_str(), func);
 *         if (error != LAMBDUST_SUCCESS) {
 *             throw std::runtime_error("Failed to register function");
 *         }
 *     }
 * };
 * 
 * int main() {
 *     try {
 *         LambdustInterpreter interp;
 *         std::string result = interp.eval("(+ 1 2 3)");
 *         std::cout << "Result: " << result << std::endl;
 *     } catch (const std::exception& e) {
 *         std::cerr << "Error: " << e.what() << std::endl;
 *         return 1;
 *     }
 *     return 0;
 * }
 * @endcode
 */

#endif /* LAMBDUST_INCLUDE_EXAMPLES */

#endif /* LAMBDUST_H */