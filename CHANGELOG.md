# Changelog

All notable changes to the Fluxion project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive documentation for all public APIs
- `CONTRIBUTING.md` with development guidelines
- `CHANGELOG.md` for tracking version history
- Enhanced README examples with runnable code
- Detailed operator documentation for `fluxion-stream`
- Detailed subscription documentation for `fluxion-exec`

### Changed
- Consolidated license to Apache-2.0 (removed MIT dual-licensing)
- Improved README structure with quick start guide
- Enhanced crate-level documentation

### Fixed
- Cleaned up TestChannel vs FluxionChannel terminology inconsistencies

## [0.1.0] - 2025-11-15

### Added
- Initial release of Fluxion reactive streaming library
- Core crates: `fluxion`, `fluxion-core`, `fluxion-stream`, `fluxion-exec`
- Support crates: `fluxion-error`, `fluxion-merge`, `fluxion-ordered-merge`, `fluxion-test-utils`

#### fluxion-core
- `Ordered` trait for temporal ordering
- `OrderedWrapper` implementation
- `CompareByInner` trait for inner value comparisons
- `IntoStream` trait for stream conversion
- Lock utilities with poison recovery (`safe_lock`, `try_lock`)

#### fluxion-stream
- Stream operators with ordering guarantees:
  - `combine_latest` - Combine multiple streams with latest values
  - `with_latest_from` - Sample secondary stream on primary emissions
  - `ordered_merge` - Merge streams preserving temporal order
  - `take_latest_when` - Sample on filter condition
  - `take_while_with` - Conditional emission with termination
  - `emit_when` - Gate source stream with filter stream
  - `combine_with_previous` - Pair consecutive values
- `FluxionStream` wrapper type
- Integration with tokio streams

#### fluxion-exec
- `subscribe_async` - Sequential async stream processing
- `subscribe_latest_async` - Latest-value processing with cancellation
- Cancellation token support
- Error handling callbacks

#### fluxion-error
- `FluxionError` enum with comprehensive error variants
- `Result<T>` type alias
- Error conversion traits and utilities
- Helper methods for error creation

#### fluxion-test-utils
- `TestChannel` for imperative test setup
- `Sequenced<T>` wrapper for ordered test data
- Test data fixtures (Person, Animal, Plant)
- Assertion helpers

### Documentation
- Root README with project overview
- Per-crate READMEs with usage examples
- API documentation for public types and functions
- Architecture example demonstrating design patterns
- Refactoring plan for error handling improvements

### Infrastructure
- Cargo workspace with multiple crates
- CI configuration (GitHub Actions)
- Local CI helper scripts (PowerShell)
- Benchmark suite for performance testing
- Comprehensive test coverage

---

## Version Numbering

Fluxion follows Semantic Versioning (SemVer):

- **MAJOR** version for incompatible API changes
- **MINOR** version for added functionality in a backward-compatible manner  
- **PATCH** version for backward-compatible bug fixes

## Unreleased Changes

Changes that are committed to `main` but not yet released will be documented under `[Unreleased]` at the top of this file.

## Release Process

1. Update version numbers in all `Cargo.toml` files
2. Update `CHANGELOG.md` with release notes
3. Create a git tag for the version
4. Publish crates to crates.io (when ready)
5. Create GitHub release with changelog

---

[Unreleased]: https://github.com/umbgtt10/fluxion/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.0
