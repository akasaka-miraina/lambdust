/**
 * @file cpp_modern_features.cpp
 * @brief Modern C++ features integration with Lambdust
 * 
 * This example demonstrates advanced C++ features like templates, 
 * smart pointers, concepts (C++20), coroutines, and ranges when
 * integrating with Lambdust.
 */

#include <iostream>
#include <string>
#include <memory>
#include <vector>
#include <optional>
#include <variant>
#include <functional>
#include <algorithm>
#include <numeric>
#include <ranges>
#include <concepts>
#include <future>
#include <thread>
#include <chrono>

extern "C" {
#include "lambdust.h"
}

namespace lambdust::modern {

/**
 * @brief Concept for types that can be converted to Scheme
 */
template<typename T>
concept SchemeConvertible = requires(T t) {
    { std::to_string(t) } -> std::convertible_to<std::string>;
} || std::convertible_to<T, std::string>;

/**
 * @brief Smart pointer wrapper for Lambdust context
 */
using InterpreterPtr = std::unique_ptr<LambdustContext, 
                                      decltype(&lambdust_destroy_context)>;

/**
 * @brief Create a managed Lambdust context
 */
InterpreterPtr make_interpreter() {
    auto* ctx = lambdust_create_context();
    if (!ctx) {
        throw std::runtime_error("Failed to create Lambdust context");
    }
    return InterpreterPtr{ctx, lambdust_destroy_context};
}

/**
 * @brief Result type for Scheme evaluation
 */
using EvalResult = std::variant<std::string, std::runtime_error>;

/**
 * @brief Safe evaluation with error handling
 */
EvalResult safe_eval(const InterpreterPtr& ctx, const std::string& expr) {
    char* result = nullptr;
    int error = lambdust_eval(ctx.get(), expr.c_str(), &result);
    
    if (error == LAMBDUST_SUCCESS) {
        std::string str_result = result ? result : "";
        if (result) lambdust_free_string(result);
        return str_result;
    } else {
        const char* error_msg = lambdust_get_last_error(ctx.get());
        return std::runtime_error(error_msg ? error_msg : "Unknown error");
    }
}

/**
 * @brief Template for type-safe Scheme value conversion
 */
template<SchemeConvertible T>
std::string to_scheme_string(const T& value) {
    if constexpr (std::is_arithmetic_v<T>) {
        return std::to_string(value);
    } else if constexpr (std::convertible_to<T, std::string>) {
        return std::string(value);
    } else {
        return std::to_string(value);
    }
}

/**
 * @brief Variadic template for building Scheme expressions
 */
template<SchemeConvertible... Args>
std::string build_expression(const std::string& function, Args&&... args) {
    std::string expr = "(" + function;
    ((expr += " " + to_scheme_string(std::forward<Args>(args))), ...);
    expr += ")";
    return expr;
}

/**
 * @brief Range-based Scheme list builder
 */
template<std::ranges::range R>
std::string build_scheme_list(const R& range) {
    std::string list = "(list";
    for (const auto& item : range) {
        list += " " + to_scheme_string(item);
    }
    list += ")";
    return list;
}

/**
 * @brief Async Scheme evaluation
 */
std::future<EvalResult> async_eval(const InterpreterPtr& ctx, 
                                  const std::string& expr) {
    return std::async(std::launch::async, [&ctx, expr]() {
        return safe_eval(ctx, expr);
    });
}

/**
 * @brief Functional-style host function registry
 */
class FunctionalRegistry {
public:
    explicit FunctionalRegistry(const InterpreterPtr& ctx) : ctx_(ctx) {}
    
    template<typename F>
    void register_function(const std::string& name, F&& func) {
        // Store the function for lifetime management
        functions_[name] = std::make_unique<std::function<std::string(const std::vector<std::string>&)>>(
            [f = std::forward<F>(func)](const std::vector<std::string>& args) -> std::string {
                return f(args);
            });
        
        // Register with C API (simplified for demonstration)
        // In a real implementation, we'd need proper C function wrappers
    }
    
    template<typename... Args>
    std::optional<std::string> call_if_exists(const std::string& name, Args&&... args) {
        if (auto it = functions_.find(name); it != functions_.end()) {
            std::vector<std::string> arg_strings = {to_scheme_string(std::forward<Args>(args))...};
            return (*it->second)(arg_strings);
        }
        return std::nullopt;
    }
    
private:
    const InterpreterPtr& ctx_;
    std::unordered_map<std::string, 
                      std::unique_ptr<std::function<std::string(const std::vector<std::string>&)>>> functions_;
};

/**
 * @brief RAII Configuration manager
 */
class SchemeConfig {
public:
    explicit SchemeConfig(const InterpreterPtr& ctx) : ctx_(ctx) {}
    
    template<SchemeConvertible T>
    SchemeConfig& set(const std::string& name, T&& value) {
        auto expr = build_expression("define", name, std::forward<T>(value));
        auto result = safe_eval(ctx_, expr);
        
        if (std::holds_alternative<std::runtime_error>(result)) {
            throw std::get<std::runtime_error>(result);
        }
        
        config_[name] = to_scheme_string(std::forward<T>(value));
        return *this;
    }
    
    std::optional<std::string> get(const std::string& name) const {
        if (auto it = config_.find(name); it != config_.end()) {
            return it->second;
        }
        return std::nullopt;
    }
    
    // Range-based iteration
    auto begin() const { return config_.begin(); }
    auto end() const { return config_.end(); }
    
private:
    const InterpreterPtr& ctx_;
    std::unordered_map<std::string, std::string> config_;
};

/**
 * @brief Coroutine-based evaluation (C++20)
 */
#if __cpp_impl_coroutine >= 201902L
#include <coroutine>

struct EvalAwaitable {
    struct promise_type {
        EvalResult result_;
        
        EvalAwaitable get_return_object() { 
            return EvalAwaitable{std::coroutine_handle<promise_type>::from_promise(*this)}; 
        }
        
        std::suspend_never initial_suspend() { return {}; }
        std::suspend_never final_suspend() noexcept { return {}; }
        
        void return_value(EvalResult result) { result_ = std::move(result); }
        void unhandled_exception() { result_ = std::runtime_error("Coroutine exception"); }
    };
    
    std::coroutine_handle<promise_type> coro_;
    
    EvalAwaitable(std::coroutine_handle<promise_type> coro) : coro_(coro) {}
    
    ~EvalAwaitable() { if (coro_) coro_.destroy(); }
    
    EvalResult get_result() {
        return coro_.promise().result_;
    }
};

EvalAwaitable eval_async_coro(const InterpreterPtr& ctx, const std::string& expr) {
    co_return safe_eval(ctx, expr);
}
#endif

} // namespace lambdust::modern

// Demonstration functions
void demonstrate_smart_pointers() {
    std::cout << "=== Smart Pointers and RAII ===\n";
    
    auto interp = lambdust::modern::make_interpreter();
    std::cout << "Created managed interpreter\n";
    
    // The interpreter will be automatically cleaned up
    auto result = lambdust::modern::safe_eval(interp, "(+ 1 2 3)");
    
    if (std::holds_alternative<std::string>(result)) {
        std::cout << "Result: " << std::get<std::string>(result) << "\n";
    } else {
        std::cout << "Error: " << std::get<std::runtime_error>(result).what() << "\n";
    }
    
    std::cout << "Interpreter will be automatically destroyed\n\n";
}

void demonstrate_templates_and_concepts() {
    std::cout << "=== Templates and Concepts ===\n";
    
    auto interp = lambdust::modern::make_interpreter();
    
    // Template-based expression building
    auto expr1 = lambdust::modern::build_expression("+", 10, 20, 30);
    auto expr2 = lambdust::modern::build_expression("*", 3.14, 2);
    auto expr3 = lambdust::modern::build_expression("string-append", "\"Hello\"", "\" \"", "\"World\"");
    
    std::cout << "Generated expressions:\n";
    std::cout << "  " << expr1 << "\n";
    std::cout << "  " << expr2 << "\n";
    std::cout << "  " << expr3 << "\n";
    
    // Evaluate them
    for (const auto& expr : {expr1, expr2, expr3}) {
        auto result = lambdust::modern::safe_eval(interp, expr);
        if (std::holds_alternative<std::string>(result)) {
            std::cout << "  → " << std::get<std::string>(result) << "\n";
        }
    }
    
    std::cout << "\n";
}

void demonstrate_ranges_and_algorithms() {
    std::cout << "=== Ranges and Algorithms ===\n";
    
    auto interp = lambdust::modern::make_interpreter();
    
    // Create range of numbers
    std::vector<int> numbers = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    
    // Build Scheme list from range
    auto scheme_list = lambdust::modern::build_scheme_list(numbers);
    std::cout << "Scheme list from range: " << scheme_list << "\n";
    
    // Filter even numbers using ranges
    auto even_numbers = numbers | std::views::filter([](int n) { return n % 2 == 0; });
    auto even_scheme_list = lambdust::modern::build_scheme_list(even_numbers);
    std::cout << "Even numbers: " << even_scheme_list << "\n";
    
    // Transform and build Scheme expressions
    auto squared_expressions = numbers | std::views::transform([](int n) {
        return lambdust::modern::build_expression("*", n, n);
    });
    
    std::cout << "Square expressions:\n";
    for (const auto& expr : squared_expressions | std::views::take(5)) {
        std::cout << "  " << expr;
        auto result = lambdust::modern::safe_eval(interp, expr);
        if (std::holds_alternative<std::string>(result)) {
            std::cout << " → " << std::get<std::string>(result);
        }
        std::cout << "\n";
    }
    
    std::cout << "\n";
}

void demonstrate_async_evaluation() {
    std::cout << "=== Async Evaluation ===\n";
    
    auto interp = lambdust::modern::make_interpreter();
    
    // Start multiple async evaluations
    std::vector<std::future<lambdust::modern::EvalResult>> futures;
    
    futures.push_back(lambdust::modern::async_eval(interp, "(+ 1 2 3)"));
    futures.push_back(lambdust::modern::async_eval(interp, "(* 4 5 6)"));
    futures.push_back(lambdust::modern::async_eval(interp, "(length '(a b c d e))"));
    
    // Collect results
    std::cout << "Async evaluation results:\n";
    for (auto& future : futures) {
        auto result = future.get();
        if (std::holds_alternative<std::string>(result)) {
            std::cout << "  Result: " << std::get<std::string>(result) << "\n";
        } else {
            std::cout << "  Error: " << std::get<std::runtime_error>(result).what() << "\n";
        }
    }
    
    std::cout << "\n";
}

void demonstrate_configuration_dsl() {
    std::cout << "=== Configuration DSL ===\n";
    
    auto interp = lambdust::modern::make_interpreter();
    lambdust::modern::SchemeConfig config(interp);
    
    // Fluent configuration API
    config
        .set("app-name", "\"Modern C++ App\"")
        .set("version", "\"2.0.0\"")
        .set("max-connections", 100)
        .set("timeout", 30.5)
        .set("debug-mode", "#t");
    
    std::cout << "Configuration set:\n";
    for (const auto& [key, value] : config) {
        std::cout << "  " << key << " = " << value << "\n";
    }
    
    // Query configuration
    if (auto app_name = config.get("app-name")) {
        std::cout << "App name: " << *app_name << "\n";
    }
    
    std::cout << "\n";
}

#if __cpp_impl_coroutine >= 201902L
void demonstrate_coroutines() {
    std::cout << "=== Coroutines (C++20) ===\n";
    
    auto interp = lambdust::modern::make_interpreter();
    
    // Use coroutine for evaluation
    auto awaitable = lambdust::modern::eval_async_coro(interp, "(+ 1 2 3 4 5)");
    auto result = awaitable.get_result();
    
    if (std::holds_alternative<std::string>(result)) {
        std::cout << "Coroutine result: " << std::get<std::string>(result) << "\n";
    } else {
        std::cout << "Coroutine error: " << std::get<std::runtime_error>(result).what() << "\n";
    }
    
    std::cout << "\n";
}
#endif

int main() {
    std::cout << "=== Modern C++ Features with Lambdust ===\n\n";
    
    try {
        demonstrate_smart_pointers();
        demonstrate_templates_and_concepts();
        demonstrate_ranges_and_algorithms();
        demonstrate_async_evaluation();
        demonstrate_configuration_dsl();
        
#if __cpp_impl_coroutine >= 201902L
        demonstrate_coroutines();
#else
        std::cout << "=== Coroutines not available (requires C++20) ===\n\n";
#endif
        
        std::cout << "=== Modern C++ demonstration completed ===\n";
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}