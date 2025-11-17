# Changelog

All notable changes to the Fluxion project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Error Handling**: Comprehensive error propagation through `StreamItem<T>` enum in all operators
- **Documentation**: Added `docs/ERROR-HANDLING.md` - complete error handling guide with patterns and examples
- **Documentation**: Added `# Errors` sections to all 9 stream operators with links to error handling guide
- **API**: Implemented `CompareByInner` trait for `StreamItem<T>` to enable `with_latest_from` operator
- **Core**: New `fluxion-core::StreamItem<T>` enum for error propagation (`Value(T)` | `Error(FluxionError)`)

### Changed
- **BREAKING**: All stream operators now return `StreamItem<T>` instead of bare `T` values
  - `combine_latest` → `Stream<Item = StreamItem<OrderedWrapper<CombinedState<T>>>>`
  - `with_latest_from` → `Stream<Item = StreamItem<OrderedWrapper<R>>>`
  - `take_latest_when` → `Stream<Item = StreamItem<T>>`
  - `emit_when` → `Stream<Item = StreamItem<T>>`
  - `take_while_with` → `Stream<Item = StreamItem<T::Inner>>`
  - `combine_with_previous` → `Stream<Item = StreamItem<WithPrevious<T>>>`
  - `map_ordered` → `Stream<Item = StreamItem<U>>`
  - `filter_ordered` → `Stream<Item = StreamItem<T>>`
  - `ordered_merge` → `Stream<Item = StreamItem<T>>`
- **BREAKING**: `fluxion-error::FluxionError` now implements `Clone` trait
- **BREAKING**: Simplified `fluxion-error::FluxionError` from 12 variants to 4 actually-used variants
  - Removed: `ChannelSendError`, `ChannelReceiveError`, `CallbackPanic`, `SubscriptionError`, `InvalidState`, `Timeout`, `UnexpectedStreamEnd`, `ResourceLimitExceeded`
  - Kept: `LockError`, `StreamProcessingError`, `UserError`, `MultipleErrors`
- **Code Quality**: Simplified `std::` imports across codebase (added targeted `use` statements)
- **Documentation**: Updated `docs/ERROR-HANDLING.md` to reflect simplified error variants
- **Documentation**: Updated operator documentation to remove references to deleted error variants
- **Operators**: Lock errors now propagate as `StreamItem::Error` instead of silently dropping items
- **API**: Standardized all operators to return `impl Stream<Item = StreamItem<...>>` (removed `FluxionStream` wrapper inconsistency)
- **API**: Removed redundant `FluxionStream::from_stream()` method (use `::new()` instead)
- **Documentation**: Enhanced `filter_ordered` documentation with comprehensive examples and use cases
- **Documentation**: Updated README.md to reference error handling guide in features and guides sections
- **Tests**: Updated all 186 tests across workspace to handle `StreamItem<T>` wrapper

### Removed
- **Documentation**: Removed `docs/REFACTORING_PLAN.md` (implementation complete, details preserved in git history)

### Fixed
- **Error Handling**: Lock poisoning errors no longer cause silent data loss
- **API Consistency**: `take_while_with` now matches other operators' return type pattern

### Migration Guide

**Before (v0.1.x):**
```rust
let mut stream = stream1.combine_latest(vec![stream2], |_| true);
let result = stream.next().await.unwrap();
let value = result.get();
```

**After (v0.2.0):**
```rust
let mut stream = stream1.combine_latest(vec![stream2], |_| true);
let item = stream.next().await.unwrap();
match item {
    StreamItem::Value(result) => {
        let value = result.get();
        // Process value
    }
    StreamItem::Error(e) => {
        eprintln!("Stream error: {}", e);
        // Handle error
    }
}
```

**Quick migration for tests:**
```rust
// Add .unwrap() to extract values when errors are not expected
let value = stream.next().await.unwrap().unwrap();
```

See [Error Handling Guide](docs/ERROR-HANDLING.md) for comprehensive patterns.

## [0.1.1] - 2025-11-16

### Added
- **Documentation**: Added `docs/FLUXION_OPERATOR_SUMMARY.md` - comprehensive operator reference guide
- **Documentation**: Added `docs/FLUXION_OPERATORS_ROADMAP.md` - planned operators and priorities
- **Documentation**: Added `docs/REFACTORING_PLAN.md` - error handling implementation plan
- **Examples**: Added chaining examples to README showing real-world operator composition
- **Tests**: Created `tests/combine_latest_tests.rs` with functional and composition examples
- **Integration**: Integrated `stream-aggregation` example into workspace members
- **API Docs**: Added comprehensive documentation to all FluxionStream extension methods
- **Code of Conduct**: Added Kingdom of Heaven inspired community guidelines

### Changed
- **README**: Restructured with improved organization and table of contents
- **README**: Enhanced examples section with links to test files for verification
- **README**: Added "Chaining Multiple Operators" section with working code examples
- **CI**: Updated `build.ps1` to include stream-aggregation example in build/test pipeline
- **Formatting**: Fixed clippy warnings and formatting inconsistencies across codebase
- **Imports**: Cleaned up unused imports and resolved import warnings
- **Documentation**: Updated all references from `TestChannel` to `tokio::sync::mpsc`

### Fixed
- Resolved beginner-level API usage mistakes in examples
- Fixed code formatting to match rustfmt standards
- Corrected import statements throughout test files

### Known Issues
- **Dependencies**: Harmless duplicate `windows-sys` versions (0.60.2 and 0.61.2) from tokio ecosystem - does not affect functionality

## [0.1.0] - 2025-11-16

### Added
- First release

---

[Unreleased]: https://github.com/umbgtt10/fluxion/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.1
[0.1.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.0
