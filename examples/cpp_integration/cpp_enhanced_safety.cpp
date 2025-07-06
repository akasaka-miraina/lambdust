/**
 * @file cpp_enhanced_safety.cpp
 * @brief Enhanced safety features demonstration for Lambdust C++ integration
 * 
 * This example demonstrates the enhanced safety features of the Lambdust
 * C FFI interface including memory tracking, thread safety, resource limits,
 * and advanced error handling.
 */

#include <iostream>
#include <string>
#include <memory>
#include <thread>
#include <chrono>
#include <vector>
#include <atomic>

extern "C" {
#include "lambdust.h"
#include "lambdust_enhanced.h"
}

namespace lambdust::enhanced_safety {

/**
 * @brief RAII wrapper for enhanced Lambdust context
 */
class SafeInterpreter {
public:
    explicit SafeInterpreter(size_t max_memory = LAMBDUST_DEFAULT_MEMORY_LIMIT, 
                           uint32_t max_time = LAMBDUST_DEFAULT_TIMEOUT_MS) 
        : context_(lambdust_create_sandboxed_context(max_memory, max_time)) {
        if (!context_) {
            throw std::runtime_error("Failed to create sandboxed context");
        }
        
        // Set up error callback
        lambdust_set_error_callback(context_, error_callback, this);
        
        // Verify context health
        int health = lambdust_check_context_health(context_);
        if (health != LAMBDUST_SUCCESS) {
            lambdust_destroy_context(context_);
            context_ = nullptr;
            throw std::runtime_error("Context health check failed");
        }
    }
    
    ~SafeInterpreter() {
        if (context_) {
            // Clear sensitive data before destruction
            lambdust_clear_sensitive_data(context_);
            lambdust_destroy_context(context_);
        }
    }
    
    // Non-copyable but movable
    SafeInterpreter(const SafeInterpreter&) = delete;
    SafeInterpreter& operator=(const SafeInterpreter&) = delete;
    
    SafeInterpreter(SafeInterpreter&& other) noexcept 
        : context_(other.context_), last_error_(std::move(other.last_error_)) {
        other.context_ = nullptr;
    }
    
    SafeInterpreter& operator=(SafeInterpreter&& other) noexcept {
        if (this != &other) {
            if (context_) {
                lambdust_clear_sensitive_data(context_);
                lambdust_destroy_context(context_);
            }
            context_ = other.context_;
            last_error_ = std::move(other.last_error_);
            other.context_ = nullptr;
        }
        return *this;
    }
    
    /**
     * @brief Safe evaluation with timeout
     */
    std::string eval_safe(const std::string& code, uint32_t timeout_ms = 5000) {
        if (!context_) {
            throw std::runtime_error("Context not available");
        }
        
        // Check context health before evaluation
        int health = lambdust_check_context_health(context_);
        if (health != LAMBDUST_SUCCESS) {
            throw std::runtime_error("Context health check failed: " + 
                                   std::string(LAMBDUST_ERROR_CATEGORY(health)));
        }
        
        char* result = nullptr;
        int error = lambdust_eval_with_timeout(context_, code.c_str(), &result, timeout_ms);
        
        if (error != LAMBDUST_SUCCESS) {
            handle_error(error);
            return "";
        }
        
        std::string result_str = result ? result : "";
        if (result) {
            lambdust_free_tracked(context_, result);
        }
        
        return result_str;
    }
    
    /**
     * @brief Get memory usage statistics
     */
    struct MemoryStats {
        size_t total_allocated;
        size_t peak_usage;
        uint64_t allocation_count;
    };
    
    MemoryStats get_memory_stats() const {
        MemoryStats stats{};
        if (context_) {
            lambdust_get_memory_stats(context_, 
                                    &stats.total_allocated,
                                    &stats.peak_usage, 
                                    &stats.allocation_count);
        }
        return stats;
    }
    
    /**
     * @brief Get last error message
     */
    std::string get_last_error() const {
        return last_error_;
    }
    
    /**
     * @brief Register enhanced host function
     */
    template<typename UserData>
    void register_safe_function(const std::string& name, 
                               LambdustEnhancedHostFunction func,
                               UserData* user_data = nullptr,
                               bool thread_safe = true) {
        if (!context_) {
            throw std::runtime_error("Context not available");
        }
        
        int error = lambdust_register_function_enhanced(
            context_, name.c_str(), func, 
            static_cast<void*>(user_data), thread_safe);
        
        if (error != LAMBDUST_SUCCESS) {
            handle_error(error);
        }
    }

private:
    LambdustContext* context_;
    mutable std::string last_error_;
    
    static void error_callback(LambdustContext* context, int error_code, 
                              const char* error_message, void* user_data) {
        auto* self = static_cast<SafeInterpreter*>(user_data);
        if (self && error_message) {
            self->last_error_ = std::string("Error ") + std::to_string(error_code) + 
                               ": " + error_message;
        }
    }
    
    void handle_error(int error_code) {
        int code;
        const char* message;
        const char* location;
        
        lambdust_get_detailed_error(context_, &code, &message, &location);
        
        std::string error_msg = "Lambdust error " + std::to_string(error_code);
        if (message) {
            error_msg += ": " + std::string(message);
        }
        if (location) {
            error_msg += " at " + std::string(location);
        }
        
        throw std::runtime_error(error_msg);
    }
};

/**
 * @brief Thread-safe context manager
 */
class ThreadSafeManager {
public:
    explicit ThreadSafeManager(size_t num_contexts = std::thread::hardware_concurrency()) {
        for (size_t i = 0; i < num_contexts; ++i) {
            contexts_.emplace_back(std::make_unique<SafeInterpreter>());
        }
    }
    
    /**
     * @brief Execute code in thread-safe manner
     */
    std::string execute(const std::string& code) {
        std::lock_guard<std::mutex> lock(mutex_);
        auto& ctx = contexts_[current_context_++ % contexts_.size()];
        return ctx->eval_safe(code);
    }
    
    /**
     * @brief Get aggregated memory statistics
     */
    SafeInterpreter::MemoryStats get_total_memory_stats() const {
        std::lock_guard<std::mutex> lock(mutex_);
        SafeInterpreter::MemoryStats total{};
        
        for (const auto& ctx : contexts_) {
            auto stats = ctx->get_memory_stats();
            total.total_allocated += stats.total_allocated;
            total.peak_usage += stats.peak_usage;
            total.allocation_count += stats.allocation_count;
        }
        
        return total;
    }

private:
    std::vector<std::unique_ptr<SafeInterpreter>> contexts_;
    mutable std::mutex mutex_;
    std::atomic<size_t> current_context_{0};
};

} // namespace lambdust::enhanced_safety

// Enhanced host function example
extern "C" int enhanced_math_function(int argc, const char* const* argv, 
                                    char** result, void* user_data) {
    if (argc != 2) {
        return LAMBDUST_ARITY_ERROR;
    }
    
    try {
        double a = std::stod(argv[0]);
        double b = std::stod(argv[1]);
        double res = a * a + b * b; // x² + y²
        
        std::string result_str = std::to_string(res);
        *result = static_cast<char*>(malloc(result_str.length() + 1));
        if (!*result) {
            return LAMBDUST_MEMORY_ERROR;
        }
        
        strcpy(*result, result_str.c_str());
        return LAMBDUST_SUCCESS;
        
    } catch (const std::exception&) {
        return LAMBDUST_TYPE_ERROR;
    }
}

// Demonstration functions
void demonstrate_basic_safety() {
    std::cout << "=== Basic Safety Features ===\n";
    
    try {
        lambdust::enhanced_safety::SafeInterpreter interp;
        
        // Test basic evaluation
        auto result = interp.eval_safe("(+ 1 2 3)");
        std::cout << "Basic evaluation: " << result << "\n";
        
        // Check memory stats
        auto stats = interp.get_memory_stats();
        std::cout << "Memory stats:\n";
        std::cout << "  Total allocated: " << stats.total_allocated << " bytes\n";
        std::cout << "  Peak usage: " << stats.peak_usage << " bytes\n";
        std::cout << "  Allocation count: " << stats.allocation_count << "\n";
        
        // Test enhanced host function
        interp.register_safe_function("sum-of-squares", enhanced_math_function);
        result = interp.eval_safe(R"((sum-of-squares "3" "4"))");
        std::cout << "Enhanced function result: " << result << "\n";
        
    } catch (const std::exception& e) {
        std::cerr << "Safety test error: " << e.what() << "\n";
    }
    
    std::cout << "\n";
}

void demonstrate_thread_safety() {
    std::cout << "=== Thread Safety ===\n";
    
    try {
        lambdust::enhanced_safety::ThreadSafeManager manager(4);
        
        // Launch multiple threads
        std::vector<std::thread> threads;
        std::vector<std::string> results(8);
        
        for (int i = 0; i < 8; ++i) {
            threads.emplace_back([&manager, &results, i]() {
                try {
                    std::string code = "(* " + std::to_string(i + 1) + " " + 
                                      std::to_string(i + 1) + ")";
                    results[i] = manager.execute(code);
                } catch (const std::exception& e) {
                    results[i] = "Error: " + std::string(e.what());
                }
            });
        }
        
        // Wait for all threads
        for (auto& thread : threads) {
            thread.join();
        }
        
        // Print results
        for (size_t i = 0; i < results.size(); ++i) {
            std::cout << "Thread " << i << " result: " << results[i] << "\n";
        }
        
        // Get total memory stats
        auto total_stats = manager.get_total_memory_stats();
        std::cout << "Total memory stats across all contexts:\n";
        std::cout << "  Total allocated: " << total_stats.total_allocated << " bytes\n";
        std::cout << "  Peak usage: " << total_stats.peak_usage << " bytes\n";
        std::cout << "  Allocation count: " << total_stats.allocation_count << "\n";
        
    } catch (const std::exception& e) {
        std::cerr << "Thread safety test error: " << e.what() << "\n";
    }
    
    std::cout << "\n";
}

void demonstrate_error_handling() {
    std::cout << "=== Advanced Error Handling ===\n";
    
    try {
        lambdust::enhanced_safety::SafeInterpreter interp;
        
        // Test various error conditions
        std::vector<std::string> test_cases = {
            "(+ 1 2 3)",                    // Valid
            "(+ 1 2",                       // Syntax error
            "(undefined-function)",         // Undefined function
            "(/ 1 0)",                      // Division by zero
            "(make-string -1)",             // Invalid argument
        };
        
        for (const auto& test : test_cases) {
            std::cout << "Testing: " << test << "\n";
            try {
                auto result = interp.eval_safe(test, 1000); // 1 second timeout
                std::cout << "  Result: " << result << "\n";
            } catch (const std::exception& e) {
                std::cout << "  Error: " << e.what() << "\n";
                std::cout << "  Last error: " << interp.get_last_error() << "\n";
            }
        }
        
    } catch (const std::exception& e) {
        std::cerr << "Error handling test error: " << e.what() << "\n";
    }
    
    std::cout << "\n";
}

void demonstrate_resource_limits() {
    std::cout << "=== Resource Limits ===\n";
    
    try {
        // Create context with strict limits
        constexpr size_t small_memory_limit = 1024 * 1024; // 1MB
        constexpr uint32_t short_timeout = 500; // 500ms
        
        lambdust::enhanced_safety::SafeInterpreter limited_interp(
            small_memory_limit, short_timeout);
        
        // Test memory limit
        std::cout << "Testing memory limits...\n";
        try {
            // This might hit memory limits
            auto result = limited_interp.eval_safe("(make-vector 1000000 0)");
            std::cout << "Large allocation succeeded: " << result.substr(0, 50) << "...\n";
        } catch (const std::exception& e) {
            std::cout << "Memory limit hit (expected): " << e.what() << "\n";
        }
        
        // Test timeout
        std::cout << "Testing timeout limits...\n";
        try {
            // This might hit timeout limits
            auto result = limited_interp.eval_safe(
                "(define (slow-func n) (if (> n 0) (slow-func (- n 1)) 0)) (slow-func 10000)",
                100); // Very short timeout
            std::cout << "Long computation succeeded: " << result << "\n";
        } catch (const std::exception& e) {
            std::cout << "Timeout hit (expected): " << e.what() << "\n";
        }
        
    } catch (const std::exception& e) {
        std::cerr << "Resource limit test error: " << e.what() << "\n";
    }
    
    std::cout << "\n";
}

int main() {
    std::cout << "=== Lambdust Enhanced Safety Demo ===\n\n";
    
    try {
        demonstrate_basic_safety();
        demonstrate_thread_safety();
        demonstrate_error_handling();
        demonstrate_resource_limits();
        
        std::cout << "=== Enhanced safety demo completed ===\n";
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "Fatal error: " << e.what() << "\n";
        return 1;
    }
}