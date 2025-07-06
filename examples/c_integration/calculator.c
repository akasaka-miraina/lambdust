/**
 * @file calculator.c
 * @brief Interactive calculator using Lambdust
 * 
 * This example demonstrates building a practical application that uses
 * Scheme as its expression evaluation engine, showing how Lambdust can
 * be embedded in real applications.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include "lambdust.h"

// Calculator state
typedef struct {
    LambdustContext* ctx;
    double memory;
    int precision;
} Calculator;

/**
 * @brief Host function: Store value in memory
 */
static int host_store_memory(int argc, const char* const* argv, char** result) {
    static double* memory_ptr = NULL;
    
    if (!memory_ptr) {
        fprintf(stderr, "Memory not initialized\n");
        return LAMBDUST_RUNTIME_ERROR;
    }
    
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    *memory_ptr = atof(argv[0]);
    
    *result = malloc(32);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    snprintf(*result, 32, "%.6g", *memory_ptr);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Recall value from memory
 */
static int host_recall_memory(int argc, const char* const* argv, char** result) {
    static double* memory_ptr = NULL;
    
    if (!memory_ptr) {
        fprintf(stderr, "Memory not initialized\n");
        return LAMBDUST_RUNTIME_ERROR;
    }
    
    if (argc != 0) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    *result = malloc(32);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    snprintf(*result, 32, "%.6g", *memory_ptr);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Advanced mathematical functions
 */
static int host_advanced_math(int argc, const char* const* argv, char** result) {
    if (argc < 1 || argc > 2) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    const char* operation = argv[0];
    double value = (argc > 1) ? atof(argv[1]) : 0.0;
    double result_val = 0.0;
    
    if (strcmp(operation, "sin") == 0) {
        result_val = sin(value);
    } else if (strcmp(operation, "cos") == 0) {
        result_val = cos(value);
    } else if (strcmp(operation, "tan") == 0) {
        result_val = tan(value);
    } else if (strcmp(operation, "log") == 0) {
        if (value <= 0) return LAMBDUST_RUNTIME_ERROR;
        result_val = log(value);
    } else if (strcmp(operation, "log10") == 0) {
        if (value <= 0) return LAMBDUST_RUNTIME_ERROR;
        result_val = log10(value);
    } else if (strcmp(operation, "exp") == 0) {
        result_val = exp(value);
    } else if (strcmp(operation, "pi") == 0) {
        result_val = M_PI;
    } else if (strcmp(operation, "e") == 0) {
        result_val = M_E;
    } else {
        return LAMBDUST_INVALID_ARGUMENT;
    }
    
    *result = malloc(32);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    snprintf(*result, 32, "%.10g", result_val);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Initialize calculator with host functions
 */
static Calculator* calculator_create() {
    Calculator* calc = malloc(sizeof(Calculator));
    if (!calc) return NULL;
    
    calc->ctx = lambdust_create_context();
    if (!calc->ctx) {
        free(calc);
        return NULL;
    }
    
    calc->memory = 0.0;
    calc->precision = 6;
    
    // Set up memory pointer for host functions
    static double* memory_ptr = &calc->memory;
    (void)memory_ptr; // Use the memory pointer appropriately
    
    // Register host functions
    lambdust_register_function(calc->ctx, "store", host_store_memory);
    lambdust_register_function(calc->ctx, "recall", host_recall_memory);
    lambdust_register_function(calc->ctx, "math", host_advanced_math);
    
    // Define utility functions in Scheme
    lambdust_eval(calc->ctx, 
        "(define (deg->rad deg) (* deg (/ (math \"pi\") 180)))", NULL);
    lambdust_eval(calc->ctx, 
        "(define (rad->deg rad) (* rad (/ 180 (math \"pi\"))))", NULL);
    lambdust_eval(calc->ctx, 
        "(define (sin-deg deg) (math \"sin\" (deg->rad deg)))", NULL);
    lambdust_eval(calc->ctx, 
        "(define (cos-deg deg) (math \"cos\" (deg->rad deg)))", NULL);
    lambdust_eval(calc->ctx, 
        "(define (tan-deg deg) (math \"tan\" (deg->rad deg)))", NULL);
    
    // Mathematical constants and common functions
    lambdust_eval(calc->ctx, "(define pi (math \"pi\"))", NULL);
    lambdust_eval(calc->ctx, "(define e (math \"e\"))", NULL);
    lambdust_eval(calc->ctx, "(define (pow x n) (expt x n))", NULL);
    lambdust_eval(calc->ctx, "(define (ln x) (math \"log\" x))", NULL);
    lambdust_eval(calc->ctx, "(define (log x) (math \"log10\" x))", NULL);
    
    return calc;
}

/**
 * @brief Destroy calculator and free resources
 */
static void calculator_destroy(Calculator* calc) {
    if (calc) {
        if (calc->ctx) {
            lambdust_destroy_context(calc->ctx);
        }
        free(calc);
    }
}

/**
 * @brief Evaluate expression and return result
 */
static int calculator_eval(Calculator* calc, const char* expression, char** result) {
    return lambdust_eval(calc->ctx, expression, result);
}

/**
 * @brief Print calculator help
 */
static void print_help() {
    printf("Calculator Commands:\n");
    printf("  Basic: +, -, *, /, sqrt, expt\n");
    printf("  Trig: sin, cos, tan (radians), sin-deg, cos-deg, tan-deg (degrees)\n");
    printf("  Log: ln (natural log), log (base 10), exp\n");
    printf("  Constants: pi, e\n");
    printf("  Memory: (store value), (recall)\n");
    printf("  Functions: (define name (lambda (args) body))\n");
    printf("  Variables: (define name value)\n");
    printf("  Examples:\n");
    printf("    (+ 2 3 4)\n");
    printf("    (* pi 2)\n");
    printf("    (sin-deg 30)\n");
    printf("    (sqrt (+ (* 3 3) (* 4 4)))\n");
    printf("    (store 42) then (recall)\n");
    printf("  Type 'help' for this message, 'quit' to exit\n\n");
}

/**
 * @brief Interactive calculator mode
 */
static void interactive_mode(Calculator* calc) {
    char input[1024];
    
    printf("=== Lambdust Interactive Calculator ===\n");
    printf("Type 'help' for commands, 'quit' to exit\n\n");
    
    while (1) {
        printf("calc> ");
        fflush(stdout);
        
        if (!fgets(input, sizeof(input), stdin)) {
            break;
        }
        
        // Remove newline
        size_t len = strlen(input);
        if (len > 0 && input[len-1] == '\n') {
            input[len-1] = '\0';
        }
        
        // Skip empty input
        if (strlen(input) == 0) {
            continue;
        }
        
        // Handle special commands
        if (strcmp(input, "quit") == 0 || strcmp(input, "exit") == 0) {
            break;
        }
        
        if (strcmp(input, "help") == 0) {
            print_help();
            continue;
        }
        
        if (strcmp(input, "memory") == 0) {
            printf("Memory: %.6g\n", calc->memory);
            continue;
        }
        
        // Evaluate expression
        char* result = NULL;
        int error = calculator_eval(calc, input, &result);
        
        if (error == LAMBDUST_SUCCESS) {
            printf("= %s\n", result);
            lambdust_free_string(result);
        } else {
            const char* error_msg = lambdust_get_last_error(calc->ctx);
            printf("Error: %s\n", error_msg ? error_msg : "Unknown error");
        }
        
        printf("\n");
    }
    
    printf("Goodbye!\n");
}

/**
 * @brief Command line mode
 */
static int command_line_mode(Calculator* calc, const char* expression) {
    char* result = NULL;
    int error = calculator_eval(calc, expression, &result);
    
    if (error == LAMBDUST_SUCCESS) {
        printf("%s\n", result);
        lambdust_free_string(result);
        return 0;
    } else {
        const char* error_msg = lambdust_get_last_error(calc->ctx);
        fprintf(stderr, "Error: %s\n", error_msg ? error_msg : "Unknown error");
        return 1;
    }
}

int main(int argc, char* argv[]) {
    Calculator* calc = calculator_create();
    if (!calc) {
        fprintf(stderr, "Failed to create calculator\n");
        return 1;
    }
    
    int result = 0;
    
    if (argc > 1) {
        // Command line mode
        result = command_line_mode(calc, argv[1]);
    } else {
        // Interactive mode
        interactive_mode(calc);
    }
    
    calculator_destroy(calc);
    return result;
}