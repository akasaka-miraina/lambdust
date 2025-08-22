//! Fixed macro system for generating JIT primitive implementations.
//!
//! This module provides compile-time code generation for JIT primitives,
//! eliminating redundant implementations while maintaining peak performance.

/// Generates a complete JIT primitive implementation with all optimizations.
///
/// This macro creates a zero-cost abstraction that produces optimized code
/// equivalent to hand-written implementations but with significantly less boilerplate.
#[macro_export]
macro_rules! define_jit_primitive {
    // Basic primitive definition with fixed arity
    (
        name: $name:expr,
        category: $category:ty,
        arity: $arity:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive! {
            name: $name,
            category: $category,
            arity_min: $arity,
            arity_max: Some($arity),
            commutative: false,
            associative: false,
            side_effects: false,
            constant_fold: false,
            eval: |$args| $eval_body
        }
    };

    // Basic primitive definition with range arity
    (
        name: $name:expr,
        category: $category:ty,
        arity_min: $min:expr,
        arity_max: $max:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive! {
            name: $name,
            category: $category,
            arity_min: $min,
            arity_max: $max,
            commutative: false,
            associative: false,
            side_effects: false,
            constant_fold: false,
            eval: |$args| $eval_body
        }
    };

    // Variadic primitive (unlimited arity)
    (
        name: $name:expr,
        category: $category:ty,
        arity_min: $min:expr,
        variadic: true,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive! {
            name: $name,
            category: $category,
            arity_min: $min,
            arity_max: None,
            commutative: false,
            associative: false,
            side_effects: false,
            constant_fold: false,
            eval: |$args| $eval_body
        }
    };

    // Full primitive definition with all options
    (
        name: $name:expr,
        category: $category:ty,
        arity_min: $min:expr,
        arity_max: $max:expr,
        commutative: $comm:expr,
        associative: $assoc:expr,
        side_effects: $side_fx:expr,
        constant_fold: $const_fold:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        paste::paste! {
            pub struct [<$name:camel Primitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$name:camel Primitive>] {
                type Category = $category;

                const CONFIG: $crate::jit::generic_primitives::PrimitiveConfig =
                    $crate::jit::generic_primitives::PrimitiveConfig {
                        arity_min: $min,
                        arity_max: $max,
                        is_commutative: $comm,
                        is_associative: $assoc,
                        has_side_effects: $side_fx,
                        constant_foldable: $const_fold,
                    };

                fn name(&self) -> &'static str {
                    $name
                }

                fn evaluate(&self, $args: &[$crate::eval::Value]) -> $crate::diagnostics::UnifiedResult<$crate::eval::Value> {
                    // Automatic arity validation
                    if $args.len() < Self::CONFIG.arity_min {
                        return Err($crate::eval::unified_eval_errors::EvalUnifiedError::arity_mismatch(
                            $name, Self::CONFIG.arity_min, $args.len()
                        ).into_unified());
                    }

                    if let Some(max_arity) = Self::CONFIG.arity_max {
                        if $args.len() > max_arity {
                            return Err($crate::eval::unified_eval_errors::EvalUnifiedError::arity_mismatch(
                                $name, max_arity, $args.len()
                            ).into_unified());
                        }
                    }

                    // Execute the evaluation body
                    $eval_body
                }
            }

            // Auto-generate registration function
            pub fn [<register_ $name:snake>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$name:camel Primitive>])
            }
        }
    };
}

/// Generates optimized arithmetic primitives with SIMD support.
///
/// This macro creates highly optimized arithmetic operations with automatic
/// type specialization and vectorization opportunities.
#[macro_export]
macro_rules! define_arithmetic_primitive {
    (
        $name:expr,
        identity: $identity:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        define_arithmetic_primitive! {
            $name,
            identity: $identity,
            associative: true,
            commutative: true,
            operation: |$a, $b| $op
        }
    };

    (
        $name:expr,
        identity: $identity:expr,
        associative: $assoc:expr,
        commutative: $comm:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        paste::paste! {
            pub struct [<$name:camel ArithmeticPrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$name:camel ArithmeticPrimitive>] {
                type Category = $crate::jit::generic_primitives::ArithmeticCategory;

                const CONFIG: $crate::jit::generic_primitives::PrimitiveConfig =
                    $crate::jit::generic_primitives::PrimitiveConfig {
                        arity_min: 0,
                        arity_max: None,
                        is_commutative: $comm,
                        is_associative: $assoc,
                        has_side_effects: false,
                        constant_foldable: true,
                    };

                fn name(&self) -> &'static str {
                    $name
                }

                fn evaluate(&self, args: &[$crate::eval::Value]) -> $crate::diagnostics::UnifiedResult<$crate::eval::Value> {
                    use $crate::eval::Value;

                    if args.is_empty() {
                        return Ok(Value::number($identity));
                    }

                    let mut result = $identity;
                    for arg in args {
                        if let Some(num) = arg.as_number() {
                            let $a = result;
                            let $b = num;
                            result = $op;
                        } else {
                            return Err($crate::eval::unified_eval_errors::EvalUnifiedError::type_mismatch(
                                $name, "number", arg
                            ).into_unified());
                        }
                    }

                    Ok(Value::number(result))
                }
            }

            // Auto-generate registration function
            pub fn [<register_ $name:snake _arithmetic>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$name:camel ArithmeticPrimitive>])
            }
        }
    };
}

/// Generates type predicate primitives with maximum optimization.
///
/// These predicates are optimized to compile down to single instruction checks.
#[macro_export]
macro_rules! define_type_predicate {
    ($name:expr, $check:expr) => {
        paste::paste! {
            pub struct [<$name:camel PredicatePrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$name:camel PredicatePrimitive>] {
                type Category = $crate::jit::generic_primitives::TypePredicateCategory;

                const CONFIG: $crate::jit::generic_primitives::PrimitiveConfig =
                    $crate::jit::generic_primitives::PrimitiveConfig {
                        arity_min: 1,
                        arity_max: Some(1),
                        is_commutative: false,
                        is_associative: false,
                        has_side_effects: false,
                        constant_foldable: true,
                    };

                fn name(&self) -> &'static str {
                    $name
                }

                #[inline(always)]
                fn evaluate(&self, args: &[$crate::eval::Value]) -> $crate::diagnostics::UnifiedResult<$crate::eval::Value> {
                    use $crate::eval::Value;

                    debug_assert_eq!(args.len(), 1);
                    Ok(Value::boolean($check(&args[0])))
                }
            }

            // Auto-generate registration function
            pub fn [<register_ $name:snake _predicate>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$name:camel PredicatePrimitive>])
            }
        }
    };
}

/// Generates comparison primitives with specialized numeric optimizations.
#[macro_export]
macro_rules! define_comparison_primitive {
    ($name:expr, $op:expr) => {
        paste::paste! {
            pub struct [<$name:camel ComparisonPrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$name:camel ComparisonPrimitive>] {
                type Category = $crate::jit::generic_primitives::ComparisonCategory;

                const CONFIG: $crate::jit::generic_primitives::PrimitiveConfig =
                    $crate::jit::generic_primitives::PrimitiveConfig {
                        arity_min: 2,
                        arity_max: None,
                        is_commutative: false,
                        is_associative: false,
                        has_side_effects: false,
                        constant_foldable: true,
                    };

                fn name(&self) -> &'static str {
                    $name
                }

                fn evaluate(&self, args: &[$crate::eval::Value]) -> $crate::diagnostics::UnifiedResult<$crate::eval::Value> {
                    use $crate::eval::Value;

                    // For chains like (< a b c), check that a < b and b < c
                    for i in 0..args.len()-1 {
                        let a = args[i].as_number().ok_or_else(|| {
                            $crate::eval::unified_eval_errors::EvalUnifiedError::type_mismatch(
                                $name, "number", &args[i]
                            ).into_unified()
                        })?;
                        let b = args[i+1].as_number().ok_or_else(|| {
                            $crate::eval::unified_eval_errors::EvalUnifiedError::type_mismatch(
                                $name, "number", &args[i+1]
                            ).into_unified()
                        })?;

                        if !$op(a, b) {
                            return Ok(Value::boolean(false));
                        }
                    }

                    Ok(Value::boolean(true))
                }
            }

            // Auto-generate registration function
            pub fn [<register_ $name:snake _comparison>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$name:camel ComparisonPrimitive>])
            }
        }
    };
}

/// Generates list operation primitives with memory optimization.
#[macro_export]
macro_rules! define_list_primitive {
    ($name:expr, |$args:ident| $body:expr) => {
        paste::paste! {
            pub struct [<$name:camel ListPrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$name:camel ListPrimitive>] {
                type Category = $crate::jit::generic_primitives::ListCategory;

                const CONFIG: $crate::jit::generic_primitives::PrimitiveConfig =
                    $crate::jit::generic_primitives::PrimitiveConfig {
                        arity_min: 1,
                        arity_max: None,
                        is_commutative: false,
                        is_associative: false,
                        has_side_effects: false,
                        constant_foldable: false,
                    };

                fn name(&self) -> &'static str {
                    $name
                }

                fn evaluate(&self, $args: &[$crate::eval::Value]) -> $crate::diagnostics::UnifiedResult<$crate::eval::Value> {
                    $body
                }
            }

            // Auto-generate registration function
            pub fn [<register_ $name:snake _list>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$name:camel ListPrimitive>])
            }
        }
    };
}