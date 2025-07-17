# Development Documentation

This directory contains documentation for developers working on the Lambdust Scheme interpreter.

## 📋 Contents

- **[DEVELOPMENT_FLOW.md](DEVELOPMENT_FLOW.md)**: Development workflow, work procedures, quality checks, and quality management policies
- **[TESTING.md](TESTING.md)**: Test structure and testing strategy
- **[CODE_HYGIENIC_TASK.md](CODE_HYGIENIC_TASK.md)**: Code hygiene tasks and quality policy implementation

## 🎯 Development Principles

### Quality Management Policies
1. **"Fix, don't hide"**: Address linter/compiler warnings at the root cause for Warning Free achievement
2. **"Fix implementation, not tests"**: When tests fail, fix the implementation to prevent technical regression and maintain product quality first

### Development Workflow
- **Environment-First Architecture**: Always create `Arc<Environment>` first and share across components
- **Three-Layer Evaluator**: Use appropriate evaluator (SemanticEvaluator/RuntimeExecutor/EvaluatorInterface) with automatic fallback
- **Copy-on-Write Optimization**: Leverage COW environment for memory efficiency
- **Hygienic Macro System**: Maintain symbol hygiene in macro development

## 🔧 Essential Commands

### Development Workflow
```bash
# Quick development check
make dev-check              # fmt + lint + test

# Full CI check  
make ci-check               # fmt-check + lint + test + coverage + doc

# Individual operations
make fmt                    # Format code
make lint                   # Run clippy with warnings as errors
make test                   # Run all tests + doctests
make coverage-open          # Generate and open coverage report
```

### Testing Strategy
```bash
# Run all tests
cargo test --all-features

# Run specific test categories
cargo test --test integration_tests
cargo test unit::evaluator
cargo test semantic_evaluator_tests

# Performance testing
cargo bench
```

## 🧪 Quality Assurance

### Test Coverage
- **Unit Tests**: 564+ tests covering individual components
- **Integration Tests**: Comprehensive evaluator and system integration tests  
- **Performance Tests**: Benchmarks and regression detection
- **Compliance Tests**: R7RS Small test suite compatibility

### Code Quality Metrics
- **Warning Free**: Zero compiler/clippy warnings policy
- **Test Coverage**: High coverage with meaningful tests
- **Performance**: No regression policy with benchmark verification
- **Documentation**: Comprehensive inline and external documentation

## 📊 Architecture Guidelines

### Module Organization
```
src/
├── evaluator/          # Three-layer evaluation system
├── environment/        # COW environment management
├── macros/            # Hygienic macro system
├── type_system/       # Polynomial Universe type system
├── value/             # Optimized value representations
└── bridge/            # Rust-Scheme interoperability
```

### Development Patterns
- **Error Handling**: Use descriptive error messages with context
- **Memory Management**: Leverage RAII and Arc/Rc for shared ownership
- **Performance**: Profile before optimizing, measure improvements
- **Testing**: Write tests first for new features, maintain existing test suite

## 🔗 Related Documentation

- **Core**: See [../core/](../core/) for project overview and architecture
- **Implementation**: See [../implementation/](../implementation/) for technical specifications
- **User**: See [../user/](../user/) for build and usage instructions
- **Research**: See [../research/](../research/) for theoretical background

## 📈 Contribution Workflow

1. **Setup**: Follow build instructions in [../user/BUILD_COMMANDS.md](../user/BUILD_COMMANDS.md)
2. **Development**: Read `DEVELOPMENT_FLOW.md` for workflow and quality policies
3. **Testing**: Use `TESTING.md` for test strategy and execution
4. **Quality**: Follow `CODE_HYGIENIC_TASK.md` for code hygiene standards
5. **Integration**: Ensure all tests pass and no warnings before committing

## 🏆 Achievement Standards

- **Functionality**: All features work as specified
- **Performance**: No significant performance regression
- **Quality**: Warning-free code with high test coverage
- **Documentation**: Complete user and developer documentation
- **Maintainability**: Clean, well-documented, idiomatic code