# Changelog

All notable changes to the Fluxion project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] - Not Published (Internal Release)

**Goal:** Prepare for runtime abstraction by removing wall-clock time dependencies

### Changed
- **Breaking:** Removed `with_fresh_timestamp()` method from `Timestamped` trait (`fluxion-core`)
  - Removes implicit wall-clock time dependency from core trait
  - Wrapper types should provide their own `new()` or similar constructors
  - Prepares for runtime abstraction and deterministic time in tests
  - Migration: Replace `T::with_fresh_timestamp(value)` with custom constructors

- **Breaking:** Removed `chrono` dependency from `fluxion-stream-time`
  - Migrated from `chrono::DateTime<Utc>` to `std::time::Instant`
  - Renamed `ChronoTimestamped<T>` to `InstantTimestamped<T>`
  - Now uses monotonic timestamps instead of wall-clock time
  - Benefits: Zero external time dependencies, faster compilation, WASM-ready
  - Migration: Replace `ChronoTimestamped` with `InstantTimestamped` in your code

### Fixed
- **`emit_when` operator** (`fluxion-stream`)
  - Now uses correct timestamps based on which stream triggered the emission
  - Source triggers: Uses source event timestamp (preserves temporal ordering)
  - Filter triggers: Uses filter event timestamp (preserves causality)
  - Previously created synthetic "now" timestamps, breaking ordering guarantees

## [0.6.0] - 2025-12-18

**Goal:** Stream composition, multi-consumer support, and sampling operators

### Added
- **`tap` Operator** (`fluxion-stream`)
  - Perform side-effects without transforming items (logging, debugging, metrics)
  - `TapExt` trait with `tap(f)` method for any `Stream<Item = StreamItem<T>>`
  - Pass-through operator: items flow unchanged, callback observes each value
  - Errors pass through unchanged (callback only invoked for values)
  - Use cases: Debugging complex pipelines, logging, metrics collection, tracing
  - Comprehensive test suite: `tap_tests.rs`, `tap_error_tests.rs`, `tap_composition_tests.rs`, `tap_composition_error_tests.rs`
  - Benchmark: `tap_bench.rs` with basic and chained tap scenarios
  - Documentation in `FLUXION_OPERATOR_SUMMARY.md` and `fluxion-stream/README.md`
  - RxJS equivalent: `tap` / `do`

- **`sample_ratio` Operator** (`fluxion-stream`)
  - Probabilistic downsampling with configurable ratio (0.0 to 1.0)
  - `SampleRatioExt` trait with `sample_ratio(ratio, seed)` method
  - Deterministic seeding for reproducible tests; use `fastrand::u64(..)` in production
  - Errors always pass through unconditionally (never sampled)
  - Panics on invalid ratio (outside 0.0..=1.0)
  - Use cases: Load reduction, logging sampling, monitoring downsampling
  - Comprehensive test suite: `sample_ratio_tests.rs`, `sample_ratio_error_tests.rs`, `sample_ratio_composition_tests.rs`, `sample_ratio_composition_error_tests.rs`
  - Benchmark: `sample_ratio_bench.rs` with half/full/sparse sampling scenarios
  - Documentation in module-level docs with usage examples

- **`share` Operator** (`fluxion-stream`)
  - Convert cold streams to hot, multi-subscriber broadcast sources
  - `ShareExt` trait with `share()` method for any `Stream<Item = StreamItem<T>>`
  - `FluxionShared` subscription factory - call `subscribe()` for independent subscriber streams
  - Late subscribers miss past items (hot stream semantics)
  - Source stream consumed once, results broadcast to all subscribers
  - Comprehensive test suite: `fluxion_shared_tests.rs`, `fluxion_shared_error_tests.rs`, `fluxion_shared_composition_tests.rs`, `fluxion_shared_composition_error_tests.rs`
  - Benchmark: `share_bench.rs` with subscriber count variations
  - Documentation in `FLUXION_OPERATOR_SUMMARY.md` and `fluxion-stream/README.md`

- **`partition` Operator** (`fluxion-stream`)
  - Split a stream into two based on a predicate function
  - `PartitionExt` trait with `partition()` method returning `(PartitionedStream<T>, PartitionedStream<T>)`
  - Items satisfying predicate go to first stream, others to second stream
  - Chain-breaking operator (returns two streams, cannot chain further on original)
  - Spawns background routing task for non-blocking operation
  - Timestamp-preserving (original timestamps maintained in both output streams)
  - Error propagation to both output streams
  - Unbounded internal buffers (safe to drop one stream)
  - Use cases: Error routing, priority queues, type routing, threshold filtering
  - Comprehensive test suite: `partition_tests.rs`, `partition_error_tests.rs`, `partition_composition_tests.rs`, `partition_composition_error_tests.rs`
  - Benchmark: `partition_bench.rs` with balanced/imbalanced splits and single consumer scenarios
  - Documentation in `FLUXION_OPERATOR_SUMMARY.md` and `fluxion-stream/README.md`

- **FluxionSubject Enhancements** (`fluxion-core`)
  - `is_closed()` - Check if subject has been closed
  - `subscriber_count()` - Get number of active subscribers
  - `next(value)` - Convenience method for sending values (wraps `send(StreamItem::Value(value))`)

- **`parking_lot` Dependency** (`fluxion-core`)
  - Added `parking_lot` for non-poisoning mutex implementation
  - Enables panic-free production code

### Changed
- **Test Refactoring** (`fluxion-stream`)
  - All `merge_with` tests converted from primitive types (i32/Person) to use `TestData`
  - Standardized test data usage with packaged helpers (`person_alice()`, `person_bob()`, etc.)
  - Replaced verbose multi-line assertions with single compact assertions
  - Consistent use of `Sequenced::new()` for auto-timestamping, `with_timestamp()` only where explicit control needed
  - All error propagation tests now use `TestData` for consistency
  - Improved test readability and maintainability across all test files

- **BREAKING**: Removed `FluxionStream` wrapper type
  - Extension traits now work directly on any `Stream<Item = StreamItem<T>>`
  - Use `IntoFluxionStream` trait to convert `UnboundedReceiver` to fluxion streams
  - Simplified API: `rx.into_fluxion_stream().map_ordered(...)` instead of `FluxionStream::new(rx).map_ordered(...)`

- **`FluxionSubject` Mutex Implementation** (`fluxion-core`)
  - Migrated from `std::sync::Mutex` to `parking_lot::Mutex`
  - Eliminates all `unwrap()` calls on lock acquisition (parking_lot doesn't poison)
  - Zero `unwrap()`/`expect()` in production code

- **Algorithmic Invariant Handling** (`fluxion-ordered-merge`, `fluxion-stream`, `fluxion-stream-time`)
  - Replaced `expect()` calls with pattern matching and `unreachable!()`
  - `ordered_merge.rs` - Pattern matching for buffer access
  - `partition.rs` - `unwrap_or_else(|_| unreachable!())` for subscription
  - `delay.rs` - Pattern matching for future completion

- **Documentation**: Updated operator count to 29 (includes all stream, time, and exec operators)
  - **Note**: `window_by_count` operator was previously undocumented; now fully documented in operator summary
- **Documentation**: Updated test count to 800+ (was 730)
- **Documentation**: Improved `FluxionSubject` documentation
  - Added comprehensive module-level documentation with examples
  - Enhanced struct and method documentation
  - Added integration example showing usage with stream operators
- **Documentation**: Cleaned up `FLUXION_OPERATORS_ROADMAP.md`
  - Removed all implemented operators, keeping only planned/future operators
- **Documentation**: Updated `ASSESSMENT_CLAUDE.md`
  - Production code now has 0 `unwrap()` and 0 `expect()`
  - Updated RxRust comparison showing Fluxion wins on all quality metrics

### Migration Guide

**FluxionStream removal:**
```rust
// Before (v0.5.x)
use fluxion_stream::FluxionStream;
let stream = FluxionStream::new(rx);

// After
use fluxion_stream::IntoFluxionStream;
let stream = rx.into_fluxion_stream();
```

## [0.5.0] - 2025-12-04

**Goal:** Time-based operators, API consolidation, and test organization

### Added
- **Time Operators**: New `fluxion-stream-time` crate with temporal operators
  - `delay` - Shifts stream emissions forward by a specified duration
  - `debounce` - Emits only after a quiet period (no new values)
  - `throttle` - Enforces minimum interval between emissions
  - `sample` - Emits most recent value at periodic intervals
  - `timeout` - Errors if no emission within specified duration
  - All operators use `std::time::Duration` (v0.6.1+ uses `std::time::Instant` timestamps)
  - Heap allocation minimized in debounce and delay implementations
- **Benchmarks**: Comprehensive benchmarks for all time operators
  - `delay_bench`, `debounce_bench`, `throttle_bench`, `sample_bench`, `timeout_bench`
  - Integrated into CI pipeline
- **Composition Error Tests**: Added error propagation tests for all operators
  - Tests verify errors flow correctly through operator chains
  - Coverage for: `combine_with_previous`, `distinct_until_changed`, `emit_when`, `filter_ordered`, `map_ordered`, `merge_with`, `ordered_merge`, `scan_ordered`, `take_latest_when`, `take_while_with`, `with_latest_from`
- **Design Documentation**: Added architectural decision documents
  - `docs/design/UNORDERED_API_STRATEGY.md` - Strategy for ordered vs unordered APIs

### Changed
- **BREAKING**: Renamed execution methods for cleaner API
  - `subscribe_async` → `subscribe`
  - `subscribe_latest_async` → `subscribe_latest`
  - Module names updated: `subscribe_async.rs` → `subscribe.rs`, `subscribe_latest_async.rs` → `subscribe_latest.rs`
  - Trait names updated: `SubscribeAsyncExt` → `SubscribeExt`, `SubscribeLatestAsyncExt` → `SubscribeLatestExt`
- **Core Trait Unification**: Introduced unified `Fluxion` trait replacing fragmented trait hierarchy
  - Replaced `ComparableInner`, `ComparableSync`, `ComparableTimestamped`, `ComparableUnpin`, `ComparableUnpinTimestamped`, and `OrderedFluxionItem`
  - New `Fluxion` trait enforces `Timestamped + Clone + Send + Sync + Unpin + 'static + Debug + Ord`
  - Simplified type bounds across all stream operators
- **Test Organization**: Major restructuring of test files
  - All operator tests now organized in dedicated folders (e.g., `tests/combine_latest/`, `tests/emit_when/`)
  - Each operator folder contains: `mod.rs`, `*_tests.rs`, `*_error_tests.rs`, `*_composition_tests.rs`, `*_composition_error_tests.rs`
  - Removed monolithic test files from `fluxion/tests/`
  - Tests refactored to use standard test data helpers
  - Eliminated use of `sleep()` in tests - replaced with deterministic synchronization
- **Documentation**: Updated `FLUXION_OPERATOR_SUMMARY.md`
  - Corrected API reference from `.order()` to `.timestamp()`
  - Added missing operators to summary table
- **Doc Tests**: Stabilized flaky doc tests in `subscribe` and `subscribe_latest`
  - Replaced race-prone synchronization with `NotifyOnPendingStream` pattern
  - No more `yield_now()` or `sleep()` in doc tests

### Fixed
- **Bug**: Error propagation issue in `take_while_with` operator
  - Errors now correctly propagate through the operator chain
- **Bug**: Lock handling issues in several operators
  - Improved lock recovery and error handling

### Removed
- **Legacy Traits**: Deleted obsolete trait files in `fluxion-core`
  - `comparable_inner.rs`, `comparable_sync.rs`, `comparable_timestamped.rs`
  - `comparable_unpin.rs`, `comparable_unpin_timestamped.rs`, `ordered_fluxion_item.rs`
- **Monolithic Test Files**: Removed large test files from `fluxion/tests/`
  - `composition_error_tests.rs` (545 lines) - tests distributed to operator folders
  - `composition_tests.rs` (2730 lines) - tests distributed to operator folders

### Migration Guide

**Method rename:**
```rust
// Before (v0.4.x)
stream.subscribe_async(handler, token, error_callback).await;
stream.subscribe_latest_async(handler, error_callback, token).await;

// After (v0.5.0)
stream.subscribe(handler, token, error_callback).await;
stream.subscribe_latest(handler, error_callback, token).await;
```

**Import changes:**
```rust
// Before
use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;

// After
use fluxion_exec::subscribe::SubscribeExt;
use fluxion_exec::subscribe_latest::SubscribeLatestExt;
```

## [0.4.0] - 2025-11-28

**Goal:** Expand operator library with transformation and filtering operators

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

- **Stream Control Operators**: Implemented `skip_items`, `take_items`, and `start_with` operators
  - `skip_items(n)` - Skip the first n items from the stream
  - `take_items(n)` - Take only the first n items from the stream
  - `start_with(values)` - Prepend initial values to the beginning of a stream
  - Use cases: Pagination, buffering, stream initialization, warm starts with default values
  - Comprehensive test suites with functional, error, and composition tests
  - Full documentation with examples and use case descriptions

- **Performance Analysis**: Created comprehensive performance assessment reports
  - `assessments/COMBINE-ORDERED-VS-COMBINE-UNORDERED-PERFORMANCE-COMPARISON.md` - Operator-level performance analysis showing 0-5% difference (negligible)
  - `assessments/ORDERED-MERGE-VS-SELECT-ALL-PERFORMANCE-COMPARISON.md` - Low-level primitive benchmarks showing OrderedMerge 10-43% faster
  - 36 benchmark scenarios per operator comparison (3 message sizes × 4 payload sizes × 3 stream counts)
  - Data-driven architectural decisions documented

### Changed
- **BREAKING**: Moved `type Inner` from `HasTimestamp` trait to `Timestamped` trait
  - `HasTimestamp` now only defines `type Timestamp` and `fn timestamp()` for minimal read-only access
  - `Timestamped` trait now includes `type Inner: Clone` along with construction methods
  - Better separation of concerns: ordering (HasTimestamp) vs wrapping (Timestamped)
  - Updated all implementations across workspace (24 files)
  - Updated documentation in README files and INTEGRATION.md to reflect new trait structure
  - **Migration**: Add `type Inner = YourType;` to your `Timestamped` implementations and remove it from `HasTimestamp`

- **Workspace Standardization**: Updated all member crate Cargo.toml files to use workspace inheritance
  - All dependencies now use `workspace = true` for consistency
  - All metadata (edition, rust-version, authors, license) inherited from workspace root
  - Added missing `fluxion-stream-common` to workspace.dependencies
  - Cleaner, more maintainable workspace configuration

- **Benchmark Organization**: Reorganized benchmarks in `fluxion-ordered-merge`
  - Consolidated from separate benchmark files into single `benches/benchmarks.rs` entry point
  - Modules `merge_ordered_bench` and `select_all_bench` now imported from main benchmark file
  - Matches pattern used in `fluxion-stream/benches`
  - Single `[[bench]]` entry in Cargo.toml

### Decision
- **Dual API Model**: Investigated and **rejected** dual-API approach (ordered vs unordered implementations)
  - POC implemented and benchmarked with 36 scenarios per comparison
  - Performance analysis showed negligible difference (0-5%) at operator level due to Arc<Mutex> bottleneck
  - OrderedMerge primitive showed 10-43% advantage, but doesn't translate to operator level
  - Conclusion: Complexity of maintaining two APIs not justified by performance gains
  - Fluxion will remain focused on ordered semantics as its core value proposition

### Fixed

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
  - Full documentation in `docs/FLUXION_OPERATOR_SUMMARY.md#on_error`
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
- **Tests**: Updated all tests across workspace to handle `StreamItem<T>` wrapper (186 at v0.2.0)
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

[Unreleased]: https://github.com/umbgtt10/fluxion/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/umbgtt10/fluxion/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/umbgtt10/fluxion/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/umbgtt10/fluxion/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/umbgtt10/fluxion/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/umbgtt10/fluxion/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.2.1
[0.2.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.2.0
[0.1.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.1
[0.1.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.0
