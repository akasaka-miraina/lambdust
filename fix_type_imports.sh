#!/bin/bash
# Fix ambiguous type imports in the types module

# Fix unified_type_errors.rs
sed -i '' 's/use super::{Type, TypeVar, Substitution, TypeConstraint};/use super::{Type, TypeVar, TypeConstraint};\nuse crate::types::substitution::Substitution;/g' src/types/unified_type_errors.rs

# Fix inference.rs
sed -i '' 's/use super::{Type, TypeVar, TypeScheme, Constraint, Effect, Row, Substitution};/use super::{Type, TypeVar, Constraint, Effect, Row};\nuse crate::types::type_scheme::TypeScheme;\nuse crate::types::substitution::Substitution;/g' src/types/inference.rs

# Fix type_classes.rs
sed -i '' 's/use super::{Type, TypeVar, TypeScheme, Constraint, Effect, Kind};/use super::{Type, TypeVar, Constraint, Effect, Kind};\nuse crate::types::type_scheme::TypeScheme;/g' src/types/type_classes.rs

# Fix algebraic.rs
sed -i '' 's/use super::{Type, TypeVar, TypeScheme, Constraint, Substitution};/use super::{Type, TypeVar, Constraint};\nuse crate::types::type_scheme::TypeScheme;\nuse crate::types::substitution::Substitution;/g' src/types/algebraic.rs

# Fix r7rs_integration.rs
sed -i '' 's/use super::{Type, TypeVar, TypeScheme, Constraint, Effect};/use super::{Type, TypeVar, Constraint, Effect};\nuse crate::types::type_scheme::TypeScheme;/g' src/types/r7rs_integration.rs

# Fix integration_bridge.rs
sed -i '' 's/use super::{Type, TypeVar, TypeScheme, Constraint, Effect};/use super::{Type, TypeVar, Constraint, Effect};\nuse crate::types::type_scheme::TypeScheme;/g' src/types/integration_bridge.rs

# Fix type_env.rs
sed -i '' 's/use super::{TypeScheme, TypeConstructor};/use crate::types::type_scheme::TypeScheme;\nuse crate::types::type_constructor::TypeConstructor;/g' src/types/type_env.rs

echo "Fixed ambiguous type imports"