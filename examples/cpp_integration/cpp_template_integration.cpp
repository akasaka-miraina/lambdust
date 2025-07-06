/**
 * @file cpp_template_integration.cpp
 * @brief Template-based type-safe integration with Lambdust
 * 
 * This example demonstrates advanced template techniques for creating
 * type-safe, compile-time checked interfaces to Scheme functions.
 */

#include <iostream>
#include <string>
#include <tuple>
#include <type_traits>
#include <functional>
#include <vector>
#include <optional>
#include <sstream>
#include <memory>

extern "C" {
#include "lambdust.h"
}

namespace lambdust::template_integration {

/**
 * @brief Type traits for Scheme type mapping
 */
template<typename T>
struct scheme_type_name;

template<> struct scheme_type_name<int> { static constexpr const char* value = "integer"; };
template<> struct scheme_type_name<double> { static constexpr const char* value = "real"; };
template<> struct scheme_type_name<std::string> { static constexpr const char* value = "string"; };
template<> struct scheme_type_name<bool> { static constexpr const char* value = "boolean"; };

/**
 * @brief Compile-time string for template metaprogramming
 */
template<size_t N>
struct ConstexprString {
    constexpr ConstexprString(const char (&str)[N]) {
        std::copy_n(str, N, value);
    }
    
    char value[N];
    static constexpr size_t size = N - 1;
};

/**
 * @brief Template for converting C++ types to Scheme strings
 */
template<typename T>
std::string to_scheme_value(const T& value) {
    if constexpr (std::is_same_v<T, std::string>) {
        return "\"" + value + "\"";
    } else if constexpr (std::is_same_v<T, bool>) {
        return value ? "#t" : "#f";
    } else if constexpr (std::is_arithmetic_v<T>) {
        return std::to_string(value);
    } else {
        static_assert(std::is_same_v<T, void>, "Unsupported type for Scheme conversion");
    }
}

/**
 * @brief Template for parsing Scheme results to C++ types
 */
template<typename T>
T from_scheme_value(const std::string& str) {
    if constexpr (std::is_same_v<T, std::string>) {
        // Remove quotes if present
        if (str.size() >= 2 && str.front() == '"' && str.back() == '"') {
            return str.substr(1, str.size() - 2);
        }
        return str;
    } else if constexpr (std::is_same_v<T, bool>) {
        return str == "#t" || str == "true";
    } else if constexpr (std::is_same_v<T, int>) {
        return std::stoi(str);
    } else if constexpr (std::is_same_v<T, double>) {
        return std::stod(str);
    } else {
        static_assert(std::is_same_v<T, void>, "Unsupported type for Scheme parsing");
    }
}

/**
 * @brief Template-based function signature validation
 */
template<typename Sig>
struct function_traits;

template<typename R, typename... Args>
struct function_traits<R(Args...)> {
    using return_type = R;
    using argument_types = std::tuple<Args...>;
    static constexpr size_t arity = sizeof...(Args);
    
    template<size_t I>
    using argument_type = std::tuple_element_t<I, argument_types>;
};

/**
 * @brief Type-safe Scheme function wrapper
 */
template<typename Signature>
class TypedSchemeFunction;

template<typename R, typename... Args>
class TypedSchemeFunction<R(Args...)> {
public:
    TypedSchemeFunction(LambdustContext* ctx, const std::string& name) 
        : ctx_(ctx), name_(name) {}
    
    R operator()(Args... args) const {
        // Build function call expression
        std::ostringstream expr;
        expr << "(" << name_;
        
        // Add arguments
        ((expr << " " << to_scheme_value(args)), ...);
        expr << ")";
        
        // Evaluate
        char* result = nullptr;
        int error = lambdust_eval(ctx_, expr.str().c_str(), &result);
        
        if (error != LAMBDUST_SUCCESS) {
            const char* error_msg = lambdust_get_last_error(ctx_);
            throw std::runtime_error(error_msg ? error_msg : "Unknown error");
        }
        
        // Convert result
        std::string result_str = result ? result : "";
        if (result) lambdust_free_string(result);
        
        if constexpr (std::is_same_v<R, void>) {
            return;
        } else {
            return from_scheme_value<R>(result_str);
        }
    }
    
    // Compile-time arity check
    static constexpr size_t arity() { return sizeof...(Args); }
    
    // Compile-time type information
    template<size_t I>
    static constexpr const char* argument_type_name() {
        return scheme_type_name<std::tuple_element_t<I, std::tuple<Args...>>>::value;
    }
    
    static constexpr const char* return_type_name() {
        if constexpr (std::is_same_v<R, void>) {
            return "void";
        } else {
            return scheme_type_name<R>::value;
        }
    }

private:
    LambdustContext* ctx_;
    std::string name_;
};

/**
 * @brief Template-based host function registration
 */
template<typename F>
class HostFunctionWrapper;

template<typename R, typename... Args>
class HostFunctionWrapper<R(Args...)> {
public:
    using FunctionType = std::function<R(Args...)>;
    
    explicit HostFunctionWrapper(FunctionType func) : func_(std::move(func)) {}
    
    static int c_wrapper(int argc, const char* const* argv, char** result) {
        try {
            // Extract the wrapper instance (simplified for demonstration)
            // In a real implementation, we'd need proper context management
            static thread_local HostFunctionWrapper* current_wrapper = nullptr;
            
            if (argc != sizeof...(Args)) {
                return LAMBDUST_ARITY_ERROR;
            }
            
            if (!current_wrapper) {
                return LAMBDUST_RUNTIME_ERROR;
            }
            
            // Convert arguments and call function
            auto converted_args = convert_args(argv, std::index_sequence_for<Args...>{});
            
            if constexpr (std::is_same_v<R, void>) {
                std::apply(current_wrapper->func_, converted_args);
                *result = static_cast<char*>(malloc(1));
                if (*result) (*result)[0] = '\0';
                return LAMBDUST_SUCCESS;
            } else {
                R ret = std::apply(current_wrapper->func_, converted_args);
                std::string ret_str = to_scheme_value(ret);
                
                *result = static_cast<char*>(malloc(ret_str.length() + 1));
                if (!*result) return LAMBDUST_MEMORY_ERROR;
                
                strcpy(*result, ret_str.c_str());
                return LAMBDUST_SUCCESS;
            }
            
        } catch (const std::exception&) {
            return LAMBDUST_RUNTIME_ERROR;
        }
    }

private:
    FunctionType func_;
    
    template<size_t... I>
    static std::tuple<Args...> convert_args(const char* const* argv, std::index_sequence<I...>) {
        return std::make_tuple(from_scheme_value<Args>(argv[I])...);
    }
};

/**
 * @brief Template-based Scheme interface builder
 */
template<typename... Functions>
class SchemeInterface {
public:
    explicit SchemeInterface(LambdustContext* ctx) : ctx_(ctx) {}
    
    template<typename Signature>
    TypedSchemeFunction<Signature> get_function(const std::string& name) {
        return TypedSchemeFunction<Signature>(ctx_, name);
    }
    
    template<typename F>
    void register_host_function(const std::string& name, F&& func) {
        using Traits = function_traits<F>;
        auto wrapper = std::make_unique<HostFunctionWrapper<F>>(std::forward<F>(func));
        
        // Store for lifetime management
        host_functions_[name] = std::move(wrapper);
        
        // Register with Lambdust (simplified)
        lambdust_register_function(ctx_, name.c_str(), HostFunctionWrapper<F>::c_wrapper);
    }

private:
    LambdustContext* ctx_;
    std::unordered_map<std::string, std::unique_ptr<void, void(*)(void*)>> host_functions_;
};

/**
 * @brief Compile-time DSL for Scheme code generation
 */
template<typename... Exprs>
struct SchemeDSL {
    template<ConstexprString name, typename... Args>
    static std::string call(Args&&... args) {
        std::ostringstream expr;
        expr << "(" << name.value;
        ((expr << " " << to_scheme_value(std::forward<Args>(args))), ...);
        expr << ")";
        return expr.str();
    }
    
    template<ConstexprString var, typename T>
    static std::string define(T&& value) {
        return "(define " + std::string(var.value) + " " + to_scheme_value(std::forward<T>(value)) + ")";
    }
    
    template<ConstexprString name, ConstexprString params, ConstexprString body>
    static std::string define_function() {
        return "(define (" + std::string(name.value) + " " + std::string(params.value) + ") " + std::string(body.value) + ")";
    }
};

/**
 * @brief Template-based validation and testing framework
 */
template<typename Signature>
class SchemeTestCase {
public:
    SchemeTestCase(LambdustContext* ctx, const std::string& name) 
        : func_(ctx, name), name_(name) {}
    
    template<typename... Args>
    auto test(Args&&... args) -> typename function_traits<Signature>::return_type {
        static_assert(sizeof...(Args) == function_traits<Signature>::arity, 
                     "Argument count mismatch");
        
        std::cout << "Testing " << name_ << " with arguments: ";
        ((std::cout << to_scheme_value(args) << " "), ...);
        std::cout << "\n";
        
        try {
            auto result = func_(std::forward<Args>(args)...);
            std::cout << "  Result: " << to_scheme_value(result) << "\n";
            return result;
        } catch (const std::exception& e) {
            std::cout << "  Error: " << e.what() << "\n";
            throw;
        }
    }

private:
    TypedSchemeFunction<Signature> func_;
    std::string name_;
};

} // namespace lambdust::template_integration

// Demonstration functions
void demonstrate_typed_functions() {
    std::cout << "=== Typed Function Calls ===\n";
    
    auto* ctx = lambdust_create_context();
    if (!ctx) {
        throw std::runtime_error("Failed to create context");
    }
    
    // Define some Scheme functions
    lambdust_eval(ctx, "(define (add-numbers x y) (+ x y))", nullptr);
    lambdust_eval(ctx, "(define (greet name) (string-append \"Hello, \" name \"!\"))", nullptr);
    lambdust_eval(ctx, "(define (is-positive? x) (> x 0))", nullptr);
    
    // Create typed function wrappers
    using namespace lambdust::template_integration;
    
    auto add_numbers = TypedSchemeFunction<int(int, int)>(ctx, "add-numbers");
    auto greet = TypedSchemeFunction<std::string(std::string)>(ctx, "greet");
    auto is_positive = TypedSchemeFunction<bool(double)>(ctx, "is-positive?");
    
    // Type-safe calls
    std::cout << "add-numbers(10, 20) = " << add_numbers(10, 20) << "\n";
    std::cout << "greet(\"World\") = " << greet("World") << "\n";
    std::cout << "is-positive?(5.5) = " << std::boolalpha << is_positive(5.5) << "\n";
    std::cout << "is-positive?(-3.0) = " << std::boolalpha << is_positive(-3.0) << "\n";
    
    lambdust_destroy_context(ctx);
    std::cout << "\n";
}

void demonstrate_compile_time_dsl() {
    std::cout << "=== Compile-time DSL ===\n";
    
    auto* ctx = lambdust_create_context();
    if (!ctx) {
        throw std::runtime_error("Failed to create context");
    }
    
    using namespace lambdust::template_integration;
    using DSL = SchemeDSL<>;
    
    // Generate Scheme code at compile time
    auto add_expr = DSL::call<"+">(1, 2, 3, 4, 5);
    auto mul_expr = DSL::call<"*">(2, 3, 4);
    auto define_pi = DSL::define<"pi">(3.14159);
    
    std::cout << "Generated expressions:\n";
    std::cout << "  " << add_expr << "\n";
    std::cout << "  " << mul_expr << "\n";
    std::cout << "  " << define_pi << "\n";
    
    // Evaluate them
    char* result = nullptr;
    
    lambdust_eval(ctx, define_pi.c_str(), &result);
    if (result) { lambdust_free_string(result); result = nullptr; }
    
    lambdust_eval(ctx, add_expr.c_str(), &result);
    std::cout << "  → " << (result ? result : "error") << "\n";
    if (result) { lambdust_free_string(result); result = nullptr; }
    
    lambdust_eval(ctx, mul_expr.c_str(), &result);
    std::cout << "  → " << (result ? result : "error") << "\n";
    if (result) { lambdust_free_string(result); result = nullptr; }
    
    lambdust_destroy_context(ctx);
    std::cout << "\n";
}

void demonstrate_template_validation() {
    std::cout << "=== Template Validation ===\n";
    
    auto* ctx = lambdust_create_context();
    if (!ctx) {
        throw std::runtime_error("Failed to create context");
    }
    
    using namespace lambdust::template_integration;
    
    // Define test functions
    lambdust_eval(ctx, "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))", nullptr);
    lambdust_eval(ctx, "(define (string-reverse s) (list->string (reverse (string->list s))))", nullptr);
    
    // Create test cases with compile-time type checking
    auto factorial_test = SchemeTestCase<int(int)>(ctx, "factorial");
    auto reverse_test = SchemeTestCase<std::string(std::string)>(ctx, "string-reverse");
    
    std::cout << "Function testing with compile-time validation:\n";
    
    try {
        factorial_test.test(5);
        factorial_test.test(0);
        factorial_test.test(7);
        
        reverse_test.test("hello");
        reverse_test.test("world");
        
        // This would cause a compile-time error:
        // factorial_test.test("not a number");  // Type mismatch
        // factorial_test.test(1, 2);            // Wrong arity
        
    } catch (const std::exception& e) {
        std::cout << "Test error: " << e.what() << "\n";
    }
    
    lambdust_destroy_context(ctx);
    std::cout << "\n";
}

void demonstrate_metaprogramming() {
    std::cout << "=== Template Metaprogramming ===\n";
    
    using namespace lambdust::template_integration;
    
    // Compile-time function information
    using AddFunc = TypedSchemeFunction<int(int, int)>;
    using GreetFunc = TypedSchemeFunction<std::string(std::string)>;
    
    std::cout << "Function metadata (compile-time):\n";
    std::cout << "  AddFunc arity: " << AddFunc::arity() << "\n";
    std::cout << "  AddFunc return type: " << AddFunc::return_type_name() << "\n";
    std::cout << "  AddFunc arg 0 type: " << AddFunc::argument_type_name<0>() << "\n";
    std::cout << "  AddFunc arg 1 type: " << AddFunc::argument_type_name<1>() << "\n";
    
    std::cout << "  GreetFunc arity: " << GreetFunc::arity() << "\n";
    std::cout << "  GreetFunc return type: " << GreetFunc::return_type_name() << "\n";
    std::cout << "  GreetFunc arg 0 type: " << GreetFunc::argument_type_name<0>() << "\n";
    
    // Template specialization examples
    static_assert(AddFunc::arity() == 2, "AddFunc should have arity 2");
    static_assert(GreetFunc::arity() == 1, "GreetFunc should have arity 1");
    
    std::cout << "\n";
}

int main() {
    std::cout << "=== Template-based Lambdust Integration ===\n\n";
    
    try {
        demonstrate_typed_functions();
        demonstrate_compile_time_dsl();
        demonstrate_template_validation();
        demonstrate_metaprogramming();
        
        std::cout << "=== Template integration demo completed ===\n";
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }
}