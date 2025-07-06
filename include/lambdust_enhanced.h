/**
 * @file lambdust_enhanced.h
 * @brief Enhanced C API for Lambdust Scheme interpreter with advanced safety features
 * 
 * This header provides enhanced safety features including:
 * - Thread safety validation
 * - Memory tracking and limits
 * - Resource management
 * - Callback mechanisms
 * - Security features
 * 
 * Include this header in addition to lambdust.h for advanced functionality.
 * 
 * @version 0.1.1
 * @author Lambdust Team
 */

#ifndef LAMBDUST_ENHANCED_H
#define LAMBDUST_ENHANCED_H

#include "lambdust.h"
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Enhanced Error Codes */
typedef enum {
    LAMBDUST_THREAD_SAFETY_ERROR = 10,    /**< Thread safety violation */
    LAMBDUST_RESOURCE_LIMIT_ERROR = 11,   /**< Resource limit exceeded */
    LAMBDUST_CORRUPTED_CONTEXT = 12,      /**< Context corrupted */
    LAMBDUST_CALLBACK_ERROR = 13,         /**< Callback function error */
    LAMBDUST_SECURITY_ERROR = 14,         /**< Security violation */
} LambdustEnhancedErrorCode;

/* Enhanced Function Types */

/**
 * @brief Enhanced host function with user data support
 * 
 * @param argc Number of arguments
 * @param argv Array of argument strings
 * @param result Output result string (allocated by function)
 * @param user_data User-provided data pointer
 * @return Error code
 */
typedef int (*LambdustEnhancedHostFunction)(
    int argc,
    const char* const* argv,
    char** result,
    void* user_data
);

/**
 * @brief Error callback function
 * 
 * @param context The context where error occurred
 * @param error_code The error code
 * @param error_message The error message
 * @param user_data User-provided data pointer
 */
typedef void (*LambdustErrorCallback)(
    LambdustContext* context,
    int error_code,
    const char* error_message,
    void* user_data
);

/* Enhanced Memory Management */

/**
 * @brief Allocate memory with tracking
 * 
 * Allocates memory that is tracked by the context for automatic cleanup
 * and memory limit enforcement.
 * 
 * @param context Lambdust context
 * @param size Size in bytes to allocate
 * @return Pointer to allocated memory, or NULL on failure
 */
char* lambdust_alloc_tracked(LambdustContext* context, size_t size);

/**
 * @brief Free tracked memory
 * 
 * Frees memory that was allocated with lambdust_alloc_tracked.
 * 
 * @param context Lambdust context
 * @param ptr Pointer to free
 * @return Error code
 */
int lambdust_free_tracked(LambdustContext* context, char* ptr);

/**
 * @brief Get memory usage statistics
 * 
 * @param context Lambdust context
 * @param total_allocated Pointer to store total allocated bytes (optional)
 * @param peak_usage Pointer to store peak memory usage (optional)
 * @param allocation_count Pointer to store allocation count (optional)
 * @return Error code
 */
int lambdust_get_memory_stats(
    LambdustContext* context,
    size_t* total_allocated,
    size_t* peak_usage,
    uint64_t* allocation_count
);

/* Enhanced Function Registration */

/**
 * @brief Register enhanced host function with user data
 * 
 * Registers a host function that can access user data and specifies
 * whether the function is thread-safe.
 * 
 * @param context Lambdust context
 * @param name Function name in Scheme
 * @param func Function pointer
 * @param user_data User data pointer (can be NULL)
 * @param thread_safe Whether function is thread-safe
 * @return Error code
 */
int lambdust_register_function_enhanced(
    LambdustContext* context,
    const char* name,
    LambdustEnhancedHostFunction func,
    void* user_data,
    bool thread_safe
);

/**
 * @brief Set error callback for enhanced error handling
 * 
 * @param context Lambdust context
 * @param callback Error callback function
 * @param user_data User data for callback
 * @return Error code
 */
int lambdust_set_error_callback(
    LambdustContext* context,
    LambdustErrorCallback callback,
    void* user_data
);

/* Context Health and Safety */

/**
 * @brief Check context health and integrity
 * 
 * Performs comprehensive checks on context validity, thread safety,
 * and resource integrity.
 * 
 * @param context Lambdust context
 * @return Error code (LAMBDUST_SUCCESS if healthy)
 */
int lambdust_check_context_health(LambdustContext* context);

/**
 * @brief Increment context reference count
 * 
 * For shared contexts, increment the reference count to prevent
 * premature destruction.
 * 
 * @param context Lambdust context
 * @return Error code
 */
int lambdust_context_ref(LambdustContext* context);

/* Enhanced Evaluation */

/**
 * @brief Evaluate with timeout
 * 
 * Evaluates Scheme code with a timeout to prevent infinite loops.
 * 
 * @param context Lambdust context
 * @param code Scheme code to evaluate
 * @param result Output result string
 * @param timeout_ms Timeout in milliseconds
 * @return Error code
 */
int lambdust_eval_with_timeout(
    LambdustContext* context,
    const char* code,
    char** result,
    uint32_t timeout_ms
);

/* Sandboxing */

/**
 * @brief Create sandboxed context with resource limits
 * 
 * Creates a context with enforced resource limits for security.
 * 
 * @param max_memory Maximum memory usage in bytes
 * @param max_execution_time_ms Maximum execution time in milliseconds
 * @return Sandboxed context or NULL on failure
 */
LambdustContext* lambdust_create_sandboxed_context(
    size_t max_memory,
    uint32_t max_execution_time_ms
);

/* Advanced Error Handling */

/**
 * @brief Get detailed error information
 * 
 * Retrieves comprehensive error information including location data.
 * 
 * @param context Lambdust context
 * @param error_code Pointer to store error code (optional)
 * @param error_message Pointer to store error message (optional)
 * @param error_location Pointer to store error location (optional)
 * @return Error code
 */
int lambdust_get_detailed_error(
    LambdustContext* context,
    int* error_code,
    const char** error_message,
    const char** error_location
);

/* Security */

/**
 * @brief Clear sensitive data from context
 * 
 * Clears potentially sensitive data like error messages and user data
 * from the context for security purposes.
 * 
 * @param context Lambdust context
 * @return Error code
 */
int lambdust_clear_sensitive_data(LambdustContext* context);

/* Utility Macros */

/**
 * @brief Check if error code indicates success
 */
#define LAMBDUST_IS_SUCCESS(code) ((code) == LAMBDUST_SUCCESS)

/**
 * @brief Check if error code indicates failure
 */
#define LAMBDUST_IS_ERROR(code) ((code) != LAMBDUST_SUCCESS)

/**
 * @brief Get error category
 */
#define LAMBDUST_ERROR_CATEGORY(code) \
    ((code) < 5 ? "Basic" : \
     (code) < 10 ? "Evaluation" : \
     (code) < 15 ? "Enhanced" : "Unknown")

/* Constants */

/** Default memory limit for sandboxed contexts (100MB) */
#define LAMBDUST_DEFAULT_MEMORY_LIMIT (100 * 1024 * 1024)

/** Default execution timeout (30 seconds) */
#define LAMBDUST_DEFAULT_TIMEOUT_MS (30 * 1000)

/** Maximum context age (24 hours) */
#define LAMBDUST_MAX_CONTEXT_AGE_MS (24 * 60 * 60 * 1000)

#ifdef __cplusplus
}
#endif

#endif /* LAMBDUST_ENHANCED_H */

/*
 * Example Usage:
 * 
 * #include "lambdust.h"
 * #include "lambdust_enhanced.h"
 * 
 * // Enhanced host function
 * int my_enhanced_function(int argc, const char* const* argv, 
 *                         char** result, void* user_data) {
 *     MyData* data = (MyData*)user_data;
 *     // Use user data...
 *     *result = lambdust_alloc_tracked(data->context, strlen("result") + 1);
 *     strcpy(*result, "result");
 *     return LAMBDUST_SUCCESS;
 * }
 * 
 * // Error callback
 * void my_error_callback(LambdustContext* ctx, int code, 
 *                       const char* msg, void* user_data) {
 *     printf("Error %d: %s\n", code, msg);
 * }
 * 
 * int main() {
 *     // Create context
 *     LambdustContext* ctx = lambdust_create_context();
 *     
 *     // Set error callback
 *     lambdust_set_error_callback(ctx, my_error_callback, NULL);
 *     
 *     // Register enhanced function
 *     MyData data = {ctx};
 *     lambdust_register_function_enhanced(ctx, "my-func", 
 *                                        my_enhanced_function, &data, true);
 *     
 *     // Check health
 *     if (lambdust_check_context_health(ctx) == LAMBDUST_SUCCESS) {
 *         // Safe to use
 *         char* result = NULL;
 *         int error = lambdust_eval_with_timeout(ctx, "(my-func)", 
 *                                               &result, 5000);
 *         if (LAMBDUST_IS_SUCCESS(error)) {
 *             printf("Result: %s\n", result);
 *             lambdust_free_tracked(ctx, result);
 *         }
 *     }
 *     
 *     // Get memory stats
 *     size_t total, peak;
 *     uint64_t count;
 *     lambdust_get_memory_stats(ctx, &total, &peak, &count);
 *     printf("Memory: %zu total, %zu peak, %llu allocations\n", 
 *            total, peak, count);
 *     
 *     // Clean up
 *     lambdust_clear_sensitive_data(ctx);
 *     lambdust_destroy_context(ctx);
 *     return 0;
 * }
 */