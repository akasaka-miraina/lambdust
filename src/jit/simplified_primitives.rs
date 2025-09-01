#![allow(missing_docs)]//! Simplified primitive registration without complex macros.
//! This provides a direct implementation approach for JIT primitives.

use crate::diagnostics::{Result, UnifiedResult};
use crate::eval::Value;
use crate::jit::generic_primitives::{
    ArithmeticOperation, ArithmeticPrimitive, ComparisonOperation, ComparisonPrimitive,
    GenericPrimitiveRegistry, ListOperation, ListPrimitive, TypePredicateOperation,
    TypePredicatePrimitive, UnifiedGenericPrimitive,
};

/// Creates and registers all basic arithmetic primitives
pub fn register_arithmetic_primitives(registry: &mut GenericPrimitiveRegistry) -> Result<()> {
    // Addition
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "+",
        operation: ArithmeticOperation::Add,
    }))?;

    // Subtraction
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "-",
        operation: ArithmeticOperation::Subtract,
    }))?;

    // Multiplication
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "*",
        operation: ArithmeticOperation::Multiply,
    }))?;

    // Division
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "/",
        operation: ArithmeticOperation::Divide,
    }))?;

    // Modulo
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "modulo",
        operation: ArithmeticOperation::Modulo,
    }))?;

    // Absolute value
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "abs",
        operation: ArithmeticOperation::Abs,
    }))?;

    // Min
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "min",
        operation: ArithmeticOperation::Min,
    }))?;

    // Max
    registry.register(UnifiedGenericPrimitive::Arithmetic(ArithmeticPrimitive {
        name: "max",
        operation: ArithmeticOperation::Max,
    }))?;

    Ok(())
}

/// Creates and registers all basic comparison primitives
pub fn register_comparison_primitives(registry: &mut GenericPrimitiveRegistry) -> Result<()> {
    // Equal
    registry.register(UnifiedGenericPrimitive::Comparison(ComparisonPrimitive {
        name: "equal?",
        operation: ComparisonOperation::Equal,
    }))?;

    // Less than
    registry.register(UnifiedGenericPrimitive::Comparison(ComparisonPrimitive {
        name: "<",
        operation: ComparisonOperation::Less,
    }))?;

    // Greater than
    registry.register(UnifiedGenericPrimitive::Comparison(ComparisonPrimitive {
        name: ">",
        operation: ComparisonOperation::Greater,
    }))?;

    // Less than or equal
    registry.register(UnifiedGenericPrimitive::Comparison(ComparisonPrimitive {
        name: "<=",
        operation: ComparisonOperation::LessEqual,
    }))?;

    // Greater than or equal
    registry.register(UnifiedGenericPrimitive::Comparison(ComparisonPrimitive {
        name: ">=",
        operation: ComparisonOperation::GreaterEqual,
    }))?;

    // Numeric equal
    registry.register(UnifiedGenericPrimitive::Comparison(ComparisonPrimitive {
        name: "=",
        operation: ComparisonOperation::NumEqual,
    }))?;

    Ok(())
}

/// Creates and registers all basic list primitives
pub fn register_list_primitives(registry: &mut GenericPrimitiveRegistry) -> Result<()> {
    // Car
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "car",
        operation: ListOperation::Car,
    }))?;

    // Cdr
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "cdr",
        operation: ListOperation::Cdr,
    }))?;

    // Cons
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "cons",
        operation: ListOperation::Cons,
    }))?;

    // List constructor
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "list",
        operation: ListOperation::List,
    }))?;

    // Length
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "length",
        operation: ListOperation::Length,
    }))?;

    // Append
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "append",
        operation: ListOperation::Append,
    }))?;

    // Reverse
    registry.register(UnifiedGenericPrimitive::List(ListPrimitive {
        name: "reverse",
        operation: ListOperation::Reverse,
    }))?;

    Ok(())
}

/// Creates and registers all basic type predicate primitives
pub fn register_type_predicate_primitives(registry: &mut GenericPrimitiveRegistry) -> Result<()> {
    // Number?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "number?",
            operation: TypePredicateOperation::IsNumber,
        },
    ))?;

    // String?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "string?",
            operation: TypePredicateOperation::IsString,
        },
    ))?;

    // Symbol?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "symbol?",
            operation: TypePredicateOperation::IsSymbol,
        },
    ))?;

    // List?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "list?",
            operation: TypePredicateOperation::IsList,
        },
    ))?;

    // Pair?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "pair?",
            operation: TypePredicateOperation::IsPair,
        },
    ))?;

    // Null?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "null?",
            operation: TypePredicateOperation::IsNull,
        },
    ))?;

    // Boolean?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "boolean?",
            operation: TypePredicateOperation::IsBoolean,
        },
    ))?;

    // Procedure?
    registry.register(UnifiedGenericPrimitive::TypePredicate(
        TypePredicatePrimitive {
            name: "procedure?",
            operation: TypePredicateOperation::IsProcedure,
        },
    ))?;

    Ok(())
}

/// Master function to register all primitives
pub fn register_all_primitives(registry: &mut GenericPrimitiveRegistry) -> Result<()> {
    register_arithmetic_primitives(registry)?;
    register_comparison_primitives(registry)?;
    register_list_primitives(registry)?;
    register_type_predicate_primitives(registry)?;
    Ok(())
}
