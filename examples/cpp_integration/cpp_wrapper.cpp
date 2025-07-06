/**
 * @file cpp_wrapper.cpp
 * @brief C++ wrapper class for Lambdust integration
 * 
 * This example demonstrates how to create a modern C++ wrapper around
 * the Lambdust C API, providing RAII, exception safety, and type safety.
 */

#include <iostream>
#include <string>
#include <memory>
#include <stdexcept>
#include <functional>
#include <vector>
#include <unordered_map>
#include <sstream>

extern "C" {
#include "lambdust.h"
}

namespace lambdust {

/**
 * @brief Exception class for Lambdust errors
 */
class LambdustException : public std::runtime_error {
public:
    explicit LambdustException(const std::string& message, int error_code = 0)
        : std::runtime_error(message), error_code_(error_code) {}
    
    int error_code() const noexcept { return error_code_; }

private:
    int error_code_;
};

/**
 * @brief RAII wrapper for C strings returned by Lambdust
 */
class ManagedString {
public:
    explicit ManagedString(char* ptr) : ptr_(ptr) {}
    
    ~ManagedString() {
        if (ptr_) {
            lambdust_free_string(ptr_);
        }
    }
    
    // Move-only semantics
    ManagedString(const ManagedString&) = delete;
    ManagedString& operator=(const ManagedString&) = delete;
    
    ManagedString(ManagedString&& other) noexcept : ptr_(other.ptr_) {
        other.ptr_ = nullptr;
    }
    
    ManagedString& operator=(ManagedString&& other) noexcept {
        if (this != &other) {
            if (ptr_) {
                lambdust_free_string(ptr_);
            }
            ptr_ = other.ptr_;
            other.ptr_ = nullptr;
        }
        return *this;
    }
    
    const char* c_str() const { return ptr_ ? ptr_ : ""; }
    std::string str() const { return std::string(c_str()); }
    
    bool empty() const { return !ptr_ || ptr_[0] == '\0'; }

private:
    char* ptr_;
};

/**
 * @brief Type-safe host function wrapper
 */
template<typename... Args>
class HostFunction {
public:
    using FunctionType = std::function<std::string(Args...)>;
    
    explicit HostFunction(FunctionType func) : func_(std::move(func)) {}
    
    static int c_wrapper(int argc, const char* const* argv, char** result) {
        try {
            auto* self = static_cast<HostFunction*>(
                *reinterpret_cast<void**>(const_cast<char**>(argv) - 1));
            
            if (argc != sizeof...(Args)) {
                return LAMBDUST_ARITY_ERROR;
            }
            
            // Convert arguments and call function
            std::string ret = call_with_converted_args(self->func_, argv, 
                                                     std::index_sequence_for<Args...>{});
            
            // Allocate result
            *result = static_cast<char*>(malloc(ret.length() + 1));
            if (!*result) return LAMBDUST_MEMORY_ERROR;
            
            strcpy(*result, ret.c_str());
            return LAMBDUST_SUCCESS;
            
        } catch (const std::exception&) {
            return LAMBDUST_RUNTIME_ERROR;
        }
    }

private:
    template<std::size_t... I>
    static std::string call_with_converted_args(const FunctionType& func, 
                                               const char* const* argv,
                                               std::index_sequence<I...>) {
        return func(convert_arg<Args>(argv[I])...);
    }
    
    template<typename T>
    static T convert_arg(const char* arg) {
        if constexpr (std::is_same_v<T, std::string>) {
            return std::string(arg);
        } else if constexpr (std::is_same_v<T, int>) {
            return std::stoi(arg);
        } else if constexpr (std::is_same_v<T, double>) {
            return std::stod(arg);
        } else {
            static_assert(!std::is_same_v<T, T>, "Unsupported argument type");
        }
    }
    
    FunctionType func_;
};

/**
 * @brief Modern C++ wrapper for Lambdust interpreter
 */
class Interpreter {
public:
    /**
     * @brief Constructor - creates a new Lambdust context
     */
    Interpreter() : context_(lambdust_create_context()) {
        if (!context_) {
            throw LambdustException("Failed to create Lambdust context");
        }
        
        if (!lambdust_check_library()) {
            throw LambdustException("Lambdust library health check failed");
        }
    }
    
    /**
     * @brief Destructor - automatically cleans up resources
     */
    ~Interpreter() {
        if (context_) {
            lambdust_destroy_context(context_);
        }
    }
    
    // Non-copyable but movable
    Interpreter(const Interpreter&) = delete;
    Interpreter& operator=(const Interpreter&) = delete;
    
    Interpreter(Interpreter&& other) noexcept : context_(other.context_) {
        other.context_ = nullptr;
    }
    
    Interpreter& operator=(Interpreter&& other) noexcept {
        if (this != &other) {
            if (context_) {
                lambdust_destroy_context(context_);
            }
            context_ = other.context_;
            other.context_ = nullptr;
        }
        return *this;
    }
    
    /**
     * @brief Evaluate Scheme expression
     */
    std::string eval(const std::string& expression) {
        char* result = nullptr;
        int error = lambdust_eval(context_, expression.c_str(), &result);
        
        if (error != LAMBDUST_SUCCESS) {
            const char* error_msg = lambdust_get_last_error(context_);
            throw LambdustException(
                error_msg ? error_msg : "Unknown evaluation error", error);
        }
        
        ManagedString managed_result(result);
        return managed_result.str();
    }
    
    /**
     * @brief Register a host function with type safety
     */
    template<typename... Args>
    void register_function(const std::string& name, 
                         std::function<std::string(Args...)> func) {
        auto host_func = std::make_unique<HostFunction<Args...>>(std::move(func));
        
        // Store the function for lifetime management
        host_functions_[name] = std::move(host_func);
        
        int error = lambdust_register_function(context_, name.c_str(), 
                                             HostFunction<Args...>::c_wrapper);
        
        if (error != LAMBDUST_SUCCESS) {
            host_functions_.erase(name);
            throw LambdustException("Failed to register host function: " + name, error);
        }
    }
    
    /**
     * @brief Call a Scheme function with arguments
     */
    template<typename... Args>
    std::string call_function(const std::string& name, Args&&... args) {
        std::ostringstream expr;
        expr << "(" << name;
        
        // Add arguments
        ((expr << " " << format_arg(std::forward<Args>(args))), ...);
        expr << ")";
        
        return eval(expr.str());
    }
    
    /**
     * @brief Get library version
     */
    static std::string version() {
        return std::string(lambdust_get_version());
    }
    
    /**
     * @brief Load and evaluate a Scheme file
     */
    std::string load_file(const std::string& filename) {
        std::ifstream file(filename);
        if (!file) {
            throw LambdustException("Cannot open file: " + filename);
        }
        
        std::string content((std::istreambuf_iterator<char>(file)),
                           std::istreambuf_iterator<char>());
        
        return eval(content);
    }

private:
    LambdustContext* context_;
    std::unordered_map<std::string, std::unique_ptr<void, void(*)(void*)>> host_functions_;
    
    template<typename T>
    std::string format_arg(T&& arg) {
        if constexpr (std::is_same_v<std::decay_t<T>, std::string>) {
            return "\"" + arg + "\"";
        } else if constexpr (std::is_arithmetic_v<std::decay_t<T>>) {
            return std::to_string(arg);
        } else {
            return std::string(arg);
        }
    }
};

} // namespace lambdust

// Example usage and testing
void demonstrate_basic_usage() {
    std::cout << "=== Basic Usage Example ===\n";
    
    try {
        lambdust::Interpreter interp;
        
        std::cout << "Lambdust version: " << lambdust::Interpreter::version() << "\n";
        
        // Basic arithmetic
        std::cout << "Basic arithmetic:\n";
        std::cout << "(+ 1 2 3) = " << interp.eval("(+ 1 2 3)") << "\n";
        std::cout << "(* 6 7) = " << interp.eval("(* 6 7)") << "\n";
        
        // String operations
        std::cout << "\nString operations:\n";
        std::cout << R"((string-append "Hello" ", " "World!") = )" 
                  << interp.eval(R"((string-append "Hello" ", " "World!"))") << "\n";
        
        // Function definitions
        std::cout << "\nFunction definitions:\n";
        interp.eval("(define (square x) (* x x))");
        std::cout << "(square 5) = " << interp.eval("(square 5)") << "\n";
        
    } catch (const lambdust::LambdustException& e) {
        std::cerr << "Lambdust error: " << e.what() << " (code: " << e.error_code() << ")\n";
    }
}

void demonstrate_host_functions() {
    std::cout << "\n=== Host Functions Example ===\n";
    
    try {
        lambdust::Interpreter interp;
        
        // Register host functions with type safety
        interp.register_function<int, int>("cpp-add", 
            [](int a, int b) -> std::string {
                return std::to_string(a + b);
            });
        
        interp.register_function<std::string>("cpp-greet",
            [](const std::string& name) -> std::string {
                return "Hello, " + name + "!";
            });
        
        interp.register_function<double>("cpp-square-root",
            [](double x) -> std::string {
                if (x < 0) throw std::invalid_argument("Negative input");
                return std::to_string(std::sqrt(x));
            });
        
        // Test host functions
        std::cout << "Host function calls:\n";
        std::cout << "(cpp-add 10 20) = " << interp.eval("(cpp-add \"10\" \"20\")") << "\n";
        std::cout << R"((cpp-greet "C++") = )" << interp.eval(R"((cpp-greet "C++"))") << "\n";
        std::cout << "(cpp-square-root 16) = " << interp.eval("(cpp-square-root \"16\")") << "\n";
        
        // Combine with Scheme functions
        interp.eval("(define (hypotenuse a b) (cpp-square-root (+ (* a a) (* b b))))");
        std::cout << "(hypotenuse 3 4) = " << interp.eval("(hypotenuse 3 4)") << "\n";
        
    } catch (const lambdust::LambdustException& e) {
        std::cerr << "Lambdust error: " << e.what() << "\n";
    } catch (const std::exception& e) {
        std::cerr << "Standard error: " << e.what() << "\n";
    }
}

void demonstrate_advanced_features() {
    std::cout << "\n=== Advanced Features Example ===\n";
    
    try {
        lambdust::Interpreter interp;
        
        // Complex data structures
        std::cout << "Complex data structures:\n";
        std::cout << "(list 1 2 3 4 5) = " << interp.eval("(list 1 2 3 4 5)") << "\n";
        std::cout << "(map (lambda (x) (* x x)) '(1 2 3 4)) = " 
                  << interp.eval("(map (lambda (x) (* x x)) '(1 2 3 4))") << "\n";
        
        // Higher-order functions
        std::cout << "\nHigher-order functions:\n";
        interp.eval("(define (apply-twice f x) (f (f x)))");
        interp.eval("(define (increment x) (+ x 1))");
        std::cout << "(apply-twice increment 5) = " 
                  << interp.eval("(apply-twice increment 5)") << "\n";
        
        // Error handling demonstration
        std::cout << "\nError handling:\n";
        try {
            interp.eval("(+ 1 2"); // Missing closing parenthesis
        } catch (const lambdust::LambdustException& e) {
            std::cout << "Caught parsing error: " << e.what() << "\n";
        }
        
    } catch (const lambdust::LambdustException& e) {
        std::cerr << "Lambdust error: " << e.what() << "\n";
    }
}

int main() {
    std::cout << "=== Lambdust C++ Wrapper Demo ===\n\n";
    
    try {
        demonstrate_basic_usage();
        demonstrate_host_functions();
        demonstrate_advanced_features();
        
        std::cout << "\n=== Demo completed successfully ===\n";
        return 0;
        
    } catch (const std::exception& e) {
        std::cerr << "Fatal error: " << e.what() << "\n";
        return 1;
    }
}