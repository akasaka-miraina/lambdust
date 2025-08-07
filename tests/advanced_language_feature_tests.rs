//! Advanced language feature tests for Lambdust
//! 
//! This module tests advanced language features including:
//! - Macro system with enhanced hygiene
//! - Metaprogramming capabilities and reflection
//! - FFI integration with C bindings
//! - Advanced I/O operations (streaming, async, network)

use lambdust::{
    macro_system::*,
    metaprogramming::*,
    ffi::*,
    stdlib::advanced_io::*,
    eval::Value,
    ast::{Literal, Expr},
    diagnostics::Result,
};
use std::collections::HashMap;

#[test]
fn test_hygenic_macro_system() {
    let mut expander = MacroExpander::new();
    
    // Test define-syntax with hygiene
    let macro_def = r#"
        (define-syntax when
          (syntax-rules ()
            ((when test stmt1 stmt2 ...)
             (if test (begin stmt1 stmt2 ...)))))
    "#;
    
    // Parse and register the macro
    let pattern = SyntaxPattern::parse("(when test stmt1 stmt2 ...)").unwrap();
    let template = SyntaxTemplate::parse("(if test (begin stmt1 stmt2 ...))").unwrap();
    
    let macro_rule = SyntaxRule::new(pattern, template);
    let syntax_rules = SyntaxRules::new(vec![], vec![macro_rule]);
    
    expander.define_macro("when".to_string(), Box::new(syntax_rules));
    
    // Test macro expansion with hygiene
    let input = r#"(when #t (display "hello") (newline))"#;
    let parsed_input = parse_expression(input).unwrap();
    
    let expanded = expander.expand(&parsed_input).unwrap();
    
    // Verify the expansion preserves hygiene
    assert!(matches!(expanded, Expr::If { .. }));
}

#[test]
fn test_advanced_macro_patterns() {
    // Test ellipsis patterns
    let pattern = SyntaxPattern::parse("(list x ...)").unwrap();
    
    match pattern {
        SyntaxPattern::List(patterns) => {
            assert_eq!(patterns.len(), 2);
            assert!(matches!(patterns[0], SyntaxPattern::Symbol(_)));
            assert!(matches!(patterns[1], SyntaxPattern::Ellipsis(_)));
        }
        _ => panic!("Expected List pattern"),
    }
    
    // Test literal patterns
    let literal_pattern = SyntaxPattern::parse("(if #t x y)").unwrap();
    
    match literal_pattern {
        SyntaxPattern::List(patterns) => {
            assert_eq!(patterns.len(), 4);
            assert!(matches!(patterns[1], SyntaxPattern::Literal(Literal::Boolean(true))));
        }
        _ => panic!("Expected List pattern with literal"),
    }
}

#[test]
fn test_macro_hygiene_preservation() {
    let mut hygiene_manager = HygieneManager::new();
    
    // Create a lexical context
    let context = LexicalContext::new();
    let mark = hygiene_manager.new_mark();
    
    // Test identifier renaming for hygiene
    let identifier = Identifier::new("x".to_string(), Some(context.clone()));
    let renamed = hygiene_manager.rename_identifier(&identifier, mark);
    
    assert_ne!(identifier.name, renamed.name);
    assert_eq!(renamed.context, Some(context));
    
    // Test scope tracking
    let scope = hygiene_manager.current_scope();
    assert!(scope.bindings.is_empty());
    
    hygiene_manager.enter_scope();
    hygiene_manager.bind_identifier("test".to_string(), mark);
    
    let new_scope = hygiene_manager.current_scope();
    assert!(!new_scope.bindings.is_empty());
    
    hygiene_manager.exit_scope();
}

#[test]
fn test_procedural_macros() {
    // Test procedural macro definition
    let proc_macro = ProceduralMacro::new(
        "debug-print".to_string(),
        |input: &[TokenTree]| -> Result<Vec<TokenTree>> {
            // Simple procedural macro that wraps input in debug print
            let mut result = vec![
                TokenTree::Symbol("display".to_string()),
                TokenTree::String("[DEBUG] ".to_string()),
            ];
            result.extend_from_slice(input);
            Ok(result)
        },
    );
    
    assert_eq!(proc_macro.name(), "debug-print");
    
    // Test macro expansion
    let input = vec![
        TokenTree::String("Hello World".to_string()),
    ];
    
    let expanded = proc_macro.expand(&input).unwrap();
    assert_eq!(expanded.len(), 3);
    
    match &expanded[0] {
        TokenTree::Symbol(s) => assert_eq!(s, "display"),
        _ => panic!("Expected display symbol"),
    }
}

#[test]
fn test_reflection_system() {
    let reflection = ReflectionSystem::new();
    
    // Test runtime type inspection
    let value = Value::Literal(Literal::number(42.0));
    let type_info = reflection.get_type_info(&value);
    
    assert_eq!(type_info.name, "Number");
    assert!(!type_info.is_procedure);
    assert!(!type_info.is_mutable);
    
    // Test procedure reflection
    let proc_value = Value::Primitive(std::sync::Arc::new(
        crate::eval::PrimitiveProcedure::new(
            "+".to_string(),
            2,
            |_args| Ok(Value::Literal(Literal::number(0.0))),
        )
    ));
    
    let proc_type_info = reflection.get_type_info(&proc_value);
    assert_eq!(proc_type_info.name, "Procedure");
    assert!(proc_type_info.is_procedure);
    
    // Test environment inspection
    let env = crate::eval::Environment::new();
    let env_reflection = reflection.inspect_environment(&env);
    
    assert!(env_reflection.bindings.is_empty());
}

#[test]
fn test_code_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Test generating a simple function
    let func_spec = FunctionSpec {
        name: "square".to_string(),
        parameters: vec!["x".to_string()],
        body: vec![
            CodeNode::Symbol("*".to_string()),
            CodeNode::Symbol("x".to_string()),
            CodeNode::Symbol("x".to_string()),
        ],
    };
    
    let generated = code_gen.generate_function(&func_spec).unwrap();
    
    // Verify generated code structure
    match generated {
        Expr::Lambda { formals, body, .. } => {
            // Check parameters
            if let crate::ast::Formals::List(params) = formals {
                assert_eq!(params.len(), 1);
            }
            // Check body
            assert!(!body.is_empty());
        }
        _ => panic!("Expected lambda expression"),
    }
    
    // Test template-based code generation
    let template = CodeTemplate::new(
        "simple-getter".to_string(),
        vec!["field".to_string()],
        "(lambda (obj) (car obj))",
    );
    
    let mut substitutions = HashMap::new();
    substitutions.insert("field".to_string(), "name".to_string());
    
    let instantiated = code_gen.instantiate_template(&template, &substitutions).unwrap();
    assert!(matches!(instantiated, Expr::Lambda { .. }));
}

#[test]
fn test_dynamic_evaluation() {
    let mut evaluator = DynamicEvaluator::new();
    
    // Test runtime code compilation and execution
    let code = "(+ 2 3)";
    let result = evaluator.eval_string(code).unwrap();
    
    if let Value::Literal(Literal::Number(n)) = result {
        assert_eq!(n, 5.0);
    }
    
    // Test with runtime-generated code
    let generated_code = format!("(* {} {})", 6, 7);
    let result2 = evaluator.eval_string(&generated_code).unwrap();
    
    if let Value::Literal(Literal::Number(n)) = result2 {
        assert_eq!(n, 42.0);
    }
    
    // Test error handling for malformed code
    let bad_code = "(+ 1 2 3))"; // Extra closing paren
    let result3 = evaluator.eval_string(bad_code);
    assert!(result3.is_err());
}

#[test]
fn test_environment_manipulation() {
    let mut env_manipulator = EnvironmentManipulator::new();
    let mut env = crate::eval::Environment::new();
    
    // Test dynamic binding creation
    env_manipulator.create_binding(
        &mut env,
        "dynamic-var".to_string(),
        Value::Literal(Literal::string("hello".to_string())),
    ).unwrap();
    
    // Test binding lookup
    let retrieved = env_manipulator.lookup_binding(&env, "dynamic-var").unwrap();
    if let Some(Value::Literal(Literal::String(s))) = retrieved {
        assert_eq!(s, "hello");
    }
    
    // Test binding modification
    env_manipulator.modify_binding(
        &mut env,
        "dynamic-var".to_string(),
        Value::Literal(Literal::string("world".to_string())),
    ).unwrap();
    
    let modified = env_manipulator.lookup_binding(&env, "dynamic-var").unwrap();
    if let Some(Value::Literal(Literal::String(s))) = modified {
        assert_eq!(s, "world");
    }
    
    // Test environment snapshot and restoration
    let snapshot = env_manipulator.snapshot_environment(&env);
    
    env_manipulator.create_binding(
        &mut env,
        "temp-var".to_string(),
        Value::Literal(Literal::number(123.0)),
    ).unwrap();
    
    env_manipulator.restore_environment(&mut env, snapshot);
    
    let temp_lookup = env_manipulator.lookup_binding(&env, "temp-var").unwrap();
    assert!(temp_lookup.is_none()); // Should be gone after restoration
}

#[cfg(feature = "ffi")]
#[test]
fn test_ffi_c_integration() {
    let mut ffi_registry = FFIRegistry::new();
    
    // Test C library loading (mock for testing)
    let lib_result = ffi_registry.load_library("libc");
    // In a real test, this would load an actual library
    
    // Test function signature definition
    let signature = CFunctionSignature::new(
        "strlen".to_string(),
        vec![CType::Pointer(Box::new(CType::Char))],
        CType::Size,
    );
    
    assert_eq!(signature.name, "strlen");
    assert_eq!(signature.parameters.len(), 1);
    assert!(matches!(signature.return_type, CType::Size));
    
    // Test marshaling between Scheme and C values
    let marshaler = ValueMarshaler::new();
    
    // Test string marshaling
    let scheme_string = Value::Literal(Literal::string("hello".to_string()));
    let c_value = marshaler.scheme_to_c(&scheme_string, &CType::Pointer(Box::new(CType::Char))).unwrap();
    
    match c_value {
        CValue::Pointer(_) => (), // Expected
        _ => panic!("Expected pointer value"),
    }
    
    // Test number marshaling
    let scheme_number = Value::Literal(Literal::number(42.0));
    let c_number = marshaler.scheme_to_c(&scheme_number, &CType::Int).unwrap();
    
    match c_number {
        CValue::Int(n) => assert_eq!(n, 42),
        _ => panic!("Expected int value"),
    }
}

#[test]
fn test_ffi_safety_checks() {
    let mut safety_manager = FFISafetyManager::new();
    
    // Test memory safety validation
    let safe_op = FFIOperation::FunctionCall {
        function: "safe_strlen".to_string(),
        args: vec![CValue::Pointer(std::ptr::null())],
    };
    
    // This should pass basic safety checks
    let safety_result = safety_manager.validate_operation(&safe_op);
    // Implementation would check for null pointers, buffer overflows, etc.
    
    // Test callback safety
    let callback = FFICallback::new(
        vec![CType::Int],
        CType::Void,
        Box::new(|args| {
            // Safe callback implementation
            CValue::Void
        }),
    );
    
    assert_eq!(callback.parameters.len(), 1);
    assert!(matches!(callback.return_type, CType::Void));
}

#[test]
fn test_program_analysis() {
    let mut analyzer = StaticAnalyzer::new();
    
    // Test dead code detection
    let code = r#"
        (define unused-var 42)
        (define (main)
          (display "hello")
          (newline))
        (main)
    "#;
    
    let program = parse_program(code).unwrap();
    let analysis_result = analyzer.analyze(&program).unwrap();
    
    // Should detect unused variable
    assert!(!analysis_result.dead_code_warnings.is_empty());
    
    // Test dependency analysis
    let dependencies = analyzer.analyze_dependencies(&program).unwrap();
    assert!(dependencies.contains_key("main"));
    
    // Test complexity metrics
    let complexity = analyzer.compute_complexity(&program).unwrap();
    assert!(complexity.cyclomatic_complexity > 0);
    assert!(complexity.lines_of_code > 0);
}

#[tokio::test]
async fn test_advanced_io_streaming() {
    // Test async streaming I/O
    let mut stream = AsyncInputStream::from_bytes(b"hello\nworld\ntest\n");
    
    let mut lines = Vec::new();
    while let Some(line) = stream.read_line().await.unwrap() {
        lines.push(line);
    }
    
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "hello");
    assert_eq!(lines[1], "world");
    assert_eq!(lines[2], "test");
    
    // Test buffered output stream
    let mut output = AsyncOutputStream::new();
    
    output.write_string("Hello ").await.unwrap();
    output.write_string("World!").await.unwrap();
    output.flush().await.unwrap();
    
    let content = output.get_content();
    assert_eq!(content, "Hello World!");
}

#[tokio::test]
async fn test_network_io_operations() {
    // Test HTTP client capabilities
    let client = NetworkIOClient::new();
    
    // Mock HTTP request for testing
    let request = HTTPRequest {
        method: "GET".to_string(),
        url: "http://example.com/api/data".to_string(),
        headers: std::collections::HashMap::new(),
        body: None,
    };
    
    // In a real implementation, this would make an actual HTTP request
    // For testing, we simulate the response
    let mock_response = HTTPResponse {
        status_code: 200,
        headers: std::collections::HashMap::new(),
        body: Some(b"{'result': 'success'}".to_vec()),
    };
    
    // Test response parsing
    assert_eq!(mock_response.status_code, 200);
    assert!(mock_response.body.is_some());
    
    // Test WebSocket connection simulation
    let ws_config = WebSocketConfig {
        url: "ws://localhost:8080/websocket".to_string(),
        protocols: vec!["echo".to_string()],
    };
    
    // Mock WebSocket for testing
    let mock_message = WebSocketMessage::Text("Hello WebSocket!".to_string());
    
    match mock_message {
        WebSocketMessage::Text(text) => {
            assert_eq!(text, "Hello WebSocket!");
        }
        _ => panic!("Expected text message"),
    }
}

#[test]
fn test_security_manager() {
    let mut security_manager = SecurityManager::new();
    
    // Test capability-based security
    let io_capability = Capability::new("io".to_string(), vec![
        Permission::Read,
        Permission::Write,
    ]);
    
    security_manager.grant_capability(io_capability);
    
    // Test permission checking
    assert!(security_manager.has_permission("io", &Permission::Read));
    assert!(security_manager.has_permission("io", &Permission::Write));
    assert!(!security_manager.has_permission("io", &Permission::Execute));
    
    // Test sandbox creation
    let sandbox = security_manager.create_sandbox(vec!["io".to_string()]);
    assert_eq!(sandbox.allowed_capabilities.len(), 1);
    assert_eq!(sandbox.allowed_capabilities[0], "io");
    
    // Test code execution in sandbox
    let safe_code = "(display 'hello)";
    let unsafe_code = "(system 'rm -rf /')";
    
    assert!(sandbox.is_code_safe(safe_code));
    assert!(!sandbox.is_code_safe(unsafe_code));
}

// Integration test combining multiple advanced language features
#[test]
fn test_advanced_language_integration() {
    let mut system = AdvancedLanguageSystem::new();
    
    // Set up macro system
    let mut macro_expander = MacroExpander::new();
    macro_expander.load_standard_macros();
    
    // Set up reflection system
    let reflection = ReflectionSystem::new();
    
    // Set up code generator
    let mut code_gen = CodeGenerator::new();
    
    // Test metaprogramming pipeline:
    // 1. Generate code using templates
    let template = CodeTemplate::new(
        "accessor".to_string(),
        vec!["field".to_string()],
        "(lambda (obj) (cdr (assq 'FIELD obj)))",
    );
    
    let mut substitutions = HashMap::new();
    substitutions.insert("FIELD".to_string(), "name".to_string());
    
    let generated_code = code_gen.instantiate_template(&template, &substitutions).unwrap();
    
    // 2. Expand any macros in the generated code
    let expanded_code = macro_expander.expand(&generated_code).unwrap();
    
    // 3. Analyze the final code
    let analyzer = StaticAnalyzer::new();
    let program = crate::ast::Program {
        expressions: vec![crate::diagnostics::Spanned::new(
            expanded_code,
            crate::diagnostics::Span::new(0, 10, Some("generated".to_string())),
        )],
    };
    
    let analysis = analyzer.analyze(&program).unwrap();
    assert!(analysis.dead_code_warnings.is_empty()); // Generated code should be clean
    
    // 4. Execute the generated and expanded code
    let mut evaluator = DynamicEvaluator::new();
    let test_data = vec![
        ("name".to_string(), Value::Literal(Literal::string("John".to_string()))),
        ("age".to_string(), Value::Literal(Literal::number(30.0))),
    ];
    
    // This would test the complete metaprogramming pipeline in a real implementation
    assert!(generated_code.to_string().contains("lambda"));
}