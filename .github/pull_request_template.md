# Pull Request

## Description
<!-- Provide a brief description of the changes in this PR -->

## Type of Change
<!-- Mark the relevant option with an "x" -->
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring
- [ ] Test improvements
- [ ] CI/CD improvements

## Related Issues
<!-- Link to any related issues -->
Fixes #(issue number)
Relates to #(issue number)

## Changes Made
<!-- List the main changes made in this PR -->
- 
- 
- 

## Testing
<!-- Describe the testing that has been performed -->
- [ ] Unit tests pass (`cargo test --lib`)
- [ ] Integration tests pass (`cargo test`)
- [ ] R7RS compliance tests pass
- [ ] Performance tests completed
- [ ] Manual testing performed

### Test Results
```
<!-- Paste relevant test output here -->
```

## Code Quality
<!-- Confirm code quality checks -->
- [ ] Code compiles without errors (`cargo check --lib`)
- [ ] Clippy shows no warnings (`cargo clippy`)
- [ ] Code is properly formatted (`cargo fmt`)
- [ ] Documentation is updated where necessary
- [ ] New public APIs are documented

## Performance Impact
<!-- Describe any performance implications -->
- [ ] No performance impact
- [ ] Performance improvement (describe below)
- [ ] Potential performance regression (justify below)

### Performance Details
<!-- If applicable, provide performance test results -->

## Breaking Changes
<!-- If this is a breaking change, describe what breaks and how to migrate -->
- [ ] No breaking changes
- [ ] Breaking changes documented below

### Migration Guide
<!-- If applicable, provide migration instructions -->

## R7RS Compliance
<!-- For language features, confirm R7RS compliance -->
- [ ] Not applicable
- [ ] Maintains R7RS-small compliance
- [ ] Enhances R7RS-large compliance
- [ ] SRFI implementation (specify which)

## Security Considerations
<!-- Describe any security implications -->
- [ ] No security implications
- [ ] Security review required
- [ ] FFI changes require security review
- [ ] Memory safety verified

## Documentation
<!-- Confirm documentation updates -->
- [ ] No documentation changes needed
- [ ] README updated
- [ ] API documentation updated
- [ ] Architecture documentation updated
- [ ] User guide updated

## Checklist
<!-- Final checklist before submission -->
- [ ] I have read the [development guidelines](docs/development/CLAUDE.md)
- [ ] My code follows the project's coding standards
- [ ] I have performed a self-review of my code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] Any dependent changes have been merged and published

## Additional Notes
<!-- Any additional information that would be helpful for reviewers -->

---

### For Reviewers
<!-- This section is for maintainers -->
**Review Focus Areas:**
- [ ] Code quality and style
- [ ] Test coverage
- [ ] Performance impact
- [ ] Security implications
- [ ] R7RS compliance
- [ ] Documentation completeness
- [ ] Breaking change assessment