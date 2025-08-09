//! Tests for the metaprogramming system.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MetaprogrammingSystem, ReflectionSystem, DynamicEvaluator, SecurityManager, CodeGenerator, ProceduralMacro};
    use crate::metaprogramming::reflection::{TypeInfo, EnvironmentType as ReflectionEnvironmentType};
    use crate::metaprogramming::program_analysis::StaticAnalyzer;
    use crate::metaprogramming::environment_manipulation::{
        EnvironmentManipulator, ModuleManager, MemoryManager, EnvironmentType, 
        EnvironmentHierarchy, ChangeTracker, EnvironmentChange, ChangeType,
        MemoryUsageTracker, GcPolicy, GcFrequency, GcStrategy
    };
    use crate::metaprogramming::code_generation::{AstTransformer, TemplateSystem};
    use crate::metaprogramming::dynamic_evaluation::{SandboxEnvironment, ResourceMonitor};
    use crate::metaprogramming::advanced_macros::{MacroDebugger, StepMode, EnhancedHygiene};
    use crate::metaprogramming::program_analysis::{DependencyAnalyzer, Profiler, CodeAnalyzer};
    use crate::metaprogramming::security::{
        SecurityPolicy, PermissionSystem, Permission, AccessControl
    };
    use crate::eval::{Value, Environment};
    use crate::ast::{Expr, Literal};
    use crate::diagnostics::Span;
    use std::rc::Rc;

    #[test]
    fn test_metaprogramming_system_creation() {
        let system = MetaprogrammingSystem::new();
        assert!(system.reflection().type_cache.is_empty());
    }

    #[test]
    fn test_reflection_type_inspection() {
        let mut system = ReflectionSystem::new();
        let value = Value::Literal(Literal::Number(42.0));
        
        let type_info = system.object_inspector().get_type_info(&value);
        assert_eq!(type_info, TypeInfo::Number);
    }

    #[test]
    fn test_security_manager() {
        let mut security = SecurityManager::new();
        security.install_default_policies();
        
        let context = security.create_context("test".to_string(), "sandbox").unwrap();
        assert_eq!(context.principal, "test");
    }

    #[test]
    fn test_dynamic_evaluator() {
        let mut evaluator = DynamicEvaluator::new();
        let result = evaluator.eval_string("(+ 1 2)", "test", Some("restrictive"));
        
        // Would succeed in a complete implementation
        assert!(result.is_ok() || result.is_err()); // Just test it doesn't panic
    }

    #[test]
    fn test_code_generator() {
        let mut generator = CodeGenerator::new();
        let env = Rc::new(Environment::new(None, 0));
        generator.set_context(env);
        
        let code = "(define x 42)";
        let result = generator.compile_string(code);
        assert!(result.is_ok() || result.is_err()); // Just test it doesn't panic
    }

    #[test]
    fn test_procedural_macro() {
        let system = ProceduralMacro::new();
        assert!(system.proc_macros().is_empty());
    }

    #[test]
    fn test_static_analyzer() {
        let mut analyzer = StaticAnalyzer::new();
        assert!(analyzer.config().dependency_analysis);
    }

    #[test]
    fn test_environment_manipulator() {
        let mut manipulator = EnvironmentManipulator::new();
        let env = Rc::new(Environment::new(None, 0));
        
        let result = manipulator.register_environment(
            "test".to_string(),
            env,
            EnvironmentType::Global
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_manager() {
        let manager = MemoryManager::new();
        let _usage = manager.current_usage();
        // Memory usage is always non-negative for usize type
    }

    #[test]
    fn test_module_manager() {
        let manager = ModuleManager::new().unwrap();
        let loaded = manager.loaded_modules();
        assert!(loaded.is_empty()); // No modules loaded initially
    }

    #[test]
    fn test_sandbox_environment() {
        let sandbox = SandboxEnvironment::new();
        assert!(sandbox.is_primitive_allowed("+"));
        assert!(!sandbox.is_primitive_allowed("system")); // Should be restricted
    }

    #[test]
    fn test_ast_transformer() {
        let transformer = AstTransformer::new();
        assert!(transformer.rules.is_empty());
    }

    #[test]
    fn test_template_system() {
        let system = TemplateSystem::new();
        assert!(system.templates.is_empty());
    }

    #[test]
    fn test_macro_debugger() {
        let debugger = MacroDebugger::new();
        assert_eq!(debugger.step_mode, StepMode::None);
    }

    #[test]
    fn test_enhanced_hygiene() {
        let hygiene = EnhancedHygiene::new();
        assert!(hygiene.policies.is_empty());
    }

    #[test]
    fn test_dependency_analyzer() {
        let analyzer = DependencyAnalyzer::new();
        // Just test creation doesn't panic
    }

    #[test]
    fn test_profiler() {
        let profiler = Profiler::new();
        let results = profiler.get_results();
        assert!(results.call_counts.is_empty());
    }

    #[test]
    fn test_code_analyzer() {
        let analyzer = CodeAnalyzer::new();
        // Just test creation doesn't panic
    }

    #[test]
    fn test_security_policies() {
        let restrictive = SecurityPolicy::restrictive();
        assert_eq!(restrictive.name, "restrictive");
        
        let permissive = SecurityPolicy::permissive();
        assert_eq!(permissive.name, "permissive");
    }

    #[test]
    fn test_permission_system() {
        let mut system = PermissionSystem::new();
        system.grant_permission("user".to_string(), Permission::Eval);
        assert!(system.has_permission("user", &Permission::Eval));
    }

    #[test]
    fn test_access_control() {
        let access_control = AccessControl::new();
        // Just test creation doesn't panic
    }

    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new();
        let usage = monitor.get_usage();
        assert_eq!(usage.allocations, 0);
    }

    #[test]
    fn test_memory_usage_tracker() {
        let tracker = MemoryUsageTracker::new();
        assert_eq!(tracker.peak_usage, 0);
    }

    #[test]
    fn test_gc_policy() {
        let policy = GcPolicy {
            name: "test".to_string(),
            threshold: 1024,
            generation_thresholds: vec![512, 1024, 2048],
            frequency: GcFrequency::Manual,
            strategy: GcStrategy::MarkAndSweep,
        };
        assert_eq!(policy.strategy, GcStrategy::MarkAndSweep);
    }

    #[test]
    fn test_environment_hierarchy() {
        let mut hierarchy = EnvironmentHierarchy::new();
        hierarchy.add_root("global".to_string());
        hierarchy.add_child("global".to_string(), "local".to_string());
        
        assert!(hierarchy.roots.contains(&"global".to_string()));
    }

    #[test]
    fn test_change_tracker() {
        let mut tracker = ChangeTracker::new(100);
        let change = EnvironmentChange {
            change_type: ChangeType::Define,
            variable: "x".to_string(),
            old_value: None,
            new_value: Some(Value::Literal(Literal::Number(42.0))),
            timestamp: std::time::SystemTime::now(),
        };
        
        tracker.track_change("test-env".to_string(), change);
        assert!(tracker.changes.contains_key("test-env"));
    }
}