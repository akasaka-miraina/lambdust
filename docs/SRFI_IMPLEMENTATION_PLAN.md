# SRFI Implementation Plan for R7RS Small Compliance

## Overview

This document outlines the implementation plan for SRFIs (Scheme Requests for Implementation) that are required for full R7RS Small compliance in Lambdust.

## Required SRFIs for R7RS Small

### SRFI 9: Define-record-type (MANDATORY)
**Status**: 🟡 Planning  
**Priority**: Critical - Required for R7RS Small compliance

#### Description
Defines a syntax for creating record types (structured data types) in Scheme.

#### Syntax
```scheme
(define-record-type <type-name>
  (<constructor> <field-name> ...)
  <predicate>
  (<field-name> <accessor> [<modifier>])
  ...)
```

#### Example Usage
```scheme
(define-record-type point
  (make-point x y)
  point?
  (x point-x set-point-x!)
  (y point-y set-point-y!))

(define p (make-point 3 4))
(point? p)          ; => #t
(point-x p)         ; => 3
(set-point-x! p 5)
(point-x p)         ; => 5
```

#### Implementation Plan
1. **Macro System Extension**: Extend macro expander to handle `define-record-type`
2. **Record Type System**: Create internal representation for record types
3. **Constructor Generation**: Generate constructor procedures
4. **Predicate Generation**: Generate type predicate procedures
5. **Accessor/Mutator Generation**: Generate field access and mutation procedures

#### Technical Challenges
- Record types need to be first-class values
- Type checking at runtime
- Integration with existing value system
- Memory layout optimization

---

### SRFI 45: Primitives for Expressing Iterative Lazy Algorithms (MANDATORY)
**Status**: 🔴 Not Started  
**Priority**: High - Required for R7RS Small compliance

#### Description
Extends the basic `delay` and `force` primitives with additional functionality for iterative lazy algorithms.

#### Key Features
- `lazy` - creates lazy promises
- `delay` - creates eager promises (standard)
- `force` - forces evaluation (standard)
- `eager` - creates eager values

#### Implementation Plan
1. **Promise Value Type**: Extend value system with promise types
2. **Lazy Evaluation Engine**: Implement lazy evaluation mechanism
3. **Force/Delay Implementation**: Standard promise functionality
4. **Iterative Support**: Support for iterative lazy algorithms

---

### SRFI 46: Basic Syntax-rules Extensions (MANDATORY)
**Status**: 🔴 Not Started  
**Priority**: High - Required for R7RS Small compliance

#### Description
Extends the basic `syntax-rules` macro system with additional pattern matching capabilities.

#### Key Features
- Ellipsis after the last pattern variable
- Better error reporting for macro expansion
- Enhanced pattern matching

#### Implementation Plan
1. **Macro Pattern Extension**: Extend pattern matching in macro expander
2. **Ellipsis Handling**: Improve ellipsis pattern support
3. **Error Reporting**: Better error messages for macro failures

---

## Implementation Phases

### Phase 1: SRFI 9 - Define-record-type
**Timeline**: Current Sprint  
**Dependencies**: Macro system, Value system

1. Design record type internal representation
2. Implement macro expansion for `define-record-type`
3. Create constructor generation logic
4. Implement predicate generation
5. Add accessor/mutator generation
6. Add comprehensive tests

### Phase 2: Core System Improvements
**Timeline**: Next Sprint

1. Enhance macro system for SRFI 46
2. Implement promise/lazy evaluation for SRFI 45
3. Integration testing

### Phase 3: Documentation and Testing
**Timeline**: Following Sprint

1. Complete test suites for all SRFIs
2. Performance optimization
3. Documentation updates

## Architecture Considerations

### Integration Points
- **Value System**: New record types need to integrate with existing `Value` enum
- **Macro System**: `define-record-type` is implemented as a macro
- **Environment**: Record type definitions need proper scoping
- **Error System**: Type errors for records need descriptive messages

### Memory Management
- Records should be garbage-collected like other Scheme values
- Field access should be O(1) for performance
- Record types should be shareable across environments

### Performance Targets
- Constructor calls: O(n) where n = number of fields
- Field access: O(1)
- Type checking: O(1)
- Memory overhead: Minimal (one word per field + type tag)

## Testing Strategy

### Unit Tests
- Basic record creation and manipulation
- Type checking behavior
- Error cases (wrong arity, type errors)
- Edge cases (empty records, circular references)

### Integration Tests
- Records in complex data structures
- Records with higher-order functions
- Serialization/deserialization
- Performance benchmarks

### Compliance Tests
- R7RS Small test suite compatibility
- SRFI reference implementation compatibility

## Success Criteria

1. **Functionality**: All SRFI features work as specified
2. **Performance**: No significant performance regression
3. **Compliance**: Passes R7RS Small test suite
4. **Documentation**: Complete user and developer documentation
5. **Maintainability**: Clean, well-documented implementation

## Risk Assessment

### High Risk
- **Macro System Complexity**: `define-record-type` requires sophisticated macro expansion
- **Type System Integration**: Records need to integrate cleanly with existing types

### Medium Risk
- **Performance Impact**: Record operations should not slow down the interpreter
- **Memory Usage**: Records should not cause memory leaks

### Low Risk
- **API Design**: SRFI specifications are well-defined
- **Testing**: Comprehensive test suites available

## Future Considerations

### Beyond R7RS Small
After completing mandatory SRFIs, consider implementing:
- SRFI 1: List Library
- SRFI 13: String Libraries  
- SRFI 69: Basic Hash Tables
- SRFI 111: Boxes
- SRFI 125: Intermediate Hash Tables

### Optimization Opportunities
- Compile-time record type optimization
- Inline field access for known types
- Memory layout optimization for common record patterns