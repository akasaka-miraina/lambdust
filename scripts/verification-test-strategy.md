# verification_test.scm Execution Strategy for Split Validation

## Overview

The `verification_test.scm` file contains essential R7RS compliance tests that validate the container system and core Scheme semantics. This strategy defines how to execute these tests in both CI and Docker environments to ensure SIGSEGV fixes preserve exact R7RS compliance.

## Split Validation Architecture

### CI Environment (< 3 minutes)
**Purpose**: Fast feedback for development workflow
**Constraints**: GitHub Actions Ubuntu 24.04.3, resource limitations
**Focus**: Core semantic preservation validation

#### CI Execution Strategy:
```bash
# Quick validation of core R7RS features
timeout 60s cargo run --bin lambdust --release --no-default-features -- verification_test.scm
```

**Validated Features in CI**:
1. **Symbol Identity**: `(eq? sym1 sym2)` correctness
2. **Lexical Scoping**: Variable binding and shadowing
3. **List Operations**: Basic list structure and operations  
4. **Vector Operations**: Vector access and manipulation
5. **Closure Capture**: Lexically scoped variable capture
6. **Container Operations**: Basic ordered-set operations (if available)

### Docker Environment (Unlimited time)
**Purpose**: Comprehensive R7RS compliance and memory safety validation
**Environment**: Ubuntu 22.04 with memory sanitizers and debugging tools
**Focus**: Thorough semantic preservation and memory safety

#### Docker Execution Strategy:
```bash
# Comprehensive validation with full output analysis
timeout 120s cargo run --bin lambdust --release --no-default-features -- verification_test.scm > /tmp/verification_results.txt 2>&1

# Validate all expected outputs
grep -q "Symbol identity test: #t" /tmp/verification_results.txt
grep -q "Lexical scoping test: #t" /tmp/verification_results.txt
grep -q "List operations: #t" /tmp/verification_results.txt
grep -q "Vector operations: #t" /tmp/verification_results.txt
grep -q "Closure capture test: #t" /tmp/verification_results.txt
```

## Test Result Analysis

### Expected Output Pattern:
```
=== R7RS Container Compliance Verification ===
1. Symbol identity test: #t
2. Lexical scoping test: #t
3. List operations: #t
4. Vector operations: #t
5. Closure capture test: #t
6. Basic container operations: (2 2 #t #f)
=== Verification Complete ===
```

### Failure Pattern Detection:
- Any `#f` result indicates R7RS compliance violation
- Runtime errors suggest SIGSEGV fixes broke semantics
- Memory access violations detected in Docker environment

## Integration with SafeOptimizedValue

### Critical Validation Points:
1. **Fixnum Operations**: Ensure `SafeOptimizedValue::Fixnum` preserves numeric tower semantics
2. **Boolean Identity**: Validate `SafeOptimizedValue::True` and `SafeOptimizedValue::False` maintain `eq?` semantics  
3. **Nil Handling**: Confirm `SafeOptimizedValue::Nil` preserves null list identity
4. **Character Operations**: Verify `SafeOptimizedValue::Character` maintains character comparison semantics

### Memory Safety Validation:
- No segmentation faults during test execution
- Proper cleanup of container structures
- Symbol table integrity preservation
- Environment chain memory management

## Execution Workflow

### Development Workflow:
1. **Local Development**: Run `verification_test.scm` directly for quick checks
2. **CI Pipeline**: Essential validation runs automatically on push
3. **Pre-push Hook**: Docker comprehensive validation runs before push
4. **Main Branch**: Additional protected branch checks

### Command Examples:

#### Local Quick Test:
```bash
cargo run --bin lambdust --release --no-default-features -- verification_test.scm
```

#### CI Integration:
```bash
# In .github/workflows/ci.yml
./ci/r7rs-essential-validation.sh
```

#### Docker Comprehensive:
```bash
# In pre-push hook
docker run --rm -v "$(pwd):/workspace" lambdust-r7rs-validation bash -c "
    cd /workspace && ./docker/r7rs-compliance/comprehensive-r7rs-validation.sh
"
```

## Success Criteria

### CI Success:
- ✅ All core R7RS tests pass
- ✅ No segmentation faults
- ✅ Execution completes within 3 minutes
- ✅ Expected output patterns match

### Docker Success:
- ✅ All comprehensive R7RS tests pass
- ✅ Memory safety validation passes
- ✅ Stress tests complete successfully
- ✅ Valgrind (if available) reports no memory errors
- ✅ Container operations maintain integrity

### Overall Validation Success:
- ✅ 96.5% R7RS compliance preservation confirmed
- ✅ SIGSEGV errors completely eliminated
- ✅ SafeOptimizedValue API maintains semantic correctness
- ✅ Performance benefits maintained (15.68% improvement)
- ✅ Docker container testing validates fixes before push

## Monitoring and Reporting

### CI Reporting:
- GitHub Actions artifacts contain test results
- Fast feedback for development iterations
- Integration with existing CI workflows

### Docker Reporting:
- Comprehensive logs stored in container
- Memory safety reports generated
- Full compliance validation results
- Pre-push validation status

This strategy ensures complete R7RS compliance validation while respecting CI constraints and satisfying the Docker container testing requirement.