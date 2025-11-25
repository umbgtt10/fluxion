# Changelog

All notable changes to the Fluxion project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Transformation Operator**: Implemented `scan_ordered` operator for stateful accumulation
  - Accumulates state across stream items, emitting intermediate results for each input
  - Similar to `Iterator::fold` but emits per-item instead of final result only
  - Use cases: Running totals/averages, state machines, building collections over time, moving window calculations
  - Can transform types (e.g., i32 → String, Event → State)
  - Error propagation without state reset
  - Comprehensive test suite: 11 functional tests + 9 error tests + 6 composition tests = 26 total tests
  - Performance benchmarks covering simple accumulation, counting, and complex state (Vec accumulator)
  - Fully integrated into FluxionStream with method forwarding
  - Complete documentation with 5 examples (running sum, type transformation, list building, state machine, error handling)
- **Filtering Operators**: Implemented `distinct_until_changed` and `distinct_until_changed_by` operators
  - `distinct_until_changed` - Suppress consecutive duplicate values using `PartialEq`
  - `distinct_until_changed_by` - Custom comparison function for duplicate suppression (no `PartialEq` requirement)
  - Use cases: Change detection, noise reduction, field-based comparison, case-insensitive filtering, threshold-based filtering
  - Comprehensive test suite: 17 unit/error tests + 7 composition tests + 6 doctests = 30 total tests
  - Performance benchmarks covering various duplicate factors and comparison strategies
  - Follows Rust stdlib patterns (`sort`/`sort_by`, `dedup`/`dedup_by`, `max`/`max_by`)

### Changed
- **BREAKING**: Moved `type Inner` from `HasTimestamp` trait to `Timestamped` trait
  - `HasTimestamp` now only defines `type Timestamp` and `fn timestamp()` for minimal read-only access
  - `Timestamped` trait now includes `type Inner: Clone` along with construction methods
  - Better separation of concerns: ordering (HasTimestamp) vs wrapping (Timestamped)
  - Updated all implementations across workspace (24 files)
  - Updated documentation in README files and INTEGRATION.md to reflect new trait structure
  - Migration: Add `type Inner = YourType;` to your `Timestamped` implementations and remove it from `HasTimestamp`

## [0.3.0] - 2025-11-24

### Added
- **Complete Example Application**: New `legacy-integration` example demonstrating wrapper pattern for legacy systems
  - Integrates 3 legacy data sources: Database (JSON), Message Queue (XML), File Watcher (CSV)
  - Demonstrates adapter pattern for adding timestamps at system boundaries
  - Uses `merge_with` for stateful repository aggregation with shared state
  - Real-time analytics with `subscribe_async` for event processing
  - Graceful shutdown with Ctrl+C handling and 1-minute auto-timeout
  - Comprehensive README with architecture diagrams and key concepts
  - Added to CI build script for continuous validation
- **Error Handling Operator**: Implemented `on_error` operator for composable error handling
  - Chain of Responsibility pattern for selective error handling
  - Handlers can consume errors (return `true`) or propagate them (return `false`)
  - Multiple `on_error` calls can be chained for layered error handling
  - Full documentation in `docs/ON_ERROR_OPERATOR.md`
  - Comprehensive test suite with 13 tests covering all scenarios
  - Examples in rustdoc showing basic consumption and chain of responsibility
- **Documentation**: Enhanced error handling documentation in `docs/ERROR-HANDLING.md`
  - Added complete section on `on_error` operator with examples
  - Chain of Responsibility pattern explained with code samples
  - Error recovery strategies documented
  - Integration with existing error propagation patterns

### Changed
- **Documentation**: Reorganized README.md examples section into two categories
  - **Stream Aggregation**: Intrinsic Timestamps pattern (production event sources with built-in ordering)
  - **Legacy Integration**: Wrapper Pattern (adding timestamps at system boundaries)
  - Each example now clearly labeled with its integration pattern
- **Documentation**: Updated `INTEGRATION.md` guide with both example applications
  - Pattern 1 (Intrinsic Timestamps) links to `stream-aggregation` example
  - Pattern 3 (Wrapper Timestamps) links to `legacy-integration` example
  - Pattern 2 (Extrinsic Timestamps) links to test suites
  - Comprehensive examples section with clear pattern associations
- **Testing**: Consolidated test patterns across workspace for consistency
  - All tests now follow established create/compose/send/next pattern
  - Improved test readability and maintainability
  - Updated 450+ test assertions across 13 test files
- **CI/CD**: Added `legacy-integration` example to CI build script
  - Both examples now validated in continuous integration
  - Ensures examples remain up-to-date with library changes

### Fixed
- **Documentation**: Removed duplicate `docs/fluxion_operators_roadmap.md` file
  - Fixed case sensitivity issue (kept uppercase `FLUXION_OPERATORS_ROADMAP.md`)
  - All documentation references already pointed to correct uppercase version
  - Prevents broken links on case-sensitive filesystems (Linux/macOS)

## [0.2.2] - 2025-11-23

### Added
- **Trait Hierarchy**: Introduced 5 specialized traits in `fluxion-core` for operator bounds
  - `ComparableInner` - Base trait with `'static` bounds for extracting comparable inner values
  - `ComparableSync` - Thread-safe inner values without `'static` requirement (for gating operators)
  - `ComparableTimestamped` - Extends `ComparableInner` with timestamp operations
  - `ComparableUnpin` - Ordered items with `Unpin` inner values
  - `ComparableUnpinTimestamped` - Extends `ComparableTimestamped` with `Unpin`
  - Provides semantic clarity and type-level documentation for operator requirements
  - Applied to all 7 stream operators: `combine_latest`, `with_latest_from`, `take_while_with`, `emit_when`, `take_latest_when`, `map_ordered`, `filter_ordered`
- **CI/CD**: Added doctest verification to CI pipeline
  - GitHub Actions workflow now runs `cargo test --doc`
  - Local CI script (`.ci/ci.ps1`) now includes doctest step
  - Ensures all documentation examples compile and pass
- **CI/CD**: Added benchmark compilation verification to CI pipeline (`cargo bench --no-run`)
- **CI/CD**: Integrated Tarpaulin for code coverage tracking in CI pipeline
- **Benchmarks**: Comprehensive performance benchmarking suite for all stream operators
  - Added 10 operator benchmarks: `map_ordered`, `filter_ordered`, `combine_latest`, `combine_with_previous`, `with_latest_from`, `merge_with`, `ordered_merge`, `emit_when`, `take_latest_when`, `take_while_with`
  - Benchmarks test multiple payload sizes (0, 128 bytes) and stream sizes (100, 1K, 10K elements)
  - All benchmarks use Criterion.rs for statistical analysis
- **Documentation**: Added benchmark links to all operator documentation in `fluxion-stream/README.md`
  - Each operator now includes links to: Full documentation | Tests | Benchmarks
- **Documentation**: Added `merge_with` operator documentation
  - Comprehensive documentation in `fluxion-stream/README.md` under Combination Operators
  - Added to operator selection guide table
  - Added to `FLUXION_OPERATOR_SUMMARY.md` with usage examples
  - Included in order attribute semantics table
- **Testing**: Added comprehensive unit tests for `channel_ext` module
- **Testing**: Added missing test coverage across multiple modules
- **Testing**: Introduced `Sequenced<T>` test wrapper with global counter-based timestamps
  - Replaces complex timestamp implementations with simple atomic counter
  - Implements `Timestamped` trait for easy test data wrapping
  - Located in `fluxion-test-utils` crate for reuse across test suites
- **Documentation**: Expanded test documentation with links to example files
- **Documentation**: Added self-contained examples for `subscribe_async` and `subscribe_latest_async`
  - Examples demonstrate sequential processing and burst cancellation patterns
  - Include inline data structures for easy understanding
- **Testing**: Added `assert_stream_ended` utility function integrated across all test files

### Changed
- **Refactoring**: Reorganized trait structure in `fluxion-core`
  - Moved all traits from `src/traits/` directory to `src/` root (one file per trait)
  - Cleaner module structure with direct trait declarations
  - Simplified imports across the workspace
- **Cleanup**: Removed redundant `OrderedFluxionItem` bounds from operators
  - Cleaned up 3 instances where trait bounds were duplicated
  - Simpler, more concise trait bounds throughout
- **BREAKING**: Refactored lock utilities to use recovery-based approach
  - Renamed `lock_or_error` → `lock_or_recover` for honest naming
  - Changed return type from `Result<MutexGuard<T>, FluxionError>` to `MutexGuard<T>`
  - Always recovers from poisoned mutexes (aligns with Rust stdlib behavior)
  - Removed `try_lock` wrapper function (redundant)
  - Simplified all operators using locks: removed ~100 lines of unreachable error-handling code
  - Updated operators: `combine_latest`, `with_latest_from`, `take_latest_when`, `take_while_with`, `emit_when`
  - Test suite reduced from 14 to 12 tests (removed redundant tests)
  - All 1,500+ tests passing
- **BREAKING**: Migrated from `Ordered` trait to `Timestamped` trait
  - Renamed core trait from `Ordered` to `Timestamped` for better clarity
  - Method changes:
    - `order() -> u64` → `timestamp() -> Self::Timestamp` (generic timestamp type)
    - `get() -> &Self::Inner` → Removed (use `&*value` dereferencing instead)
    - `inner() -> Self::Inner` → Removed (use `clone().into_inner()` instead)
    - `with_order(value, order)` → `with_timestamp(value, timestamp)`
    - Added `with_fresh_timestamp(value)` for auto-timestamping
    - Added `into_inner(self) -> Self::Inner` (required method, no default implementation)
  - All stream operators updated to use new trait
  - More idiomatic Rust: eliminated convenience methods in favor of standard patterns
  - Net -19 lines of code after refactoring (simpler, cleaner API)
- **Code Quality**: Eliminated dead code in lock error handling
  - Removed unreachable error branches that were showing as uncovered in codecov
  - Lock recovery behavior now matches Rust standard library semantics
  - Clearer API with honest function naming (`lock_or_recover` instead of `lock_or_error`)
- **Testing**: Consolidated and cleaned up test suite organization
  - Merged duplicate test files and removed redundant tests
  - Simplified test code for better maintainability
  - Improved test naming conventions for clarity
  - Fixed 200+ test compilation errors from trait migration
  - All 1,684 tests passing, all 63 doc tests passing
- **Testing**: Refactored `Sequenced<T>` implementation for simplicity
  - Removed `Deref` and `DerefMut` traits (use `.value` field directly)
  - Removed `Display` trait (not needed for test infrastructure)
  - Cleaner trait bounds referencing concrete implementation
- **Documentation**: Fixed all doc tests in `fluxion-exec` crate
  - Replaced `tokio_test::block_on` with `#[tokio::main]` for self-contained examples
  - All 8 doc tests now compile and run successfully
- **Testing**: Refactored tests to use create/compose/send/next pattern for stream composition
- **Documentation**: Enhanced README with automated example synchronization
  - Added subscribe_async_example and subscribe_latest_async_example
  - Dependencies automatically extracted from test files and synced to README
- **Quality**: Improved error handling
  - Zero `unwrap()` calls in production code
  - Only 2 justified `expect()` calls with invariant checks
- **Quality**: Fixed all Clippy warnings across workspace

### Fixed
- **Documentation**: Fixed all 8 doctest compilation errors
  - Updated 7 examples using old `Timestamped` trait API to use `HasTimestamp`
  - Added missing trait imports (`Debug`, `HasTimestamp`) in various doctests
  - All 66 doctests now compile and pass (0 ignored)
  - Updated examples in: `comparable_inner.rs`, `comparable_unpin.rs`, `timestamped.rs`, `fluxion_item.rs`, `ordered_fluxion_item.rs`, `channel_ext.rs`, `fluxion_stream.rs`
- **Bug**: Fixed timestamp preservation in `take_latest_when` operator
  - Now stores full timestamped value instead of just inner value
  - Correctly preserves trigger event's timestamp when emitting
- **Bug**: Fixed `merge_with` operator to generate timestamps internally
  - No longer depends on test utilities in production code
  - Properly assigns timestamps to merged values
- **Documentation**: Resolved all rustdoc warnings (4 → 0)
  - Fixed 3 broken intra-doc links to `Ordered` trait
  - Fixed unclosed HTML tag error in `timestamped.rs`
  - Updated all trait signatures in documentation
- **Documentation**: Comprehensive documentation consistency review
  - Updated INTEGRATION.md with current `Timestamped` trait examples
  - Updated all README files across workspace
  - Fixed all code examples to use `Sequenced<T>` test wrapper
  - Synchronized code samples with actual test implementations
  - Fixed terminology inconsistencies (sequence numbers → timestamps, ordered streams → timestamped streams)
  - Enhanced `CompareByInner` trait documentation with examples and clearer explanations

## [0.2.1] - 2025-11-18

### Fixed
- **Publishing**: Corrected README path for fluxion-rx crate to display root documentation on crates.io
- **Documentation**: Fixed broken anchor links in README.md table of contents
- **Documentation**: Standardized Error Handling Guide links to use relative paths consistently across all source files
- **Documentation**: Updated all version references from 0.2.0 to 0.2.1 across documentation files

## [0.2.0] - 2025-11-18 [YANKED]

### Added
- **Automation**: Created `.ci/sync-readme-examples.ps1` PowerShell script for automatic README synchronization
  - Syncs test file code to README examples (removes copyright headers for clean examples)
  - Auto-detects dependencies from code (use statements, attributes like `#[tokio::test]`, qualified paths like `anyhow::Result`)
  - Extracts exact versions from workspace `Cargo.toml` automatically
  - Zero manual maintenance - adding new imports automatically updates README dependencies
- **Documentation**: Comprehensive unwrap/expect assessment (`assessments/UNWRAP_EXPECT_ASSESSMENT.md`) - Grade A+ with only 1 expect() in production code
- **Error Handling**: Comprehensive error propagation through `StreamItem<T>` enum in all operators
- **Documentation**: Added `docs/ERROR-HANDLING.md` - complete error handling guide with patterns and examples
- **Documentation**: Added `# Errors` sections to all 9 stream operators with links to error handling guide
- **Documentation**: Added README.md files to all workspace crates (`fluxion-core`, `fluxion-rx`, `fluxion-merge`, `fluxion-ordered-merge`, `examples/stream-aggregation`)
- **Documentation**: Main README now references all individual crate READMEs in "Crate Documentation" and "Workspace Structure" sections
- **API**: Implemented `CompareByInner` trait for `StreamItem<T>` to enable `with_latest_from` operator
- **Core**: New `fluxion-core::StreamItem<T>` enum for error propagation (`Value(T)` | `Error(FluxionError)`)
- **Core**: Merged `fluxion-error` into `fluxion-core` - error types now in `fluxion-core::error` module
- **Testing**: Added `anyhow` dependency to all test suites for better error handling in tests

### Changed
- **Testing**: All test functions now return `anyhow::Result<()>` for proper error propagation
- **Testing**: Replaced all `.unwrap()` calls with `?` operator in test assertions (200+ replacements across 20+ test files)
- **Testing**: Improved test code quality and error reporting - test failures now show proper error context
- **BREAKING**: All stream operators now return `StreamItem<T>` instead of bare `T` values
  - `combine_latest` → `Stream<Item = StreamItem<OrderedWrapper<CombinedState<T>>>>`
  - `with_latest_from` → `Stream<Item = StreamItem<OrderedWrapper<R>>>`
  - `take_latest_when` → `Stream<Item = StreamItem<T>>`
  - `emit_when` → `Stream<Item = StreamItem<T>>`
  - `take_while_with` → `Stream<Item = StreamItem<T>>`
  - `combine_with_previous` → `Stream<Item = StreamItem<WithPrevious<T>>>`
  - `map_ordered` → `Stream<Item = StreamItem<U>>`
  - `filter_ordered` → `Stream<Item = StreamItem<T>>`
  - `ordered_merge` → `Stream<Item = StreamItem<T>>`
- **BREAKING**: `fluxion-core::FluxionError` now implements `Clone` trait
- **BREAKING**: Simplified `fluxion-core::FluxionError` from 12 variants to 4 actually-used variants
  - Removed: `ChannelSendError`, `ChannelReceiveError`, `CallbackPanic`, `SubscriptionError`, `InvalidState`, `Timeout`, `UnexpectedStreamEnd`, `ResourceLimitExceeded`
  - Kept: `LockError`, `StreamProcessingError`, `UserError`, `MultipleErrors`
- **BREAKING**: Merged `fluxion-error` crate into `fluxion-core` - import from `fluxion_core` instead of `fluxion_error`
- **BREAKING**: API method naming improvements for better ergonomics:
  - `WithPrevious::both()` → `as_pair()` (clearer semantic meaning)
  - `CombinedState::get_state()` → `values()` (more idiomatic Rust)
  - `safe_lock()` → `lock_or_error()` (explicit error handling intent)
  - Updated 100+ call sites across codebase
- **Code Quality**: Simplified `std::` imports across codebase (added targeted `use` statements)
- **Documentation**: Updated `docs/ERROR-HANDLING.md` to reflect simplified error variants
- **Documentation**: Updated operator documentation to remove references to deleted error variants
- **Documentation**: Updated all documentation references for renamed methods
- **Operators**: Lock errors now propagate as `StreamItem::Error` instead of silently dropping items
- **API**: Standardized all operators to return `impl Stream<Item = StreamItem<...>>` (removed `FluxionStream` wrapper inconsistency)
- **API**: Removed redundant `FluxionStream::from_stream()` method (use `::new()` instead)
- **Documentation**: Enhanced `filter_ordered` documentation with comprehensive examples and use cases
- **Documentation**: Updated README.md to reference error handling guide in features and guides sections
- **Tests**: Updated all 186 tests across workspace to handle `StreamItem<T>` wrapper
- **Tests**: Moved `sequenced_tests.rs` to `fluxion-test-utils/tests/` for better organization

### Removed
- **Documentation**: Removed `docs/REFACTORING_PLAN.md` (implementation complete, details preserved in git history)
- **Workspace**: Removed `fluxion-error` crate (merged into `fluxion-core`)
- **API**: Removed deprecated method names (`both()`, `get_state()`, `safe_lock()`) - use new names instead

### Fixed
- **Code Quality**: Replaced unsafe `unwrap()` with documented `expect()` in `fluxion-ordered-merge` explaining safety invariant
- **Error Handling**: Lock poisoning errors no longer cause silent data loss
- **API Consistency**: `take_while_with` now matches other operators' return type pattern

### Migration Guide

**Before (v0.1.x):**
```rust
use fluxion_error::FluxionError;

let mut stream = stream1.combine_latest(vec![stream2], |_| true);
let result = stream.next().await.unwrap();
let value = result.get();
```

**After (v0.2.0):**
```rust
use fluxion_core::FluxionError;  // Changed from fluxion_error

let mut stream = stream1.combine_latest(vec![stream2], |_| true);
let item = stream.next().await.unwrap();
match item {
    StreamItem::Value(result) => {
        let value = result.get().values();  // Changed from get_state()
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

**Error import migration:**
```rust
// Before
use fluxion_error::{FluxionError, Result};

// After
use fluxion_core::{FluxionError, Result};
```

**Method name migration:**
```rust
// Before
let state = combined.get_state();
let pair = with_prev.both();
let guard = safe_lock(&mutex, "context")?;

// After
let state = combined.values();
let pair = with_prev.as_pair();
let guard = lock_or_error(&mutex, "context")?;
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

[Unreleased]: https://github.com/umbgtt10/fluxion/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/umbgtt10/fluxion/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.2.1
[0.2.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.2.0
[0.1.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.1
[0.1.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.0
