/**
 * @file config_example.c
 * @brief Configuration management example using Lambdust
 * 
 * This example demonstrates using Scheme for application configuration,
 * showing how to create flexible, programmable configuration files.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include "lambdust.h"

// Configuration structure
typedef struct {
    char app_name[64];
    char version[16];
    int window_width;
    int window_height;
    int max_connections;
    double timeout;
    int debug_enabled;
    char log_level[16];
    char data_directory[256];
    char* plugins[10];
    int plugin_count;
} AppConfig;

// Global configuration
static AppConfig g_config = {0};

/**
 * @brief Host function: Set configuration value
 */
static int host_set_config(int argc, const char* const* argv, char** result) {
    if (argc != 2) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    const char* key = argv[0];
    const char* value = argv[1];
    
    if (strcmp(key, "app-name") == 0) {
        strncpy(g_config.app_name, value, sizeof(g_config.app_name) - 1);
    } else if (strcmp(key, "version") == 0) {
        strncpy(g_config.version, value, sizeof(g_config.version) - 1);
    } else if (strcmp(key, "window-width") == 0) {
        g_config.window_width = atoi(value);
    } else if (strcmp(key, "window-height") == 0) {
        g_config.window_height = atoi(value);
    } else if (strcmp(key, "max-connections") == 0) {
        g_config.max_connections = atoi(value);
    } else if (strcmp(key, "timeout") == 0) {
        g_config.timeout = atof(value);
    } else if (strcmp(key, "debug-enabled") == 0) {
        g_config.debug_enabled = (strcmp(value, "true") == 0 || strcmp(value, "#t") == 0);
    } else if (strcmp(key, "log-level") == 0) {
        strncpy(g_config.log_level, value, sizeof(g_config.log_level) - 1);
    } else if (strcmp(key, "data-directory") == 0) {
        strncpy(g_config.data_directory, value, sizeof(g_config.data_directory) - 1);
    } else {
        printf("Warning: Unknown configuration key: %s\n", key);
    }
    
    *result = malloc(1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    (*result)[0] = '\0';
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Add plugin to configuration
 */
static int host_add_plugin(int argc, const char* const* argv, char** result) {
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    if (g_config.plugin_count < 10) {
        size_t len = strlen(argv[0]);
        g_config.plugins[g_config.plugin_count] = malloc(len + 1);
        if (g_config.plugins[g_config.plugin_count]) {
            strcpy(g_config.plugins[g_config.plugin_count], argv[0]);
            g_config.plugin_count++;
        }
    }
    
    *result = malloc(16);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    snprintf(*result, 16, "%d", g_config.plugin_count);
    
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Get environment variable
 */
static int host_getenv(int argc, const char* const* argv, char** result) {
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    const char* env_value = getenv(argv[0]);
    if (!env_value) env_value = "";
    
    size_t len = strlen(env_value);
    *result = malloc(len + 1);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    strcpy(*result, env_value);
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Host function: Check if file exists
 */
static int host_file_exists(int argc, const char* const* argv, char** result) {
    if (argc != 1) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    struct stat st;
    int exists = (stat(argv[0], &st) == 0);
    
    *result = malloc(8);
    if (!*result) return LAMBDUST_MEMORY_ERROR;
    
    strcpy(*result, exists ? "#t" : "#f");
    return LAMBDUST_SUCCESS;
}

/**
 * @brief Initialize configuration system
 */
static LambdustContext* config_init() {
    LambdustContext* ctx = lambdust_create_context();
    if (!ctx) return NULL;
    
    // Set default values
    strcpy(g_config.app_name, "DefaultApp");
    strcpy(g_config.version, "1.0.0");
    g_config.window_width = 800;
    g_config.window_height = 600;
    g_config.max_connections = 100;
    g_config.timeout = 30.0;
    g_config.debug_enabled = 0;
    strcpy(g_config.log_level, "INFO");
    strcpy(g_config.data_directory, "./data");
    g_config.plugin_count = 0;
    
    // Register host functions
    lambdust_register_function(ctx, "set-config!", host_set_config);
    lambdust_register_function(ctx, "add-plugin!", host_add_plugin);
    lambdust_register_function(ctx, "getenv", host_getenv);
    lambdust_register_function(ctx, "file-exists?", host_file_exists);
    
    // Define configuration DSL functions
    lambdust_eval(ctx,
        "(define (app-config name version)"
        "  (set-config! \"app-name\" name)"
        "  (set-config! \"version\" version))", NULL);
    
    lambdust_eval(ctx,
        "(define (window-size width height)"
        "  (set-config! \"window-width\" (number->string width))"
        "  (set-config! \"window-height\" (number->string height)))", NULL);
    
    lambdust_eval(ctx,
        "(define (network max-conn timeout-sec)"
        "  (set-config! \"max-connections\" (number->string max-conn))"
        "  (set-config! \"timeout\" (number->string timeout-sec)))", NULL);
    
    lambdust_eval(ctx,
        "(define (logging level debug?)"
        "  (set-config! \"log-level\" level)"
        "  (set-config! \"debug-enabled\" (if debug? \"true\" \"false\")))", NULL);
    
    lambdust_eval(ctx,
        "(define (data-dir path)"
        "  (set-config! \"data-directory\" path))", NULL);
    
    lambdust_eval(ctx,
        "(define (plugins . plugin-list)"
        "  (for-each add-plugin! plugin-list))", NULL);
    
    // Environment-based configuration
    lambdust_eval(ctx,
        "(define (env-or-default var default-val)"
        "  (let ((env-val (getenv var)))"
        "    (if (string=? env-val \"\") default-val env-val)))", NULL);
    
    // Conditional configuration
    lambdust_eval(ctx,
        "(define (when-file-exists file thunk)"
        "  (if (file-exists? file) (thunk)))", NULL);
    
    return ctx;
}

/**
 * @brief Load configuration from file
 */
static int load_config_file(LambdustContext* ctx, const char* filename) {
    FILE* file = fopen(filename, "r");
    if (!file) {
        printf("Configuration file not found: %s\n", filename);
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
    
    printf("Loading configuration from: %s\n", filename);
    
    // Evaluate configuration
    char* result = NULL;
    int error = lambdust_eval(ctx, content, &result);
    
    if (error == LAMBDUST_SUCCESS) {
        printf("Configuration loaded successfully\n");
        lambdust_free_string(result);
    } else {
        const char* error_msg = lambdust_get_last_error(ctx);
        printf("Configuration load failed: %s\n", error_msg ? error_msg : "Unknown error");
    }
    
    free(content);
    return error == LAMBDUST_SUCCESS ? 0 : -1;
}

/**
 * @brief Create sample configuration file
 */
static void create_sample_config() {
    FILE* f = fopen("config.scm", "w");
    if (!f) return;
    
    fprintf(f,
        ";; Lambdust Application Configuration\n"
        ";; This file demonstrates a flexible, programmable configuration system\n"
        "\n"
        ";; Basic application information\n"
        "(app-config \"MyAwesomeApp\" \"2.1.0\")\n"
        "\n"
        ";; Window configuration\n"
        "(window-size 1024 768)\n"
        "\n"
        ";; Network settings\n"
        "(network 50 45.0)\n"
        "\n"
        ";; Logging configuration\n"
        "(logging \"DEBUG\" #t)\n"
        "\n"
        ";; Data directory (use environment variable if available)\n"
        "(data-dir (env-or-default \"DATA_DIR\" \"./app_data\"))\n"
        "\n"
        ";; Plugin configuration\n"
        "(plugins \"core-plugin\" \"ui-plugin\" \"network-plugin\")\n"
        "\n"
        ";; Conditional configuration based on environment\n"
        "(let ((env (env-or-default \"APP_ENV\" \"development\")))\n"
        "  (cond ((string=? env \"production\")\n"
        "         (logging \"WARN\" #f)\n"
        "         (network 200 60.0))\n"
        "        ((string=? env \"testing\")\n"
        "         (logging \"DEBUG\" #t)\n"
        "         (network 10 5.0))\n"
        "        (else ; development\n"
        "         (logging \"DEBUG\" #t)\n"
        "         (network 5 10.0))))\n"
        "\n"
        ";; Load additional configuration if available\n"
        "(when-file-exists \"local-config.scm\"\n"
        "  (lambda () (load \"local-config.scm\")))\n"
        "\n"
        ";; Configuration validation\n"
        "(if (< (string->number (getenv \"MAX_MEMORY\")) 1000)\n"
        "    (set-config! \"max-connections\" \"10\"))\n"
        "\n"
        ";; Log configuration completion\n"
        "(display \"Configuration loaded successfully\")\n");
    
    fclose(f);
}

/**
 * @brief Print current configuration
 */
static void print_config() {
    printf("\n=== Current Configuration ===\n");
    printf("App Name: %s\n", g_config.app_name);
    printf("Version: %s\n", g_config.version);
    printf("Window: %dx%d\n", g_config.window_width, g_config.window_height);
    printf("Max Connections: %d\n", g_config.max_connections);
    printf("Timeout: %.1f seconds\n", g_config.timeout);
    printf("Debug Enabled: %s\n", g_config.debug_enabled ? "Yes" : "No");
    printf("Log Level: %s\n", g_config.log_level);
    printf("Data Directory: %s\n", g_config.data_directory);
    printf("Plugins (%d):\n", g_config.plugin_count);
    for (int i = 0; i < g_config.plugin_count; i++) {
        printf("  - %s\n", g_config.plugins[i]);
    }
    printf("\n");
}

/**
 * @brief Test dynamic configuration changes
 */
static void test_dynamic_config(LambdustContext* ctx) {
    printf("=== Testing Dynamic Configuration ===\n");
    
    // Test runtime configuration changes
    lambdust_eval(ctx, "(set-config! \"max-connections\" \"500\")", NULL);
    printf("Changed max-connections to 500\n");
    
    lambdust_eval(ctx, "(add-plugin! \"runtime-plugin\")", NULL);
    printf("Added runtime plugin\n");
    
    // Test conditional configuration
    lambdust_eval(ctx,
        "(if (> (string->number (getenv \"USER_LEVEL\")) 5)"
        "    (set-config! \"debug-enabled\" \"true\")"
        "    (set-config! \"debug-enabled\" \"false\"))", NULL);
    
    print_config();
}

/**
 * @brief Cleanup configuration
 */
static void config_cleanup() {
    for (int i = 0; i < g_config.plugin_count; i++) {
        if (g_config.plugins[i]) {
            free(g_config.plugins[i]);
        }
    }
}

int main() {
    printf("=== Lambdust Configuration Management Example ===\n\n");
    
    // Initialize configuration system
    LambdustContext* ctx = config_init();
    if (!ctx) {
        fprintf(stderr, "Failed to initialize configuration system\n");
        return 1;
    }
    
    printf("Configuration system initialized\n");
    printf("Default configuration loaded\n");
    print_config();
    
    // Create and load sample configuration
    create_sample_config();
    load_config_file(ctx, "config.scm");
    print_config();
    
    // Test dynamic configuration
    test_dynamic_config(ctx);
    
    // Cleanup
    config_cleanup();
    lambdust_destroy_context(ctx);
    
    printf("=== Configuration example completed ===\n");
    return 0;
}