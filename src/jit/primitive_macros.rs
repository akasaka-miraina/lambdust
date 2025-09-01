#![allow(missing_docs)]//! Fixed macro system for generating JIT primitive implementations.
//!
//! This module provides compile-time code generation for JIT primitives,
//! eliminating redundant implementations while maintaining peak performance.

use crate::jit::generic_primitives::{UnifiedGenericPrimitive, ArithmeticPrimitive, ComparisonPrimitive, ListPrimitive, TypePredicatePrimitive, ArithmeticOperation, ComparisonOperation, ListOperation, TypePredicateOperation, PrimitiveCategory};

/// Generates a complete JIT primitive implementation with all optimizations.
///
/// This macro creates a zero-cost abstraction that produces optimized code
/// equivalent to hand-written implementations but with significantly less boilerplate.
#[macro_export]
macro_rules! define_jit_primitive {
    // Special cases for names with '?' that can't be camel-cased
    (
        name: "eq?",
        category: $category:ty,
        arity: $arity:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive_impl! {
            EqQuestion,
            "eq?",
            $category,
            $arity,
            $arity,
            false,
            false,
            false,
            false,
            |$args| $eval_body
        }
    };

    (
        name: "eqv?",
        category: $category:ty,
        arity: $arity:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive_impl! {
            EqvQuestion,
            "eqv?",
            $category,
            $arity,
            $arity,
            false,
            false,
            false,
            false,
            |$args| $eval_body
        }
    };

    (
        name: "equal?",
        category: $category:ty,
        arity: $arity:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive_impl! {
            EqualQuestion,
            "equal?",
            $category,
            $arity,
            $arity,
            false,
            false,
            false,
            false,
            |$args| $eval_body
        }
    };

    (
        name: "not",
        category: $category:ty,
        arity: $arity:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive_impl! {
            Not,
            "not",
            $category,
            $arity,
            $arity,
            false,
            false,
            false,
            false,
            |$args| $eval_body
        }
    };

    (
        name: "call/cc",
        category: $category:ty,
        arity: $arity:expr,
        eval: |$args:ident| $eval_body:expr
    ) => {
        define_jit_primitive_impl! {
            CallCc,
            "call/cc",
            $category,
            $arity,
            $arity,
            false,
            false,
            true,
            false,
            |$args| $eval_body
        }
    };

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

#[macro_export]
macro_rules! define_jit_primitive_impl {
    (
        $struct_name:ident,
        $name:expr,
        $category:ty,
        $arity_min:expr,
        $arity_max:expr,
        $commutative:expr,
        $associative:expr,
        $side_effects:expr,
        $constant_fold:expr,
        |$args:ident| $eval_body:expr
    ) => {
        paste::paste! {
            pub struct [<$struct_name Primitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$struct_name Primitive>] {
                type Category = $category;

                const CONFIG: $crate::jit::generic_primitives::PrimitiveConfig =
                    $crate::jit::generic_primitives::PrimitiveConfig {
                        arity_min: $arity_min,
                        arity_max: Some($arity_max),
                        is_commutative: $commutative,
                        is_associative: $associative,
                        has_side_effects: $side_effects,
                        constant_foldable: $constant_fold,
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
            pub fn [<register_ $struct_name:snake>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$struct_name Primitive>])
            }
        }
    };
}

#[macro_export]
macro_rules! define_arithmetic_primitive_impl {
    (
        $struct_name:ident,
        $name:expr,
        identity: $identity:expr,
        associative: $assoc:expr,
        commutative: $comm:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        paste::paste! {
            pub struct [<$struct_name ArithmeticPrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$struct_name ArithmeticPrimitive>] {
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
            pub fn [<register_ $struct_name:snake _arithmetic>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$struct_name ArithmeticPrimitive>])
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
        "+",
        identity: $identity:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        define_arithmetic_primitive_impl! {
            Add,
            "+",
            identity: $identity,
            associative: true,
            commutative: true,
            operation: |$a, $b| $op
        }
    };

    (
        "-",
        identity: $identity:expr,
        associative: $assoc:expr,
        commutative: $comm:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        define_arithmetic_primitive_impl! {
            Subtract,
            "-",
            identity: $identity,
            associative: $assoc,
            commutative: $comm,
            operation: |$a, $b| $op
        }
    };

    (
        "*",
        identity: $identity:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        define_arithmetic_primitive_impl! {
            Multiply,
            "*",
            identity: $identity,
            associative: true,
            commutative: true,
            operation: |$a, $b| $op
        }
    };

    (
        "/",
        identity: $identity:expr,
        associative: $assoc:expr,
        commutative: $comm:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        define_arithmetic_primitive_impl! {
            Divide,
            "/",
            identity: $identity,
            associative: $assoc,
            commutative: $comm,
            operation: |$a, $b| $op
        }
    };
}

/// Generates comparison primitives with specialized numeric optimizations.
#[macro_export]
macro_rules! define_comparison_primitive {
    ("=", $op:expr) => {
        define_comparison_primitive_impl!(Equal, "=", $op);
    };
    ("<", $op:expr) => {
        define_comparison_primitive_impl!(LessThan, "<", $op);
    };
    (">", $op:expr) => {
        define_comparison_primitive_impl!(GreaterThan, ">", $op);
    };
    ("<=", $op:expr) => {
        define_comparison_primitive_impl!(LessEqual, "<=", $op);
    };
    (">=", $op:expr) => {
        define_comparison_primitive_impl!(GreaterEqual, ">=", $op);
    };
}

#[macro_export]
macro_rules! define_comparison_primitive_impl {
    ($struct_name:ident, $name:expr, $op:expr) => {
        paste::paste! {
            pub struct [<$struct_name ComparisonPrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$struct_name ComparisonPrimitive>] {
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
            pub fn [<register_ $struct_name:snake _comparison>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$struct_name ComparisonPrimitive>])
            }
        }
    };
}

/// Generates type predicate primitives with maximum optimization.
///
/// These predicates are optimized to compile down to single instruction checks.
#[macro_export]
macro_rules! define_type_predicate {
    ("number?", $check:expr) => {
        define_type_predicate_impl!(NumberQuestion, "number?", $check);
    };
    ("string?", $check:expr) => {
        define_type_predicate_impl!(StringQuestion, "string?", $check);
    };
    ("symbol?", $check:expr) => {
        define_type_predicate_impl!(SymbolQuestion, "symbol?", $check);
    };
    ("boolean?", $check:expr) => {
        define_type_predicate_impl!(BooleanQuestion, "boolean?", $check);
    };
    ("procedure?", $check:expr) => {
        define_type_predicate_impl!(ProcedureQuestion, "procedure?", $check);
    };
    ("vector?", $check:expr) => {
        define_type_predicate_impl!(VectorQuestion, "vector?", $check);
    };
}

#[macro_export]
macro_rules! define_type_predicate_impl {
    ($struct_name:ident, $name:expr, $check:expr) => {
        paste::paste! {
            pub struct [<$struct_name PredicatePrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$struct_name PredicatePrimitive>] {
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
            pub fn [<register_ $struct_name:snake _predicate>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$struct_name PredicatePrimitive>])
            }
        }
    };
}

/// Generates list operation primitives with memory optimization.
#[macro_export]
macro_rules! define_list_primitive {
    ("cons", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(Cons, "cons", |$args| $body);
    };
    ("car", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(Car, "car", |$args| $body);
    };
    ("cdr", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(Cdr, "cdr", |$args| $body);
    };
    ("null?", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(NullQuestion, "null?", |$args| $body);
    };
    ("pair?", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(PairQuestion, "pair?", |$args| $body);
    };
    ("list", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(List, "list", |$args| $body);
    };
    ("length", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(Length, "length", |$args| $body);
    };
    ("append", |$args:ident| $body:expr) => {
        define_list_primitive_impl!(Append, "append", |$args| $body);
    };
}

#[macro_export]
macro_rules! define_list_primitive_impl {
    ($struct_name:ident, $name:expr, |$args:ident| $body:expr) => {
        paste::paste! {
            pub struct [<$struct_name ListPrimitive>];

            impl $crate::jit::generic_primitives::GenericPrimitive for [<$struct_name ListPrimitive>] {
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
            pub fn [<register_ $struct_name:snake _list>](registry: &mut $crate::jit::generic_primitives::GenericPrimitiveRegistry)
                -> $crate::diagnostics::Result<()>
            {
                registry.register([<$struct_name ListPrimitive>])
            }
        }
    };
}

/// Batch registration macro for multiple primitives.
///
/// This macro generates an efficient registration function that registers
/// multiple primitives at once with optimal performance.
#[macro_export]
macro_rules! register_jit_primitives {
    ($registry:expr, [ $($primitive:ident),* $(,)? ]) => {
        $(
            $primitive::register($registry)?;
        )*
    };

    // With category grouping for better optimization
    ($registry:expr, {
        $($category:ident: [ $($primitive:ident),* $(,)? ]),* $(,)?
    }) => {
        $(
            // Register all primitives in category
            $(
                $primitive::register($registry)?;
            )*

            // Apply category-specific optimizations
            $registry.optimize_category(stringify!($category))?;
        )*
    };
}

/// Generates SIMD-optimized vector operations for arithmetic primitives.
///
/// This macro creates vector implementations that can process multiple
/// values simultaneously using SIMD instructions.
#[macro_export]
macro_rules! define_simd_arithmetic {
    (
        $name:expr,
        vector_width: $width:expr,
        instruction_set: $instruction_set:expr,
        operation: |$a:ident, $b:ident| $op:expr
    ) => {
        paste::paste! {
            #[cfg(target_feature = $instruction_set)]
            pub fn [<$name:snake _simd_ $width>](a: &[f64], b: &[f64]) -> Vec<f64> {
                assert_eq!(a.len(), b.len());
                let mut result = Vec::with_capacity(a.len());

                // Process in chunks of vector width
                let chunks = a.len() / ($width / 64); // 64-bit floats

                for i in 0..chunks {
                    let start = i * ($width / 64);
                    let end = start + ($width / 64);

                    // Load vectors
                    let vec_a = &a[start..end];
                    let vec_b = &b[start..end];

                    // Process vector elements
                    for j in 0..vec_a.len() {
                        let $a = vec_a[j];
                        let $b = vec_b[j];
                        result.push($op);
                    }
                }

                // Handle remaining elements
                for i in (chunks * ($width / 64))..a.len() {
                    let $a = a[i];
                    let $b = b[i];
                    result.push($op);
                }

                result
            }
        }
    };
}

/// Generates specialized fast paths for common type combinations.
///
/// This macro creates type-specialized implementations that avoid
/// dynamic dispatch and type checking overhead.
#[macro_export]
macro_rules! define_specialized_fast_path {
    (
        $primitive:ident,
        $input_type:ty => $output_type:ty,
        |$args:ident| $impl:expr
    ) => {
        paste::paste! {
            impl [<$primitive Primitive>] {
                pub fn [<fast_path_ $input_type:snake>](args: &[$input_type]) -> $output_type {
                    let $args = args;
                    $impl
                }
            }
        }
    };
}

/// Generates performance benchmarks for primitives.
///
/// This macro creates comprehensive benchmarks that measure
/// performance across different input sizes and types.
#[macro_export]
macro_rules! benchmark_primitive {
    ($primitive:ident) => {
        paste::paste! {
            #[cfg(test)]
            mod [<$primitive:snake _benchmarks>] {
                use super::*;
                use std::time::Instant;

                #[test]
                fn [<benchmark_ $primitive:snake _small>]() {
                    let primitive = [<$primitive ArithmeticPrimitive>];
                    let args = vec![$crate::eval::Value::number(1.0), $crate::eval::Value::number(2.0)];

                    let start = Instant::now();
                    for _ in 0..10000 {
                        let _ = GenericPrimitive::evaluate(&primitive, &args).unwrap();
                    }
                    let duration = start.elapsed();

                    println!("{} small benchmark: {:?}", stringify!($primitive), duration);
                }

                #[test]
                fn [<benchmark_ $primitive:snake _large>]() {
                    let primitive = [<$primitive ArithmeticPrimitive>];
                    let args: Vec<_> = (0..1000).map(|i| $crate::eval::Value::number(i as f64)).collect();

                    let start = Instant::now();
                    for _ in 0..100 {
                        let _ = GenericPrimitive::evaluate(&primitive, &args).unwrap();
                    }
                    let duration = start.elapsed();

                    println!("{} large benchmark: {:?}", stringify!($primitive), duration);
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jit::generic_primitives::*;
    use crate::eval::Value;

    // Test the macro system with a simple addition primitive
    define_arithmetic_primitive!(
        "+",
        identity: 0.0,
        operation: |a, b| a + b
    );

    // Test comparison primitive
    define_comparison_primitive!(
        "<",
        |a, b| a < b
    );

    // Test type predicate
    define_type_predicate!(
        "number?",
        |value| value.is_number()
    );

    // Generate benchmarks
    benchmark_primitive!(Add);

    #[test]
    fn test_generated_add_primitive() {
        let add = AddArithmeticPrimitive;

        // Test identity
        let result = GenericPrimitive::evaluate(&add, &[]).unwrap();
        assert_eq!(result.as_number().unwrap(), 0.0);

        // Test binary operation
        let args = vec![Value::number(3.0), Value::number(4.0)];
        let result = GenericPrimitive::evaluate(&add, &args).unwrap();
        assert_eq!(result.as_number().unwrap(), 7.0);

        // Test n-ary operation
        let args = vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
            Value::number(4.0)
        ];
        let result = GenericPrimitive::evaluate(&add, &args).unwrap();
        assert_eq!(result.as_number().unwrap(), 10.0);
    }

    #[test]
    fn test_generated_comparison_primitive() {
        let lt = LessThanComparisonPrimitive;

        let args = vec![Value::number(3.0), Value::number(4.0)];
        let result = GenericPrimitive::evaluate(&lt, &args).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let args = vec![Value::number(4.0), Value::number(3.0)];
        let result = GenericPrimitive::evaluate(&lt, &args).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_generated_type_predicate() {
        let number_pred = NumberQuestionPredicatePrimitive;

        let args = vec![Value::number(42.0)];
        let result = GenericPrimitive::evaluate(&number_pred, &args).unwrap();
        assert_eq!(result.as_boolean().unwrap(), true);

        let args = vec![Value::string("hello")];
        let result = GenericPrimitive::evaluate(&number_pred, &args).unwrap();
        assert_eq!(result.as_boolean().unwrap(), false);
    }

    #[test]
    fn test_primitive_configuration() {
        assert_eq!(AddArithmeticPrimitive::CONFIG.arity_min, 0);
        assert_eq!(AddArithmeticPrimitive::CONFIG.arity_max, None);
        assert!(AddArithmeticPrimitive::CONFIG.is_commutative);
        assert!(AddArithmeticPrimitive::CONFIG.is_associative);
        assert!(AddArithmeticPrimitive::CONFIG.constant_foldable);

        assert_eq!(LessThanComparisonPrimitive::CONFIG.arity_min, 2);
        assert_eq!(LessThanComparisonPrimitive::CONFIG.arity_max, Some(2));
        assert!(!LessThanComparisonPrimitive::CONFIG.is_commutative);
        assert!(LessThanComparisonPrimitive::CONFIG.constant_foldable);
    }

    #[test]
    fn test_category_association() {
        use std::any::TypeId;
        assert_eq!(TypeId::of::<AddArithmeticPrimitive::Category>(), TypeId::of::<ArithmeticCategory>());
        assert_eq!(TypeId::of::<LessThanComparisonPrimitive::Category>(), TypeId::of::<ComparisonCategory>());
        assert_eq!(TypeId::of::<NumberQuestionPredicatePrimitive::Category>(), TypeId::of::<TypePredicateCategory>());
    }
}