/**
 * @file plugin_system.c
 * @brief Plugin system example using Lambdust
 * 
 * This example demonstrates how to build a plugin system where Scheme
 * code acts as plugins that can extend application functionality.
 * Shows advanced integration patterns and dynamic loading.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include "lambdust.h"

// Plugin information
typedef struct {
    char name[64];
    char version[16];
    char author[64];
    char description[256];
    char main_function[64];
} PluginInfo;

// Application context
typedef struct {
    LambdustContext* scheme_ctx;
    PluginInfo* plugins;
    int plugin_count;
    int plugin_capacity;
} AppContext;

/**
 * @brief Host function: Log messages from plugins
 */
static int host_log(int argc, const char* const* argv, char** result) {
    if (argc < 1 || argc > 2) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    const char* level = (argc > 1) ? argv[0] : "INFO";
    const char* message = (argc > 1) ? argv[1] : argv[0];
    
    printf("[%s] %s\n", level, message);
    
    *result = malloc(1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    (*result)[0] = '\0';
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Get application configuration
 */
static int host_get_config(int argc, const char* const* argv, char** result) {
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    const char* key = argv[0];
    const char* value = NULL;
    
    // Simulate configuration lookup
    if (strcmp(key, "app_name") == 0) {
        value = "Lambdust Plugin Demo";
    } else if (strcmp(key, "version") == 0) {
        value = "1.0.0";
    } else if (strcmp(key, "data_dir") == 0) {
        value = "./data";
    } else if (strcmp(key, "max_plugins") == 0) {
        value = "10";
    } else {
        value = "";
    }
    
    size_t len = strlen(value);
    *result = malloc(len + 1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    strcpy(*result, value);
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Call other plugins
 */
static int host_call_plugin(int argc, const char* const* argv, char** result) {
    static AppContext* app_ctx = NULL; // Set externally
    
    if (argc < 2) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    const char* plugin_name = argv[0];
    const char* function_name = argv[1];
    
    // Build function call
    char call_expr[512];
    if (argc == 2) {
        snprintf(call_expr, sizeof(call_expr), "(%s-%s)", plugin_name, function_name);
    } else {
        // With arguments - simplified for demo
        snprintf(call_expr, sizeof(call_expr), "(%s-%s \"%s\")", 
                plugin_name, function_name, argv[2]);
    }
    
    // Execute in Scheme context
    char* scheme_result = NULL;
    int error = lambdust_eval(app_ctx->scheme_ctx, call_expr, &scheme_result);
    
    if (error == LAMBDUST_SUCCESS) {
        size_t len = strlen(scheme_result);
        *result = malloc(len + 1);
        if (*result) {
            strcpy(*result, scheme_result);
        }
        lambdust_free_string(scheme_result);
        return *result ? LAMBDUST_SUCCESS : LAMBDUST_MEMORY_ERROR;
    } else {
        return LAMBDUST_RUNTIME_ERROR;
    }
}

/**
 * @brief Initialize application context
 */
static AppContext* app_create() {
    AppContext* app = malloc(sizeof(AppContext));
    if (!app) return NULL;
    
    app->scheme_ctx = lambdust_create_context();
    if (!app->scheme_ctx) {
        free(app);
        return NULL;
    }
    
    app->plugin_capacity = 10;
    app->plugins = malloc(sizeof(PluginInfo) * app->plugin_capacity);
    app->plugin_count = 0;
    
    // Register host functions
    lambdust_register_function(app->scheme_ctx, "log", host_log);
    lambdust_register_function(app->scheme_ctx, "get-config", host_get_config);
    lambdust_register_function(app->scheme_ctx, "call-plugin", host_call_plugin);
    
    // Set up plugin API in Scheme
    lambdust_eval(app->scheme_ctx,
        "(define (plugin-info name version author description main-func)"
        "  (list 'plugin-info name version author description main-func))", NULL);
    
    lambdust_eval(app->scheme_ctx,
        "(define (register-command name func)"
        "  (log \"DEBUG\" (string-append \"Registering command: \" name)))", NULL);
    
    return app;
}

/**
 * @brief Destroy application context
 */
static void app_destroy(AppContext* app) {
    if (app) {
        if (app->scheme_ctx) {
            lambdust_destroy_context(app->scheme_ctx);
        }
        if (app->plugins) {
            free(app->plugins);
        }
        free(app);
    }
}

/**
 * @brief Load a plugin from file
 */
static int load_plugin(AppContext* app, const char* filename) {
    printf("Loading plugin: %s\n", filename);
    
    // Read file
    FILE* file = fopen(filename, "r");
    if (!file) {
        fprintf(stderr, "Cannot open plugin file: %s\n", filename);
        return -1;
    }
    
    // Get file size
    fseek(file, 0, SEEK_END);
    long size = ftell(file);
    fseek(file, 0, SEEK_SET);
    
    // Read content
    char* content = malloc(size + 1);
    if (!content) {
        fclose(file);
        return -1;
    }
    
    fread(content, 1, size, file);
    content[size] = '\0';
    fclose(file);
    
    // Evaluate plugin code
    char* result = NULL;
    int error = lambdust_eval(app->scheme_ctx, content, &result);
    
    if (error == LAMBDUST_SUCCESS) {
        printf("  Plugin loaded successfully\n");
        if (result && strlen(result) > 0) {
            printf("  Result: %s\n", result);
        }
        lambdust_free_string(result);
        
        // Extract plugin name from filename
        const char* basename = strrchr(filename, '/');
        basename = basename ? basename + 1 : filename;
        
        // Remove .scm extension
        if (app->plugin_count < app->plugin_capacity) {
            strncpy(app->plugins[app->plugin_count].name, basename, 
                   sizeof(app->plugins[app->plugin_count].name) - 1);
            app->plugin_count++;
        }
    } else {
        const char* error_msg = lambdust_get_last_error(app->scheme_ctx);
        printf("  Plugin load failed: %s\n", error_msg ? error_msg : "Unknown error");
    }
    
    free(content);
    return error == LAMBDUST_SUCCESS ? 0 : -1;
}

/**
 * @brief Load plugins from directory
 */
static void load_plugins_from_directory(AppContext* app, const char* plugin_dir) {
    DIR* dir = opendir(plugin_dir);
    if (!dir) {
        printf("Plugin directory not found: %s\n", plugin_dir);
        return;
    }
    
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (entry->d_type == DT_REG) { // Regular file
            // Check if it's a .scm file
            const char* ext = strrchr(entry->d_name, '.');
            if (ext && strcmp(ext, ".scm") == 0) {
                char filepath[512];
                snprintf(filepath, sizeof(filepath), "%s/%s", plugin_dir, entry->d_name);
                load_plugin(app, filepath);
            }
        }
    }
    
    closedir(dir);
}

/**
 * @brief Create sample plugins
 */
static void create_sample_plugins() {
    // Create plugins directory
    mkdir("plugins", 0755);
    
    // Sample plugin 1: Math utilities
    FILE* f = fopen("plugins/math_utils.scm", "w");
    if (f) {
        fprintf(f, 
            ";; Math utilities plugin\n"
            "(plugin-info \"math-utils\" \"1.0\" \"Demo Author\" \n"
            "             \"Mathematical utility functions\" \"math-utils-main\")\n"
            "\n"
            "(define (math-utils-factorial n)\n"
            "  (if (<= n 1) 1 (* n (math-utils-factorial (- n 1)))))\n"
            "\n"
            "(define (math-utils-fibonacci n)\n"
            "  (cond ((<= n 0) 0)\n"
            "        ((= n 1) 1)\n"
            "        (else (+ (math-utils-fibonacci (- n 1))\n"
            "                 (math-utils-fibonacci (- n 2))))))\n"
            "\n"
            "(define (math-utils-gcd a b)\n"
            "  (if (= b 0) a (math-utils-gcd b (remainder a b))))\n"
            "\n"
            "(define (math-utils-main)\n"
            "  (log \"INFO\" \"Math utilities plugin initialized\")\n"
            "  (register-command \"factorial\" math-utils-factorial)\n"
            "  (register-command \"fibonacci\" math-utils-fibonacci)\n"
            "  (register-command \"gcd\" math-utils-gcd))\n"
            "\n"
            ";; Initialize plugin\n"
            "(math-utils-main)\n");
        fclose(f);
    }
    
    // Sample plugin 2: String utilities
    f = fopen("plugins/string_utils.scm", "w");
    if (f) {
        fprintf(f,
            ";; String utilities plugin\n"
            "(plugin-info \"string-utils\" \"1.0\" \"Demo Author\"\n"
            "             \"String manipulation utilities\" \"string-utils-main\")\n"
            "\n"
            "(define (string-utils-reverse str)\n"
            "  (list->string (reverse (string->list str))))\n"
            "\n"
            "(define (string-utils-uppercase str)\n"
            "  ;; Simplified uppercase (demo only)\n"
            "  str)\n"
            "\n"
            "(define (string-utils-word-count str)\n"
            "  (length (string-split str #\\space)))\n"
            "\n"
            "(define (string-split str delimiter)\n"
            "  ;; Simplified split function\n"
            "  (list str))\n"
            "\n"
            "(define (string-utils-main)\n"
            "  (log \"INFO\" \"String utilities plugin initialized\")\n"
            "  (register-command \"reverse\" string-utils-reverse)\n"
            "  (register-command \"word-count\" string-utils-word-count))\n"
            "\n"
            ";; Initialize plugin\n"
            "(string-utils-main)\n");
        fclose(f);
    }
    
    // Sample plugin 3: System info
    f = fopen("plugins/system_info.scm", "w");
    if (f) {
        fprintf(f,
            ";; System information plugin\n"
            "(plugin-info \"system-info\" \"1.0\" \"Demo Author\"\n"
            "             \"System information utilities\" \"system-info-main\")\n"
            "\n"
            "(define (system-info-app-name)\n"
            "  (get-config \"app_name\"))\n"
            "\n"
            "(define (system-info-version)\n"
            "  (get-config \"version\"))\n"
            "\n"
            "(define (system-info-data-dir)\n"
            "  (get-config \"data_dir\"))\n"
            "\n"
            "(define (system-info-summary)\n"
            "  (string-append \"App: \" (system-info-app-name)\n"
            "                 \" v\" (system-info-version)\n"
            "                 \" (data: \" (system-info-data-dir) \")\"))\n"
            "\n"
            "(define (system-info-main)\n"
            "  (log \"INFO\" \"System info plugin initialized\")\n"
            "  (register-command \"app-name\" system-info-app-name)\n"
            "  (register-command \"version\" system-info-version)\n"
            "  (register-command \"summary\" system-info-summary))\n"
            "\n"
            ";; Initialize plugin\n"
            "(system-info-main)\n");
        fclose(f);
    }
}

/**
 * @brief Test plugin functionality
 */
static void test_plugins(AppContext* app) {
    printf("\n=== Testing Plugin Functionality ===\n");
    
    // Test math utilities
    char* result = NULL;
    
    printf("\nTesting math-utils plugin:\n");
    lambdust_eval(app->scheme_ctx, "(math-utils-factorial 5)", &result);
    printf("factorial(5) = %s\n", result ? result : "error");
    lambdust_free_string(result);
    
    lambdust_eval(app->scheme_ctx, "(math-utils-fibonacci 8)", &result);
    printf("fibonacci(8) = %s\n", result ? result : "error");
    lambdust_free_string(result);
    
    lambdust_eval(app->scheme_ctx, "(math-utils-gcd 48 18)", &result);
    printf("gcd(48, 18) = %s\n", result ? result : "error");
    lambdust_free_string(result);
    
    // Test string utilities
    printf("\nTesting string-utils plugin:\n");
    lambdust_eval(app->scheme_ctx, "(string-utils-reverse \"hello\")", &result);
    printf("reverse(\"hello\") = %s\n", result ? result : "error");
    lambdust_free_string(result);
    
    // Test system info
    printf("\nTesting system-info plugin:\n");
    lambdust_eval(app->scheme_ctx, "(system-info-summary)", &result);
    printf("system summary = %s\n", result ? result : "error");
    lambdust_free_string(result);
}

int main() {
    printf("=== Lambdust Plugin System Example ===\n\n");
    
    // Create application
    AppContext* app = app_create();
    if (!app) {
        fprintf(stderr, "Failed to create application context\n");
        return 1;
    }
    
    printf("Application initialized\n");
    printf("Lambdust version: %s\n", lambdust_get_version());
    
    // Create sample plugins
    create_sample_plugins();
    printf("Sample plugins created\n");
    
    // Load plugins
    printf("\n=== Loading Plugins ===\n");
    load_plugins_from_directory(app, "plugins");
    
    printf("\nLoaded %d plugins:\n", app->plugin_count);
    for (int i = 0; i < app->plugin_count; i++) {
        printf("  - %s\n", app->plugins[i].name);
    }
    
    // Test plugin functionality
    test_plugins(app);
    
    // Cleanup
    app_destroy(app);
    printf("\n=== Plugin system demo completed ===\n");
    
    return 0;
}