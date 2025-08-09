# Rust Edition Upgrade Procedure

This document provides a standardized procedure for handling Rust edition upgrades in the Lambdust project, based on the 2024 edition upgrade experience.

## Overview

Rust edition upgrades can introduce significant numbers of compilation errors due to:
- New lints and warnings promoted to errors
- Changes in unsafe code requirements
- Type system refinements
- API deprecations and changes

## Pre-Upgrade Assessment

### 1. Current State Documentation
```bash
# Record current compilation status
cargo check --lib 2>&1 | tee pre-upgrade-status.log
cargo clippy 2>&1 | tee pre-clippy-status.log
```

### 2. Test Suite Baseline
```bash
# Ensure all tests pass before upgrade
cargo test 2>&1 | tee pre-upgrade-tests.log
```

## Upgrade Process

### Phase 1: Edition Update
1. Update `Cargo.toml` edition field
2. **DO NOT** downgrade edition if errors appear - maintain forward compatibility

### Phase 2: Error Assessment and Categorization
```bash
# Get complete error picture
cargo clippy --lib 2>&1 | tee post-upgrade-errors.log

# Categorize errors by type
cargo check --lib 2>&1 | grep "error\[" | cut -d: -f1 | sort | uniq -c | sort -nr
```

### Phase 3: Systematic Resolution

#### Priority Order (Fix in this sequence):

1. **Critical Compilation Errors** (Must fix to proceed)
   - Variable scope issues
   - Missing method/variant definitions
   - Incorrect method signatures

2. **Type System Issues** (Usually highest volume)
   - `Box<Error>` vs `Error` mismatches
   - Optional unwrapping (`Option<T>` vs `T`)
   - Borrowing conflicts

3. **Unsafe Code Updates** (Rust 2024 specific)
   - Add explicit `unsafe` blocks within unsafe functions
   - Raw pointer dereferences
   - FFI calls

4. **API and Lint Updates**
   - Deprecated method calls
   - New clippy lints
   - Warning promotions

#### Resolution Strategies by Error Type:

##### E0308: Type Mismatches (Most Common)
**Pattern**: `expected X, found Y`

**Common Solutions**:
```rust
// Box<Error> vs Error
// OLD: Err(Error::parse_error("msg", span))
// NEW: Err(Error::parse_error("msg", span).boxed())

// Option unwrapping
// OLD: transformer.name.clone()
// NEW: transformer.name.as_ref().unwrap_or(&"default".to_string()).clone()

// Add .cloned() for borrowing issues
// OLD: self.env.lookup(name)
// NEW: self.env.lookup(name).cloned()
```

##### E0599: Missing Methods/Variants
**Pattern**: `no method named X found` or `no variant named X found`

**Solutions**:
1. Add missing constructor methods to enums
2. Add missing variants to enums
3. Add helper methods like `.boxed()` to error types

##### E0133: Unsafe Operations (Rust 2024)
**Pattern**: `unsafe and requires unsafe block`

**Solution**:
```rust
// OLD:
pub unsafe fn from_raw(data: *const u8) -> Self {
    let slice = slice::from_raw_parts(data, size);
}

// NEW:
pub unsafe fn from_raw(data: *const u8) -> Self {
    let slice = unsafe { slice::from_raw_parts(data, size) };
}
```

## Incremental Development Rules

### **CRITICAL: Follow These Rules Throughout**

1. **One Change at a Time**: Never batch large refactoring operations
2. **Immediate Verification**: Run `cargo check --lib` after each significant change
3. **Error Count Tracking**: Must maintain or reduce error count with each step
4. **Zero-Error Requirement**: Never proceed with more errors than you started with

### Quality Standards
- **Development Phase**: `cargo check --lib` showing 0 errors is sufficient
- **Commit Phase**: `cargo clippy` showing 0 errors AND 0 warnings is required

## Common Patterns and Solutions

### 1. Adding Missing Error Helper Methods
```rust
impl SomeError {
    /// Converts this SomeError into a Box<SomeError> for use with Result types.
    pub fn boxed(self) -> Box<SomeError> {
        Box::new(self)
    }
}
```

### 2. Enum Variant Constructor Methods
```rust
impl SomeEnum {
    pub fn variant_name(arg: ArgType) -> Self {
        SomeEnum::VariantName(arg)
    }
    
    pub fn complex_variant(field1: Type1, field2: Type2) -> Self {
        SomeEnum::ComplexVariant {
            field1,
            field2,
        }
    }
}
```

### 3. Handling Unsafe Code in 2024 Edition
```rust
// Wrap all unsafe operations in explicit unsafe blocks
pub unsafe fn some_function() {
    let result = unsafe { some_unsafe_operation() };
    let ptr_deref = unsafe { *some_raw_pointer };
}
```

## Verification Process

### After Each Major Category Fix:
```bash
# Check compilation
cargo check --lib

# Count remaining errors
cargo check --lib 2>&1 | grep -c "error\[" || echo "0 errors"

# Verify specific error types are resolved
cargo check --lib 2>&1 | grep "error\[E0308\]" | wc -l
```

### Final Verification:
```bash
# All checks must pass
cargo clippy                    # 0 errors, 0 warnings
cargo test                      # All tests pass
cargo build --release          # Release build succeeds
```

## Batch Fix Strategies

For large volumes of similar errors (e.g., 1000+ E0308 type mismatches):

### 1. Pattern Recognition
Identify the most common patterns:
```bash
cargo check --lib 2>&1 | grep -A 5 "expected.*found" | head -50
```

### 2. Automated Fixes (Use with caution)
```bash
# Example: Add .boxed() to Error calls
find src -name "*.rs" -exec sed -i 's/Err(Error::/Err(Error::/g' {} \;
```

### 3. Validation After Batch Changes
Always run comprehensive checks after any batch operations.

## Recovery Procedures

### If Errors Increase During Development:
1. **STOP** immediately - do not continue with additional changes
2. Revert the problematic change
3. Try a different approach with smaller scope
4. Verify error count returns to previous level

### If Stuck on Complex Errors:
1. Focus on the highest-volume error types first
2. Create minimal reproduction cases
3. Consult Rust edition migration guides
4. Consider temporary workarounds for non-critical issues

## Documentation Requirements

### During Process:
- Document major pattern changes discovered
- Record common solutions for future reference
- Update this procedure with new findings

### Post-Upgrade:
- Update project documentation for new edition requirements
- Create developer guides for new patterns
- Document any breaking changes for users

## Success Metrics

### Completion Criteria:
- `cargo clippy`: 0 errors, 0 warnings
- `cargo test`: All tests pass
- `cargo build --release`: Succeeds
- No functionality regression

### Progress Tracking:
- Monitor error count reduction
- Track completion percentage
- Document time investment for future planning

## Lessons Learned from 2024 Edition Upgrade

### Key Insights:
1. **Volume**: 1369+ initial errors, 92% were E0308 type mismatches
2. **Categories**: Focus on systematic patterns rather than individual fixes
3. **Efficiency**: Template/Pattern missing methods had high impact
4. **Rust 2024**: Unsafe code requires explicit unsafe blocks in unsafe functions

### Most Effective Approaches:
1. Error type prioritization and batch fixing
2. Adding helper methods (.boxed(), constructors) early
3. Incremental verification prevents compound issues
4. Systematic borrowing issue resolution (.cloned() pattern)

### Time Investment:
- Assessment: ~10% of total time
- Systematic fixes: ~70% of total time  
- Verification and testing: ~20% of total time

## Future Edition Upgrades

This procedure should be updated with new patterns and solutions discovered during future Rust edition upgrades. The systematic approach and incremental development rules should remain constant.