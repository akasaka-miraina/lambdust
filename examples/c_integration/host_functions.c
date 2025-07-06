/**
 * @file host_functions.c
 * @brief Host function integration example
 * 
 * This example demonstrates how to register C functions that can be
 * called from Scheme code, enabling bidirectional integration between
 * C and Scheme.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>
#include "lambdust.h"

// Global state for demonstration
static int counter = 0;

/**
 * @brief Host function: Print a message to stdout
 */
static int host_print(int argc, const char* const* argv, char** result) {
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    printf("Scheme says: %s\n", argv[0]);
    
    // Return void (empty string)
    *result = malloc(1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    (*result)[0] = '\0';
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Add two numbers
 */
static int host_add(int argc, const char* const* argv, char** result) {
    if (argc != 2) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    double a = atof(argv[0]);
    double b = atof(argv[1]);
    double sum = a + b;
    
    *result = malloc(32);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    // Return as integer if possible, otherwise as float
    if (sum == (int)sum) {
        snprintf(*result, 32, "%d", (int)sum);
    } else {
        snprintf(*result, 32, "%.6g", sum);
    }
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Calculate square root
 */
static int host_sqrt(int argc, const char* const* argv, char** result) {
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    double x = atof(argv[0]);
    if (x < 0) {
        return LAMBDUST_RUNTIME_ERROR;
    }
    
    double root = sqrt(x);
    
    *result = malloc(32);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    snprintf(*result, 32, "%.6g", root);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Get current time as string
 */
static int host_current_time(int argc, const char* const* argv, char** result) {
    if (argc != 0) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    time_t now = time(NULL);
    char* time_str = ctime(&now);
    
    // Remove newline from ctime result
    size_t len = strlen(time_str);
    if (len > 0 && time_str[len-1] == '\n') {
        time_str[len-1] = '\0';
        len--;
    }
    
    *result = malloc(len + 1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    strcpy(*result, time_str);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Increment and return counter
 */
static int host_increment_counter(int argc, const char* const* argv, char** result) {
    if (argc != 0) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    counter++;
    
    *result = malloc(16);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    snprintf(*result, 16, "%d", counter);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: String concatenation
 */
static int host_string_concat(int argc, const char* const* argv, char** result) {
    if (argc < 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    // Calculate total length
    size_t total_len = 0;
    for (int i = 0; i < argc; i++) {
        total_len += strlen(argv[i]);
    }
    
    *result = malloc(total_len + 1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    // Concatenate strings
    (*result)[0] = '\0';
    for (int i = 0; i < argc; i++) {
        strcat(*result, argv[i]);
    }
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Register all host functions
 */
static int register_host_functions(LambdustContext* ctx) {
    int error;
    
    error = lambdust_register_function(ctx, "host-print", host_print);
    if (error != LAMBDUST_SUCCESS) return error;
    
    error = lambdust_register_function(ctx, "host-add", host_add);
    if (error != LAMBDUST_SUCCESS) return error;
    
    error = lambdust_register_function(ctx, "host-sqrt", host_sqrt);
    if (error != LAMBDUST_SUCCESS) return error;
    
    error = lambdust_register_function(ctx, "host-current-time", host_current_time);
    if (error != LAMBDUST_SUCCESS) return error;
    
    error = lambdust_register_function(ctx, "host-increment-counter", host_increment_counter);
    if (error != LAMBDUST_SUCCESS) return error;
    
    error = lambdust_register_function(ctx, "host-string-concat", host_string_concat);
    if (error != LAMBDUST_SUCCESS) return error;
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Execute and print result of Scheme expression
 */
static void execute_scheme(LambdustContext* ctx, const char* expr) {
    printf(">>> %s\n", expr);
    
    char* result = NULL;
    int error = lambdust_eval(ctx, expr, &result);
    
    if (error == LAMBDUST_SUCCESS) {
        if (strlen(result) > 0) {
            printf("    %s\n", result);
        }
        lambdust_free_string(result);
    } else {
        const char* error_msg = lambdust_get_last_error(ctx);
        printf("    Error: %s\n", error_msg ? error_msg : "Unknown error");
    }
    printf("\n");
}

int main() {
    printf("=== Lambdust Host Functions Example ===\n\n");
    
    // Create context
    LambdustContext* ctx = lambdust_create_context();
    if (!ctx) {
        fprintf(stderr, "Failed to create context\n");
        return 1;
    }
    
    // Register host functions
    if (register_host_functions(ctx) != LAMBDUST_SUCCESS) {
        fprintf(stderr, "Failed to register host functions\n");
        lambdust_destroy_context(ctx);
        return 1;
    }
    
    printf("Host functions registered successfully\n\n");
    
    // Test basic host function calls
    printf("=== Basic Host Function Calls ===\n");
    execute_scheme(ctx, "(host-print \"Hello from Scheme!\")");
    execute_scheme(ctx, "(host-add 10 20)");
    execute_scheme(ctx, "(host-sqrt 16)");
    execute_scheme(ctx, "(host-current-time)");
    
    // Test counter (stateful function)
    printf("=== Stateful Functions ===\n");
    execute_scheme(ctx, "(host-increment-counter)");
    execute_scheme(ctx, "(host-increment-counter)");
    execute_scheme(ctx, "(host-increment-counter)");
    
    // Test string operations
    printf("=== String Operations ===\n");
    execute_scheme(ctx, "(host-string-concat \"Hello\" \", \" \"World\" \"!\")");
    
    // Combine host functions with Scheme functions
    printf("=== Combined Operations ===\n");
    execute_scheme(ctx, "(define (double-and-add x y) (host-add (* x 2) y))");
    execute_scheme(ctx, "(double-and-add 5 3)");
    
    // Mathematical operations
    execute_scheme(ctx, "(define (hypotenuse a b) (host-sqrt (+ (* a a) (* b b))))");
    execute_scheme(ctx, "(hypotenuse 3 4)");
    
    // Create a Scheme function that uses multiple host functions
    printf("=== Complex Integration ===\n");
    execute_scheme(ctx, 
        "(define (report-calculation x y)"
        "  (let ((result (host-add x y)))"
        "    (host-print (host-string-concat \"The sum of \" "
        "                                   (number->string x) "
        "                                   \" and \" "
        "                                   (number->string y) "
        "                                   \" is \" "
        "                                   (number->string result)))"
        "    result))");
    execute_scheme(ctx, "(report-calculation 15 25)");
    
    // Error handling
    printf("=== Error Handling ===\n");
    execute_scheme(ctx, "(host-sqrt -1)"); // Should cause an error
    execute_scheme(ctx, "(host-add 1 2 3)"); // Wrong arity
    
    // Cleanup
    lambdust_destroy_context(ctx);
    printf("=== Example completed ===\n");
    
    return 0;
}