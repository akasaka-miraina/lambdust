/**
 * @file basic_usage.c
 * @brief Basic Lambdust C integration example
 * 
 * This example demonstrates the fundamental usage of Lambdust
 * from C code, including context creation, code evaluation,
 * error handling, and proper cleanup.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "lambdust.h"

/**
 * @brief Evaluate a Scheme expression and print the result
 * 
 * @param ctx Lambdust context
 * @param expression Scheme expression to evaluate
 * @return 0 on success, non-zero on failure
 */
static int evaluate_and_print(LambdustContext* ctx, const char* expression) {
    printf("Evaluating: %s\n", expression);
    
    char* result = NULL;
    int error = lambdust_eval(ctx, expression, &result);
    
    if (error == LAMBDUST_SUCCESS) {
        printf("Result: %s\n", result);
        lambdust_free_string(result);
        return 0;
    } else {
        const char* error_msg = lambdust_get_last_error(ctx);
        printf("Error (%d): %s\n", error, error_msg ? error_msg : "Unknown error");
        return 1;
    }
}

int main() {
    printf("=== Lambdust Basic Usage Example ===\n\n");
    
    // Check library health
    if (!lambdust_check_library()) {
        fprintf(stderr, "Lambdust library health check failed\n");
        return 1;
    }
    
    printf("Library version: %s\n\n", lambdust_get_version());
    
    // Create context
    LambdustContext* ctx = lambdust_create_context();
    if (!ctx) {
        fprintf(stderr, "Failed to create Lambdust context\n");
        return 1;
    }
    
    printf("Context created successfully\n\n");
    
    // Basic arithmetic
    printf("=== Basic Arithmetic ===\n");
    evaluate_and_print(ctx, "(+ 1 2 3)");
    evaluate_and_print(ctx, "(* 6 7)");
    evaluate_and_print(ctx, "(- 100 25)");
    evaluate_and_print(ctx, "(/ 84 12)");
    printf("\n");
    
    // List operations
    printf("=== List Operations ===\n");
    evaluate_and_print(ctx, "(list 1 2 3 4 5)");
    evaluate_and_print(ctx, "(length '(a b c d))");
    evaluate_and_print(ctx, "(append '(1 2) '(3 4))");
    evaluate_and_print(ctx, "(reverse '(1 2 3 4))");
    printf("\n");
    
    // String operations
    printf("=== String Operations ===\n");
    evaluate_and_print(ctx, "(string-length \"Hello, World!\")");
    evaluate_and_print(ctx, "(string-append \"Hello\" \", \" \"World!\")");
    printf("\n");
    
    // Variable definitions
    printf("=== Variable Definitions ===\n");
    evaluate_and_print(ctx, "(define pi 3.14159)");
    evaluate_and_print(ctx, "pi");
    evaluate_and_print(ctx, "(define greeting \"Hello, Scheme!\")");
    evaluate_and_print(ctx, "greeting");
    printf("\n");
    
    // Function definitions
    printf("=== Function Definitions ===\n");
    evaluate_and_print(ctx, "(define (square x) (* x x))");
    evaluate_and_print(ctx, "(square 5)");
    evaluate_and_print(ctx, "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))");
    evaluate_and_print(ctx, "(factorial 5)");
    printf("\n");
    
    // Conditional expressions
    printf("=== Conditional Expressions ===\n");
    evaluate_and_print(ctx, "(if (> 5 3) 'greater 'less-or-equal)");
    evaluate_and_print(ctx, "(cond ((< 2 1) 'impossible) ((> 3 2) 'possible) (else 'default))");
    printf("\n");
    
    // Error handling demonstration
    printf("=== Error Handling ===\n");
    printf("Attempting to evaluate invalid expression:\n");
    evaluate_and_print(ctx, "(+ 1 2"); // Missing closing parenthesis
    printf("\n");
    
    // Cleanup
    lambdust_destroy_context(ctx);
    printf("Context destroyed successfully\n");
    printf("\n=== Example completed ===\n");
    
    return 0;
}