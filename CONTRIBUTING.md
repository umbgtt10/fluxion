# Contributing to Fluxion

Thank you for your interest in contributing to Fluxion! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

- Be respectful and constructive in all interactions
- Focus on what is best for the project and community
- Show empathy towards other community members

## Getting Started

### Prerequisites

- Rust (version specified in `rust-toolchain.toml`)
- Familiarity with async Rust and the tokio ecosystem
- Git for version control

### Setting Up Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/umbgtt10/fluxion.git
   cd fluxion
   ```

2. Build the project:
   ```bash
   cargo build --all
   ```

3. Run tests to ensure everything works:
   ```bash
   cargo test --all-features --all-targets
   ```

4. Run the CI checks locally (Windows):
   ```powershell
   powershell -NoProfile -ExecutionPolicy Bypass -File .\.ci\ci.ps1
   ```

## Development Workflow

### Branch Strategy

- `main` - Stable branch, always in a releasable state
- Feature branches - Use descriptive names like `feature/add-operator-xyz` or `fix/issue-123`

### Making Changes

1. Create a new branch from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following the code style guidelines below

3. Write or update tests for your changes

4. Ensure all tests pass:
   ```bash
   cargo test --all-features --all-targets
   ```

5. Run clippy and fix any warnings:
   ```bash
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   ```

6. Format your code:
   ```bash
   cargo fmt --all
   ```

7. Commit your changes with clear, descriptive messages

8. Push to your fork and create a pull request

## Code Style Guidelines

### General Principles

- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Keep functions focused and single-purpose
- Use meaningful variable and function names
- Prefer explicit types over `impl Trait` in public APIs where clarity is needed

### Documentation

All public items **must** have documentation:

```rust
/// Brief one-line description.
///
/// Detailed explanation of behavior, use cases, and important considerations.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```rust
/// // Runnable example demonstrating usage
/// use fluxion_stream::FluxionStream;
/// let stream = FluxionStream::new(some_stream);
/// ```
///
/// # Errors
///
/// Describe when and why this function might fail (if applicable)
///
/// # Panics
///
/// Describe panic conditions (if applicable)
pub fn function_name() { }
```

### Testing

- Write unit tests for all new functionality
- Use descriptive test names: `test_combine_latest_emits_when_all_streams_ready`
- Include edge cases and error conditions
- Use `#[tokio::test]` for async tests
- Place tests in `tests/` directory for integration tests, or in module for unit tests

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_operator_basic_behavior() {
        // Arrange
        let stream = create_test_stream();
        
        // Act
        let result = stream.operator().collect::<Vec<_>>().await;
        
        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_operator_edge_case() {
        // Test edge cases...
    }
}
```

### Error Handling

- Use `fluxion_error::Result<T>` for fallible operations
- Provide meaningful error context using `FluxionError` variants
- Document all error conditions in function docs
- Avoid panics in library code; return errors instead
- Use `safe_lock` for mutex operations to handle poisoned locks gracefully

### Performance Considerations

- Minimize allocations in hot paths
- Use `Arc` and `Mutex` judiciously
- Prefer stack allocation when possible
- Add benchmarks for performance-critical operators (in `benches/` directory)

## Pull Request Process

### Before Submitting

- [ ] All tests pass (`cargo test --all-features --all-targets`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] No clippy warnings (`cargo clippy --workspace --all-targets --all-features -- -D warnings`)
- [ ] Documentation is complete and accurate
- [ ] Examples are runnable and tested
- [ ] Benchmarks added for performance-sensitive changes (if applicable)

### PR Description Template

```markdown
## Description
Brief description of the changes and motivation

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update

## Testing
Describe the tests you ran and how to reproduce them

## Checklist
- [ ] Tests pass
- [ ] Code formatted
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Examples updated (if applicable)
```

### Review Process

1. Maintainers will review your PR
2. Address any feedback or requested changes
3. Once approved, your PR will be merged

## Project Structure

```
fluxion/
├── fluxion/              # Main crate re-exporting types
├── fluxion-core/         # Core traits and utilities
├── fluxion-error/        # Error types
├── fluxion-stream/       # Stream operators
├── fluxion-exec/         # Execution utilities
├── fluxion-merge/        # Merge operations
├── fluxion-ordered-merge/# Ordered merge implementation
└── fluxion-test-utils/   # Test helpers (dev-only)
```

## Testing Strategy

### Unit Tests

- Test individual functions and methods
- Located in the same file as the code or in `mod tests`
- Use small, focused examples

### Integration Tests

- Test operator combinations and real-world scenarios
- Located in `tests/` directories
- Use `fluxion-test-utils` for test infrastructure

### Benchmarks

- Measure performance of operators
- Located in `benches/` directories
- Run with `cargo bench`

## Continuous Integration

The CI pipeline runs on every PR and includes:

- Building all crates
- Running all tests
- Checking code formatting
- Running clippy with warnings as errors
- Checking documentation builds

Ensure your changes pass all CI checks before requesting review.

## Adding New Stream Operators

When adding a new operator:

1. Create a new file in `fluxion-stream/src/` (e.g., `my_operator.rs`)
2. Define an extension trait with comprehensive documentation
3. Implement the trait for appropriate stream types
4. Add the module to `fluxion-stream/src/lib.rs`
5. Create tests in `fluxion-stream/tests/my_operator_tests.rs`
6. Add benchmarks if performance-critical
7. Update README with examples
8. Add to main `FluxionStream` wrapper if appropriate

## Questions?

- Open an issue for bugs or feature requests
- Check existing issues and PRs before creating new ones
- Tag maintainers for urgent matters

## License

By contributing to Fluxion, you agree that your contributions will be licensed under the Apache-2.0 license.
