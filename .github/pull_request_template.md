<!-- for GitHub Copilot review rule -->

<details>
<summary>for GitHub Copilot review rule</summary>

- Please provide suggestions for improving code quality in terms of
  - Logic accuracy
  - Security issues
  - Performance optimization
  - Potential data conflicts
  - Maintain consistency
  - Error handling
  - Improved maintainability
  - Ensure modularity
  - Reduce code complexity
  - Optimization possibilities
  - Apply best practices (DRY, SOLID, KISS)
- Focus your review on critical issues (e.g., test failures, vulnerabilities, bottlenecks above O(n²))
- Do not comment on
  - Minor code style issues
  - Lack of comments/documentation
</details>
<!-- for GitHub Copilot review rule -->

## Summary

<!-- Provide a clear and concise description of what this PR implements -->

## Related Issue

<!-- Link the corresponding issue -->
Closes #

## Changes

<!-- Describe the changes made in detail -->

### Added Features

- [ ] Feature 1: Description
- [ ] Feature 2: Description

### Bug Fixes

- [ ] Fix 1: Description
- [ ] Fix 2: Description

### Modified Files

<!-- List major changed files and reasons for changes -->

- `src/file1.rs`: Reason for change
- `src/file2.rs`: Reason for change
- `tests/test_file.rs`: Added new tests

## Testing

### Added Tests

<!-- Describe the new tests added -->

- [ ] Unit test: `test_function_name`
- [ ] Integration test: `test_integration_scenario`
- [ ] Performance test: Execution time measurement

### Test Results

```bash
# Local test execution results
cargo test
# Result: XX passed; 0 failed
```

### Test Coverage

<!-- Describe test coverage for new features -->

- [ ] Happy path tests
- [ ] Error case tests
- [ ] Edge case tests
- [ ] Performance tests (if applicable)

## Checklist

### Implementation

- [ ] Code works correctly
- [ ] Error handling is properly implemented
- [ ] Performance impact has been considered
- [ ] No memory leaks or resource leaks

### Testing

- [ ] Added tests for new features
- [ ] All existing tests pass
- [ ] Test cases properly cover error scenarios

### Documentation

- [ ] Updated CLAUDE.md if necessary
- [ ] Added documentation comments for new APIs

### R7RS Compliance

- [ ] Complies with R7RS specification (if applicable)
- [ ] Complies with SRFI specification (if applicable)

## Performance Impact

<!-- Describe any performance impact -->

- [ ] No performance impact
- [ ] Performance improvement: XX% better
- [ ] Performance degradation: XX% slower (reason: XXX)

## Breaking Changes

<!-- Describe any breaking changes to existing APIs -->

- [ ] No breaking changes
- [ ] Breaking changes present (details: XXX)

## Demo/Screenshots

<!-- Provide demo of new features if applicable -->

```scheme
;; Usage example
(example-code-here)
```

## Additional Information

<!-- Any other information you want to share with reviewers -->

### Design Decisions

<!-- Explain important design decisions -->

### Known Limitations

<!-- Describe any known limitations or future improvements -->

### References

<!-- List any references used during implementation -->

- [R7RS Specification](https://small.r7rs.org/)
- [SRFI Documents](https://srfi.schemers.org/)