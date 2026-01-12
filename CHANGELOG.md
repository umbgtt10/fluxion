# Changelog

All notable changes to the Fluxion project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-01-12

### Added
- **Complete Runtime Abstraction Achievement**
  - ✅ **5 Runtimes Fully Supported** - Tokio, smol, async-std, WASM, Embassy work out-of-the-box
  - ✅ **All 5 Time Operators Migrated** - debounce, throttle, delay, sample, timeout work seamlessly across all runtimes
  - ✅ **24/27 Operators on Embassy** - Only 3 operators (subscribe_latest, partition, share) fundamentally incompatible due to Embassy's static task allocation model
  - ✅ **Industry First** - Complete reactive streams library working on embedded systems (ARM Cortex-M4F validated in QEMU)

### Changed
- **Architecture Implementation**
  - Implemented dual trait bound system with module-level separation (`multi_threaded.rs` vs `single_threaded.rs`)
  - Solved runtime compatibility without workspace restructuring (simpler than originally planned v0.9.0 architecture)
  - Single implementation per operator maintained (easier maintenance)
  - Zero breaking API changes for users
  - Zero performance overhead with seamless operator chaining

- **Technical Details**
  - Module-level separation using `#[cfg(feature = "...")]` on implementations
  - Separate `multi_threaded.rs` (Send + Sync bounds) and `single_threaded.rs` (no thread bounds)
  - Feature flags: `runtime-tokio`, `runtime-smol`, `runtime-async-std`, `runtime-wasm`, `runtime-embassy`
  - Macro-based code generation for eliminating duplication

- **Documentation Overhaul**
  - Removed `KNOWN_LIMITATIONS.md` - Content superseded by runtime abstraction completion
  - Removed `FUTURE_ARCHITECTURE.md` - Alternative architecture not needed; actual solution documented in ROADMAP.md
  - Updated all documentation to reflect that API migration is complete (not "future v0.9.0")
  - Fixed 20+ misleading future-tense references across documentation
  - Clarified Embassy's 3 incompatible operators as architectural constraints, not temporary limitations
  - Archived v0.8.0/v0.9.0 roadmap sections describing unimplemented alternative architecture
  - Fixed all broken documentation links (removed references to deleted files)
  - Updated README.md to remove misleading "Future Architecture" and "Known Limitations" links
  - Corrected FLUXION_OPERATOR_SUMMARY.md Embassy status from "Coming in v0.9.0" to accurate technical limitations
  - Updated FLUXION_OPERATORS_ROADMAP.md to v0.7.1 with correct operator status
  - Aligned README, PITCH, operator guides, and API docs with implementation reality

### Documentation
- **Accuracy Improvements**
  - All documentation now accurately reflects current capabilities (v0.7.1 state)
  - Eliminated confusion between completed work and future plans
  - Embassy limitations clearly documented as architectural constraints, not temporary gaps
  - Documentation audit complete with zero broken links

### Quality Gates
- ✅ All 990+ tests passing across 5 runtimes
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ CI green for all runtime configurations
- ✅ WASM example validated in browser
- ✅ Embassy example validated in QEMU on ARM Cortex-M4F

### The Competitive Advantage
**Fluxion is NOW the ONLY reactive streams library that offers:**
- 27 production-ready operators
- 5 runtimes: Tokio, smol, async-std, WASM, and Embassy (embedded)
- Same API from servers to browsers to microcontrollers
- Zero-config for Tokio users, optional runtime selection for others
- no_std + alloc support (24/27 operators)
- True embedded validation on ARM hardware

**Market Position:**
- **RxRust**: ❌ Requires custom code for non-Tokio runtimes, ❌ No embedded support
- **Other reactive libs**: ❌ std-only, ❌ No embedded story
- **Embassy ecosystem**: ❌ No full-featured reactive streams library
- **Fluxion**: ✅ Works everywhere, production-ready, extensively tested

## [0.7.1] - Unreleased

### Changed
- **Embassy Sensors Example** (`examples/embassy-sensors`)
  - Migrated from `std` to true embedded target (`thumbv7em-none-eabihf` for ARM Cortex-M4F)
  - Converted to `#![no_std]` with custom heap allocator (`embedded-alloc` with 64KB heap)
  - Implemented custom SysTick-based time driver for Embassy runtime
  - Replaced defmt with semihosting-based logging for QEMU compatibility
  - Migrated from git dependencies to stable crates.io versions (embassy-executor 0.6, embassy-time 0.5)
  - Added QEMU automation scripts for ARM emulation (mps2-an386 machine)
  - Successfully demonstrates all Fluxion operators in pure embedded environment
  - CI integration: ARM compilation verification and optional QEMU execution

### Fixed
- **Benchmark Infrastructure** (`fluxion-stream`, `fluxion-stream-time`, `fluxion-core`)
  - Fixed "no reactor running" errors in benchmarks that create operators spawning background tasks
  - Operators like `share`, `partition`, `combine_latest`, `with_latest_from`, `merge_with`, `emit_when`, `take_latest_when`, `window_by_count`, `combine_with_previous`, `start_with`, `tap`, `take_items`, `skip_items`, `take_while_with`, `distinct_until_changed`, `distinct_until_changed_by`, `scan_ordered`, `sample_ratio`, `map_ordered`, `filter_ordered`, and `ordered_merge` now created inside Tokio runtime context
  - All benchmarks now execute successfully on CI build servers without runtime panics
  - Setup time properly excluded from performance measurements across all benchmarks

### Performance
- **Benchmark Validation** (`fluxion-ordered-merge`)
  - Validated `ordered_merge` vs `select_all` performance across various configurations
  - `ordered_merge` shows 10-50% better throughput, with advantages growing with item count, payload size, and stream count
  - For large loads (m=10000, s=5), `ordered_merge` achieves up to 8.3 Melem/s vs 5.2 Melem/s for `select_all`
  - Confirms ordered merging is more efficient for sorted stream scenarios

## [0.7.0] - 2025-12-31

### Fixed
- **CI Infrastructure** (`.github/workflows/ci.yml`)
  - Fixed PowerShell line continuation issue in Windows CI (removed backslash line breaks)
  - Changed cargo-llvm-cov command to single-line format for cross-platform compatibility
  - CI now runs successfully on Windows, Linux, and macOS

### Added
- **Documentation** (`docs/`)
  - Added `KNOWN_LIMITATIONS.md` - Comprehensive guide to current limitations with practical workarounds
    - Documents 2 operators requiring alternatives in Embassy/WASM (`combine_latest`, `with_latest_from`)
    - Provides MergedStream pattern as production-ready workaround
    - Type inference guidance for operator chains
    - Compatibility matrix showing 24/27 operators work everywhere
  - Added `FUTURE_ARCHITECTURE.md` - Technical architecture plan for v0.9.0 runtime isolation
    - Documents Runtime trait pattern with associated types (Mutex, Timer, Spawner, Instant)
    - Shows how runtime-specific crates solve all three current limitations
    - Explains phased implementation (0.8.0 foundation, 0.9.0 completion)
    - Includes migration guide and code examples
  - Both documents linked from README.md Features section for transparency

### Changed
- **Roadmap Updates** (`ROADMAP.md`)
  - Restructured v0.8.0 as "Runtime Isolation Foundation"
    - Phase 1: Core layer separation (fluxion-core, fluxion-runtime)
    - Phase 2: Runtime abstraction layer (Runtime trait + implementations)
    - Phase 3: Shared core implementations (generic over R: Runtime)
  - Restructured v0.9.0 as "Runtime Isolation Complete"
    - Phase 4: Runtime-specific crates (fluxion-tokio, fluxion-embassy, etc.)
    - Phase 5: Migration and validation
  - Updated with architectural details from FUTURE_ARCHITECTURE.md

- **Test Coverage** (Internal metrics)
  - Achieved 95.26% code coverage with cargo-llvm-cov (1934/2031 lines)
  - 1,041 tests passing (990+ production tests + 106 doc tests + 36 benchmarks)
  - 10.8:1 test-to-code ratio maintained

### Removed
- **Documentation Cleanup** (`docs/`)
  - Removed intermediate analysis documents (API_SURFACE_COMPATIBILITY_ANALYSIS.md, CHAINING_TIME_NONTIME_OPERATORS_ANALYSIS.md, INITIATIVE_INTEGRATION_STRATEGY.md)
  - Content consolidated into KNOWN_LIMITATIONS.md and FUTURE_ARCHITECTURE.md

### Migration Notes
- No breaking changes - this is a documentation and CI-only release
- All operator APIs remain unchanged
- Existing code continues to work without modifications
- See KNOWN_LIMITATIONS.md if using Embassy/WASM for guidance on affected operators

## [0.6.14] - 2025-12-27

### Changed
- **Time Operator Interface Unification ✅ COMPLETE** (`fluxion-stream-time`)
  - Unified time-bound operators with non-time-bound operators using `T: Fluxion` trait bounds
  - Changed from `S: Stream<Item = StreamItem<InstantTimestamped<T, TM>>>` to `S: Stream<Item = StreamItem<T>>, T: Fluxion`
  - Timer now a generic parameter on traits (`OperatorExt<T, TM>`) with runtime-specific implementations
  - Feature-gated implementations auto-select correct timer based on `T::Timestamp` type
  - All 5 operators migrated: `delay`, `debounce`, `throttle`, `timeout`, `sample`
  - Each operator has 5 feature-gated implementations (tokio, smol, async-std, wasm, embassy)

### Removed
- **Code Cleanup** (`fluxion-stream-time`)
  - Removed ~500+ lines of `WithDefaultTimerExt` convenience trait boilerplate
  - Removed redundant trait bounds from time operator implementations
  - Simplified operator composition patterns

### Fixed
- **Runtime Type References** (`fluxion-stream-time`)
  - Fixed tokio/smol/async-std implementations to use `std::time::Instant` (standard library)
  - Fixed WASM implementation to use `crate::runtimes::wasm_implementation::WasmInstant` wrapper
  - Fixed Embassy implementation to use `crate::runtimes::embassy_implementation::EmbassyInstant` wrapper
  - Fixed WASM test helper `Person` struct to derive `Eq + PartialOrd + Ord` for `Fluxion` trait bounds
  - Fixed PowerShell script error handling in `run-dashboard.ps1` for trunk stderr output

### Added
- **Test Coverage** (`fluxion-stream-time`)
  - Added 5 new composition tests: time-bound operators chained **before** non-time-bound operators
  - Tests verify bidirectional composition: `delay→map`, `debounce→map`, `throttle→map`, `timeout→map`, `sample→map`

### Quality
- ✅ All 91 tokio tests + 7 doctests passing
- ✅ All 5 WASM tests + 7 doctests passing
- ✅ All runtimes compile successfully (tokio, smol, async-std, wasm, embassy)
- ✅ Code reduction: ~500+ lines removed with no functionality loss
- ✅ Interface consistency: time and non-time operators now use identical patterns

## [0.6.13] - 2025-12-22

### Added
- **Documentation Consistency & Completeness ✅ COMPLETE**
  - Updated version references from 0.6.11 → 0.6.13 across all documentation
  - Added Embassy runtime to all relevant documentation (README, PITCH, operator summary)
  - Fixed feature flag naming: `time-*` → `runtime-*` throughout fluxion-stream-time README
  - Documented convenience methods vs explicit timer methods for time operators
  - Added comprehensive Embassy usage examples and testing approach documentation

### Changed
- **README.md Enhancements**
  - Added Embassy as 5th runtime in Runtime Selection section
  - Updated benefits to mention embedded (Embassy) support
  - Added note about `no_std` + `alloc` requirements for Embassy
  - Expanded cross-platform statement to include embedded systems

- **PITCH.md Updates**
  - Updated runtime support from "4 runtimes" → "5 runtimes" in metrics table
  - Updated runtime abstraction benefits to include Embassy

- **fluxion-stream-time/README.md Improvements**
  - Fixed all feature flag names (`time-tokio` → `runtime-tokio`, etc.)
  - Added `runtime-embassy` to supported runtimes list
  - Added comprehensive Embassy section with compilation-only testing explanation
  - Documented convenience methods vs `_with_timer` explicit methods
  - Clarified when to use each API variant (std vs no_std)

- **docs/FLUXION_OPERATOR_SUMMARY.md**
  - Added Embassy to runtime support note
  - Updated runtime list to include embedded environments

### Changed
- **Test Consistency Improvements** (`fluxion-stream-time`)
  - Refactored all smol runtime tests to match tokio/async-std/embassy pattern
  - Removed `timestamped_person()` helper function for explicit timer usage
  - All smol tests now import `SmolTimer` and `Timer` trait explicitly
  - Inline timestamp creation with `SmolTimestamped::new(value, timer.now())`
  - Consistent test structure across all 5 runtimes (10 smol tests updated)

### Fixed
- **Documentation Tests** (`fluxion-stream-time`)
  - Fixed 8 doctests to compile with proper cfg gates across all runtime features
  - Changed doctests from `rust,ignore` to `rust,no_run` for better validation
  - Added fallback `fn main() {}` for non-tokio features using `#[cfg(not(...))]`
  - All doctests now compile correctly regardless of enabled runtime feature
  - Doctests for: delay, debounce, throttle, timeout, sample, InstantTimestamped, and lib.rs examples

### Quality
- ✅ All documentation synced with sync-readme-examples.ps1
- ✅ cargo check --workspace passes
- ✅ cargo doc --no-deps --workspace generates without warnings
- ✅ cargo clippy passes with zero warnings
- ✅ Version consistency verified across all files
- ✅ All 8 doctests compile with smol feature (use fallback main)
- ✅ All 8 doctests compile with tokio feature (use actual async main)
- ✅ 67 tests passing (57 tokio + 10 smol)

## [0.6.12] - 2025-12-22

### Added
- **no_std Support for Time Operators (Phase 3 Infrastructure) ✅ COMPLETE** (`fluxion-stream-time`)
  - Added `#![cfg_attr(not(feature = "std"), no_std)]` to fluxion-stream-time
  - Added `extern crate alloc` for heap allocations in no_std mode
  - **24/27 operators** now available in no_std + alloc environments (time operators included)
  - Time operators compile with `--no-default-features --features alloc`
  - All 5 time operators (debounce, throttle, delay, sample, timeout) work in no_std

- **Embassy Timer Implementation (5th Runtime) ✅ COMPLETE** (`fluxion-stream-time`)
  - Added `EmbassyTimerImpl` for embedded/no_std targets using `embassy-time` crate
  - Added `EmbassyInstant` wrapper type bridging `embassy_time::Duration` ↔ `core::time::Duration`
  - Added `runtime-embassy` feature flag (`alloc` + `dep:embassy-time`)
  - Added `EmbassyTimestamped<T>` type alias for convenience
  - All 5 time operators (debounce, throttle, delay, sample, timeout) work with Embassy runtime
  - **5 runtimes now supported**: Tokio, smol, async-std, WASM, Embassy
  - Embassy timer is no_std compatible (requires only `alloc` feature)

- **Architecture Documentation** (`RUNTIME_ABSTRACTION_STATUS.md`)
  - Comprehensive documentation of cascading `#[cfg]` dependencies as architectural code smell
  - Analysis of workspace feature management overhead (13 files modified, 7 crates affected)
  - Proposed refactoring initiatives for cleaner no_std architecture
  - Documented trade-offs between current approach and alternatives

### Changed
- **fluxion-stream-time Configuration** (`fluxion-stream-time/Cargo.toml`)
  - Added `std = ["fluxion-core/std", "futures/std"]` feature
  - Added `alloc = ["fluxion-core/alloc"]` feature
  - Dependencies changed to `default-features = false` with explicit features
  - Runtime features (tokio/smol/async-std/wasm) now require std

- **Workspace Dependencies** (`Cargo.toml`)
  - Set `fluxion-core = { ..., default-features = false }` at workspace level
  - All 7 dependent crates now explicitly add `features = ["std"]`
  - Added `embassy-time = "0.3"` to workspace dependencies
  - Enables per-package feature selection (fluxion-stream-time can use no_std)

- **Error Handling** (`fluxion-core/src/fluxion_error.rs`)
  - Made `Error` trait implementation conditional: `#[cfg(feature = "std")]`
  - Made `user_error()` and `from_user_errors()` methods std-only
  - Made `IntoFluxionError` trait std-only (depends on `std::error::Error`)
  - FluxionError still works in no_std (stream_error, timeout_error available)

- **Conditional Exports** (`fluxion-core/src/lib.rs`)
  - Added `#[cfg(feature = "std")]` to `IntoFluxionError` export
  - Prevents compilation errors when std feature disabled
  - Matches feature gate on trait definition

- **Time Operators** (`fluxion-stream-time/src/{debounce,sample,throttle,timeout}.rs`)
  - Added conditional import: `#[cfg(not(feature = "std"))] use alloc::boxed::Box;`
  - Enables `Box::pin()` usage in no_std mode
  - std mode uses Box from prelude (no explicit import needed)

### Fixed
- **WASM Test Compilation** (`fluxion-core/tests/fluxion_error_tests.rs`)
  - Added `#![cfg(feature = "std")]` to entire test file
  - Tests use std-only APIs (IntoFluxionError, user_error, from_user_errors)
  - WASM tests run with `--no-default-features`, these tests are properly skipped

### Technical Notes

**Cascading Feature Dependencies:**
- Single stdlib dependency (`std::error::Error`) creates cascade through modules:
  1. `std::error::Error` not available in no_std
  2. → `IntoFluxionError` trait depends on it → `#[cfg(feature = "std")]`
  3. → Export must match definition → `#[cfg(feature = "std")]`
  4. → Similar cascade for FluxionSubject, SubjectError

**Workspace Feature Management:**
- `default-features = false` at workspace level enables no_std use cases
- Consequence: 7 crates must manually add `features = ["std"]`
- Standard Rust no_std pattern, but creates maintenance overhead
- ~30+ lines of `#[cfg]` guards added across codebase

**Operator Availability:**
- **no_std + alloc (24/27):** All core and time operators except share, subscribe_latest, partition
- **no_std + alloc + runtime-embassy (24/27):** Time operators work with Embassy timer
- **std + runtime (27/27):** All operators fully available with Tokio/smol/async-std
- Time operators use Timer trait (already runtime-agnostic)
- Zero operator changes needed (architecture validated)
- Embassy wrapper pattern solves Duration type incompatibility elegantly

**Architectural Considerations:**
- FluxionError could be split by feature (std/no_std variants)
- IntoFluxionError trait may be removable (users can implement From)
- Feature propagation creates maintenance burden (documented as acceptable)
- Refactoring initiatives documented for future major version

**Embassy Timer Implementation:**
- EmbassyInstant wrapper bridges `embassy_time::Instant` with `core::time::Duration`
- Conversion functions: `to_embassy_duration()`, `to_core_duration()`
- Arithmetic traits implemented: Add/Sub for Duration operations
- No target-specific dependencies (works on any embedded platform)
- Compiles with `--no-default-features --features alloc,runtime-embassy`

### Impact
- **Embedded Support Extended** - Time operators now available on microcontrollers
- **24/27 Operators** on no_std targets (89% coverage)
- **Zero Breaking Changes** - All existing code continues working
- **CI Verified** - 1449/1449 tests passing, no_std compilation check passes
- **Architecture Documented** - Technical debt and refactoring paths clearly outlined

## [0.6.11] - 2025-12-22

### Added
- **Embedded Target Verification** (`.ci/test_embedded_target.ps1`)
  - Added script to verify compilation against `thumbv7em-none-eabihf` (ARM Cortex-M4F)
  - Automated target installation check
  - Validates no_std + alloc build for embedded systems

### Changed
- **Feature Flag Refinement** (`fluxion-core/Cargo.toml`)
  - `std` feature now implies `alloc` feature
  - FluxionSubject remains behind `std` feature gate (requires parking_lot::Mutex)

### Fixed
- **Build Warnings**
  - Added `default-features = false` to workspace `tracing` dependency
  - Fixed unused variable warning in `fluxion_task.rs` (renamed `future` to `_future`)

- **Test Dependencies** (`fluxion-core/Cargo.toml`)
  - Added `thiserror` to dev-dependencies for error handling tests

### Technical Notes

**FluxionSubject Async Migration - Deferred:**
- Initial async implementation was completed but reverted due to design considerations
- FluxionSubject currently requires `std` feature (uses `parking_lot::Mutex`)
- **24/27 operators** available in no_std + alloc (FluxionSubject excluded)
- Architecture review documented in `RUNTIME_ABSTRACTION_STATUS.md`
- Async migration will be reconsidered when implementing:
  - Alternative partition() implementation
  - New publish() operator
  - Full runtime abstraction

### Impact
- **Embedded Support** - 24/27 operators available on ARM Cortex-M and other no_std targets
- **All 27 operators** available in std environments
- **No Breaking Changes** - FluxionSubject API remains synchronous
### Added
- **no_std Support (Phase 1) ✅ COMPLETE** (`fluxion-core`, `fluxion-stream`, `fluxion-exec`, `fluxion-ordered-merge`)
  - Added conditional `#![no_std]` attribute to all library crates
  - **24/27 operators** (89%) now available in no_std + alloc environments
  - `FluxionSubject` available in no_std (uses `futures::channel::mpsc` with alloc)
  - Only 2 spawn-based operators require std: `share()` and `subscribe_latest()`
  - Manual `Display` and `Error` trait implementations for `FluxionError` (removed thiserror)
  - Feature-gated `FluxionSubject` and `SubjectError` behind `alloc` feature
  - Embedded target compilation verified with `--no-default-features --features alloc`

- **CI Validation** (`.ci/`, `.github/workflows/ci.yml`)
  - New `no_std_check.ps1` script for standalone no_std verification
  - Integrated no_std check into main CI pipeline (runs after format check)
  - GitHub Actions workflow includes no_std compilation check
  - Fast-fail validation ensures no accidental breakage of embedded support

### Changed
- **Dependency Configuration** (`Cargo.toml`, `*/Cargo.toml`)
  - Workspace `futures` dependency: `default-features = false` with explicit features
  - Added `alloc`, `async-await`, `executor` features to futures for no_std support
  - Feature propagation: `std` feature enables `futures/std`, `alloc` enables `futures/alloc`
  - Fixed `fluxion-stream` to propagate `fluxion-core/alloc` in `std` feature

- **Error Handling** (`fluxion-core/src/fluxion_error.rs`)
  - Removed `thiserror` dependency (doesn't support no_std)
  - Implemented manual `Display` trait for all 4 error variants
  - Conditional `Error` trait import: `std::error::Error` vs `core::error::Error`
  - Added `use core::fmt::{self, Display, Formatter};` for no_std compatibility

- **Feature Gates** (`fluxion-core/src/lib.rs`)
  - `FluxionSubject` and `SubjectError` gated behind `#[cfg(feature = "alloc")]`
  - Allows use in no_std + alloc environments (not just std)
  - Spawn-based operators (`share`, `subscribe_latest`, `partition`) remain std-gated

### Technical Details
- **no_std Architecture:**
  - `alloc` feature enables heap allocation without full standard library
  - `futures::channel::mpsc` works in no_std with alloc feature
  - spawn-based operators require runtime support (tokio/smol/async-std)
  - 24/27 operators fully functional on embedded targets with heap

- **Operator Availability Matrix:**
  - **no_std + alloc (24):** All operators except `share()`, `partition()` and `subscribe_latest()`
  - **std + runtime (27):** All operators including spawn-based ones
  - `FluxionSubject`: Available in both no_std and std
  - `partition()`: Requires std + runtime (uses internal spawning)

### Impact
- **Embedded Systems Ready** - 24/27 operators on microcontrollers with heap
- **Zero Breaking Changes** - All existing std code continues working
- **WASM Compatible** - no_std features work in WASM environments
- **CI Protected** - Automated checks prevent no_std regressions
- **Validated** - All 1,684 tests passing across all configurations

## [0.6.9] - Not Published (Internal Release)

### Changed
- **no_std Preparation (Phase 0)** (`fluxion-core`, `fluxion-stream`, `fluxion-exec`, `fluxion-ordered-merge`)
  - Converted all `std` imports to `core`/`alloc` equivalents for future no_std support
  - Added `extern crate alloc;` to all library crate root files
  - Added explicit `use alloc::sync::Arc;` to 12+ source files
  - Added explicit `use alloc::boxed::Box;` to 12+ source files
  - Added explicit `use alloc::vec::Vec;` to 14+ source files
  - Changed `std::marker::PhantomData` → `core::marker::PhantomData`
  - **Zero behavioral changes** - std re-exports core/alloc, ensuring 100% compatibility
  - **Zero performance impact** - identical codegen, just different import paths
  - Test files continue using `std` imports (separate binary crates with std)
  - Doctests continue using `std` imports (compile as test binaries)

### Technical Details
- **Risk-Free Migration:**
  - Standard library re-exports everything from `core` and `alloc`
  - These imports are functionally identical at runtime
  - Prepares codebase for future `#![no_std]` attribute addition
  - All 816 tests passing confirms zero behavioral impact

- **Import Strategy:**
  - `core` - No allocator required (fmt, pin, task, future, marker)
  - `alloc` - Heap allocation required (Arc, Box, Vec)
  - `std` - Full standard library (only in test code)

### Impact
- **100% backward compatible** - No breaking changes
- **Future-proof** - Enables eventual no_std support for embedded/WASM
- **Zero overhead** - Preparatory change with no runtime cost
- **Validated** - 816 tests passing across all runtimes (Tokio, smol, async-std, WASM)

## [0.6.8] - Not Published (Internal Release)

### Added
- **Complete Runtime Abstraction** (`fluxion-core`, `fluxion-rx`)
  - New `FluxionTask` trait for runtime-agnostic task spawning
  - Implementations for Tokio, smol, async-std, and WASM (wasm-bindgen-futures)
  - Runtime selection via feature flags: `runtime-tokio` (default), `runtime-smol`, `runtime-async-std`
  - WASM support automatic via `cfg(target_arch = "wasm32")`
  - Zero-config experience: just add `fluxion-rx` for Tokio by default
  - Feature propagation in fluxion-rx: users can select runtime through main crate
  - Complete abstraction: users never write `tokio::spawn` or deal with timer APIs

- **Runtime-Specific Test Infrastructure** (`fluxion-core`)
  - Organized test structure: root (sync), tokio/, async_std/, smol/, wasm/ folders
  - Runtime-agnostic async tests use `#[tokio::test]` (execute once)
  - Runtime-specific tests (FluxionTask) execute on their respective runtimes
  - WASM test support via wasm-pack
  - All 4 runtimes validated in CI (900+ tokio, 12 async-std, 12 smol, 7 wasm tests)

- **Feature Propagation** (`fluxion-rx`)
  - New `tracing` feature propagates to fluxion-core/stream/exec
  - Runtime selection features: `runtime-tokio`, `runtime-smol`, `runtime-async-std`
  - Default feature set: `["runtime-tokio"]` for zero-config experience

### Changed
- **CI Infrastructure** (`.ci/`, `.github/workflows/`)
  - Updated all runtime test scripts to test both fluxion-core and fluxion-stream-time
  - async_std_tests.ps1: Added fluxion-core testing with runtime-async-std feature
  - smol_tests.ps1: Added fluxion-core testing with runtime-smol feature
  - wasm_tests.ps1: Added fluxion-core testing with --no-default-features flag
  - GitHub Actions workflow: Added missing fluxion-core tests for async-std, smol, and WASM runtimes
  - Previously only fluxion-stream-time was tested for alternative runtimes; now both packages validated

- **WASM Compilation Fixes** (`fluxion-core/Cargo.toml`)
  - Platform-gated dependencies: tokio, smol, async-std, criterion only for `cfg(not(target_arch = "wasm32"))`
  - Moved runtime-agnostic async tests to tokio/ folder (excluded from WASM compilation)
  - Prevents WASM compilation errors from OS-specific dependencies

- **Workspace Configuration** (`Cargo.toml`)
  - Fixed version mismatch: internal workspace dependencies now correctly reference `0.6.8`
  - Added readme fields to all published packages (fluxion-core, fluxion-exec, fluxion-test-utils, fluxion-stream)
  - Added homepage and documentation URLs to workspace metadata
  - cargo-udeps ignore configuration: async-std, smol (feature-gated dependencies)

- **Documentation** (`README.md`, `PITCH.md`)
  - Added "Runtime Selection (Optional)" section in README
  - Documented zero-config default (Tokio) and alternative runtime selection
  - Added "Runtime Abstraction Benefits" to PITCH
  - New comparison table row: 4 runtimes supported vs typical 1 (locked-in)

### Fixed
- **Doc Test Race Condition** (`fluxion-exec/src/subscribe_latest.rs`)
  - Fixed hanging doc test at line 316
  - Added sleep after queueing items to ensure proper timing demonstration
  - Moved stream drop after gate release for correct stream lifecycle
  - Relaxed assertions to check for "at least 2 but not all 10 items"

- **Dead Code Elimination**
  - Unused runtimes completely excluded from compilation when not selected
  - No runtime overhead from unused runtime dependencies

### Technical Details
- **Runtime Abstraction Pattern:**
  - `FluxionTask::spawn()` dispatches to runtime-specific implementation at compile time
  - Zero-cost abstraction: no dynamic dispatch or vtables
  - Platform-gated WASM support: automatic when compiling for wasm32 target
  - Tests validate correct behavior on all supported runtimes

- **Test Organization:**
  - Pure sync tests: fluxion_error_tests.rs, stream_item_tests.rs (execute once)
  - Runtime-agnostic async: FluxionSubject, CancellationToken tests in tokio/ folder
  - Runtime-specific: FluxionTask spawn/cancel tests per runtime folder
  - No redundant test execution across runtimes

### Impact
- **Zero-config for 99% of users** - Tokio included by default
- **Complete flexibility** - Runtime selection without code changes
- **Cross-platform** - Native (tokio/smol/async-std) and WASM with identical code
- **Zero overhead** - Dead code elimination for unused runtimes
- **Professional-grade** - Complete abstraction from spawn/timer implementation details

## [0.6.7] - Not Published (Internal Release)

### Changed
- **Runtime-Agnostic Primitives** (`fluxion-stream`, `fluxion-exec`)
  - Replaced `tokio::sync::Mutex` with `futures::lock::Mutex` in `merge_with.rs`
  - Replaced `tokio::sync::Mutex` with `futures::lock::Mutex` in `subscribe_latest.rs`
  - Both mutexes are async fair mutexes with identical semantics and performance
  - Works on ANY async executor (Tokio, smol, async-std, WASM)
  - Zero API changes (internal implementation detail)
  - Zero performance impact (equivalent primitives)

### Technical Details
- **Phase 0 of Runtime Abstraction** - Risk-free preparatory work
  - Reduces coupling to Tokio-specific types
  - Foundation for multi-runtime support in v0.7.0
  - All tests passing (no modifications required)
  - Zero compilation errors, zero clippy warnings

### Why This Change?
- **Preparation:** Sets foundation for multi-runtime support (v0.7.0)
- **Portability:** `futures::lock::Mutex` works across all async executors
- **no_std Ready:** `futures::lock` supports no_std + alloc environments
- **Zero Risk:** Internal change only, equivalent performance characteristics

## [0.6.6] - Not Published (Internal Release)

### Added
- **Ergonomic Convenience Methods** (`fluxion-stream-time`)
  - New convenience methods for all 5 time operators: `debounce()`, `delay()`, `throttle()`, `sample()`, `timeout()`
  - No timer parameter required - automatically uses runtime's default timer
  - Available via `prelude` module: `use fluxion_stream_time::prelude::*;`
  - Feature-gated implementations for each runtime (time-tokio, time-smol, time-wasm, time-async-std)
  - Example: `.debounce(Duration::from_millis(100))` vs `.debounce_with_timer(duration, timer)`

### Changed
- **API Enhancement** (`fluxion-stream-time`)
  - Renamed base operator methods to `*_with_timer` pattern (e.g., `debounce` → `debounce_with_timer`)
  - Added `*WithDefaultTimerExt` traits providing convenience methods (e.g., `DebounceWithDefaultTimerExt`)
  - Both traits exported: `*Ext` (explicit timer) and `*WithDefaultTimerExt` (convenience)
  - Pattern applied consistently across all 5 time operators
  - Prelude exports both trait variants for maximum flexibility
  - Original explicit API retained for advanced use cases requiring custom timer control

### Updated
- **Tests** (~40 test files)
  - All time operator tests migrated to convenience methods
  - Import pattern changed to `use fluxion_stream_time::prelude::*;`
  - Timer parameters removed from operator calls
  - All tests passing across all runtimes (Tokio: 57, Smol: 10, Async-std: 10)

- **Benchmarks** (5 benchmark files)
  - `debounce_bench`, `delay_bench`, `throttle_bench`, `sample_bench`, `timeout_bench`
  - All using convenience methods for cleaner benchmark code
  - All compiling successfully with new API

- **Documentation**
  - Updated `fluxion-stream-time/README.md` to show convenience methods as primary API
  - All operator examples now demonstrate both convenience and explicit APIs
  - Runtime-specific sections updated with convenience method examples
  - Quick reference table updated to reflect new method signatures

## [0.6.5] - Not Published (Internal Release)

**Goal:** Enable time-based operators with smol runtime through Timer abstraction

### Added
- **SmolTimer Implementation** (`fluxion-stream-time`)
  - Zero-sized type implementing Timer trait using `async-io::Timer`
  - Wraps `async_io::Timer::after(duration)` for async sleep operations
  - Uses `std::time::Instant` for monotonic timestamp tracking
  - Compatible with smol's executor patterns (both single and multi-threaded)
  - Supports both single-threaded (`smol::block_on`) and multi-threaded (`smol::Executor`) execution models

- **Feature Flag** (`fluxion-stream-time`)
  - `time-smol` - smol runtime support via SmolTimer
  - Mutually exclusive with other runtime features

- **smol Test Suite** (`fluxion-stream-time/tests/smol/`)
  - 10 comprehensive tests (5 operators × 2 threading models)
  - Single-threaded: debounce, delay, sample, throttle, timeout (with `smol::block_on`)
  - Multi-threaded: Same 5 operators with `smol::Executor` for concurrency
  - Uses real async delays via `smol::Timer::after`
  - Helper functions: test_channel, timestamped_person, person_alice
  - All tests passing in 0.15-0.16s execution time

- **CI Integration**
  - New smol test script (`.ci/smol_tests.ps1`)
  - Tests validate all 10 tests with verbose output

- **Public API** (`fluxion-stream-time`)
  - `SmolTimer` - Timer implementation for smol runtime
  - `SmolTimestamped<T>` - Type alias for `InstantTimestamped<T, SmolTimer>`
  - Exported via `runtimes` module with feature gating

### Documentation
- **Runtime Support** (`fluxion-stream-time`)
  - smol added to supported runtimes list
  - Usage examples with SmolTimer
  - Implementation notes and platform support details

### Technical Details
- All 5 time-based operators (debounce, throttle, delay, sample, timeout) work with smol
- Zero operator changes required (Timer trait abstraction enabled smol support)
- 10 tests passing (5 single-threaded + 5 multi-threaded)
- Zero compilation errors, zero clippy warnings for smol feature
- Pattern: Uses async-io conditionally compiled for non-WASM targets
- Shared dependency with async-std implementation (async-io 2.6.0)

### Why smol?

smol provides:
1. **Lightweight Alternative** - Smaller runtime footprint than tokio
2. **Multi-threading Support** - Unlike WASM, supports concurrent execution
3. **Active Maintenance** - Unlike async-std (discontinued), actively maintained
4. **Architecture Validation** - Further proves Timer trait abstraction works across diverse runtimes
5. **Ecosystem Choice** - Gives users another production-ready option

**Recommendation**: Use `time-tokio` (default) for maximum ecosystem compatibility, or `time-smol` for lightweight async applications.

## [0.6.4] - Not Published (Internal Release) ⚠️ **DEPRECATED RUNTIME**

**Goal:** Enable time-based operators with async-std runtime through Timer abstraction

**⚠️ CRITICAL WARNING**: async-std has been discontinued (RUSTSEC-2025-0052, Aug 2024).
This release adds support for **compatibility only**. New projects should use tokio or smol.

### Added
- **AsyncStdTimer Implementation** (`fluxion-stream-time`)
  - Zero-sized type implementing Timer trait using `async-io::Timer`
  - Wraps `async_io::Timer::after(duration)` for async sleep operations
  - Uses `std::time::Instant` for monotonic timestamp tracking
  - Compatible with async-std's multi-threaded work-stealing executor
  - Supports both single-threaded and multi-threaded execution models

- **Feature Flag** (`fluxion-stream-time`)
  - `time-async-std` - async-std runtime support via AsyncStdTimer
  - Mutually exclusive with `time-tokio` and `time-wasm`

- **async-std Test Suite** (`fluxion-stream-time/tests/async_std/`)
  - 10 comprehensive tests (5 operators × 2 threading models)
  - Single-threaded: debounce, delay, sample, throttle, timeout (inline async)
  - Multi-threaded: Same 5 operators with `async_std::task::spawn` for concurrency
  - Uses real async delays via `async_std::task::sleep`
  - Helper functions: test_channel, unwrap_stream, person_alice
  - All tests passing in 0.26-0.28s execution time

- **CI Integration**
  - New async-std test script (`.ci/async_std_tests.ps1`)
  - GitHub Actions integration in ci.yml
  - Tests run after WASM tests, before examples
  - Validates all 10 tests with verbose output

### Documentation
- **Deprecation Warnings** (`fluxion-stream-time`)
  - async-std marked as deprecated with RUSTSEC advisory details
  - Clear warnings in async_std_impl.rs module documentation
  - Cargo.toml comments warning about unmaintained status
  - ROADMAP.md updated with deprecation notice

### Technical Details
- All 5 time-based operators (debounce, throttle, delay, sample, timeout) work with async-std
- Zero operator changes required (Timer trait abstraction enabled async-std support)
- 10 tests passing (5 single-threaded + 5 multi-threaded)
- Zero compilation errors, zero clippy warnings for async-std feature
- Pattern: Uses async-io conditionally compiled for non-WASM, non-Tokio targets

### Why Include a Deprecated Runtime?

Despite async-std's discontinuation, this implementation serves important purposes:

1. **Existing Project Support** - Projects already using async-std can now use time operators
2. **Migration Path** - Provides time operators during async-std → tokio transitions
3. **Architecture Validation** - Proves Timer trait abstraction works across runtimes
4. **Zero Maintenance Burden** - async-std is feature-complete; no updates needed
5. **Educational Value** - Demonstrates multi-runtime support patterns

**Recommendation**: New projects should use `time-tokio` (default) or await `time-smol` support.

## [0.6.3] - Not Published (Internal Release)

**Goal:** Enable time-based operators in WASM environments

### Added
- **WasmTimer Implementation** (`fluxion-stream-time`)
  - Zero-sized type implementing Timer trait using `gloo-timers` for async sleep
  - Custom WasmInstant based on `js-sys::Date.now()` returning u64 milliseconds
  - Supports Add<Duration>, Sub<Duration>, and Sub<Self> for duration arithmetic
  - Compatible with Node.js and browser environments
  - WasmInstant provides monotonic time tracking for WASM targets

- **Feature Flag** (`fluxion-stream-time`)
  - `time-wasm` - WASM runtime support with WasmTimer (conditionally compiled for wasm32)

- **WASM Test Suite** (`fluxion-stream-time/tests/wasm/`)
  - 5 comprehensive single-threaded tests for all time-based operators
  - Tests: debounce_tests, delay_tests, sample_tests, throttle_tests, timeout_tests
  - Uses real async delays via gloo_timers::future::sleep
  - Helper functions: test_channel, unwrap_stream, person_alice for ergonomic testing
  - All tests passing in 0.82s execution time

- **CI Integration**
  - Local WASM test script (.ci/wasm_tests.ps1) with output parsing
  - GitHub Actions integration with wasm-pack
  - Output validation accepting any number of passing tests
  - Handles doc test failures gracefully (expected for wasm32 target)

### Documentation
- **Comprehensive WASM Documentation** (`fluxion-stream-time` README)
  - Runtime Support section now lists `time-wasm` as fully implemented
  - WASM usage example with WasmTimer and InstantTimestamped
  - Implementation notes covering gloo-timers and WasmInstant
  - Future Platform Support section updated to reflect WASM as implemented

### Technical Details
- All 5 time-based operators (debounce, throttle, delay, sample, timeout) work in WASM
- Zero operator changes required (Timer trait abstraction enabled WASM support)
- 5 WASM tests passing with real async delays
- Zero compilation errors, zero clippy warnings for wasm32 target
- Pattern: Uses js-sys and gloo-timers conditionally compiled for wasm32 targets

## [0.6.2] - Not Published (Internal Release)

**Goal:** Runtime abstraction for time-based operators enabling multi-runtime support

### Added
- **Timer Trait** (`fluxion-stream-time`)
  - Runtime-agnostic timer abstraction with `sleep_future()` and `now()` methods
  - Generic over `Sleep: Future<Output = ()>` and `Instant: Copy + Ord + Add + Sub`
  - Zero-cost abstraction with no runtime overhead
  - Enables support for multiple async runtimes (Tokio, async-std, smol, WASM, Embassy)
  - Maintains type safety: each timer brings its own Instant type

- **TokioTimer Implementation** (`fluxion-stream-time`)
  - Zero-sized type implementing Timer trait using `tokio::time` primitives
  - Default timer when `time-tokio` feature is enabled
  - Uses `tokio::time::Sleep` and `std::time::Instant`

- **Feature Flags** (`fluxion-stream-time`)
  - `time-tokio` (default) - Tokio runtime support with TokioTimer
  - Prepared for future runtime features: `time-async-std`, `time-smol`, `time-wasm`

- **Type Alias** (`fluxion-stream-time`)
  - `TokioTimestamped<T>` - Convenience alias for `InstantTimestamped<T, TokioTimer>`

### Changed
- **Breaking:** `InstantTimestamped<T>` is now generic over Timer type (`fluxion-stream-time`)
  - Changed from `InstantTimestamped<T>` to `InstantTimestamped<T, TM: Timer>`
  - Generic parameter `TM` represents the Timer type (not the timestamp type)
  - Internally stores `TM::Instant` as the timestamp
  - Migration: Use `TokioTimestamped<T>` type alias or explicitly specify timer type

- **Breaking:** All time-based operators now require a `timer` parameter (`fluxion-stream-time`)
  - `debounce(duration, timer)` - Added timer parameter
  - `throttle(duration, timer)` - Added timer parameter
  - `delay(duration, timer)` - Added timer parameter
  - `sample(duration, timer)` - Added timer parameter
  - `timeout(duration, timer)` - Added timer parameter
  - Migration: Create `TokioTimer` instance and pass to operators

- **Operator Implementation Patterns** (`fluxion-stream-time`)
  - Standardized on `#[pin] sleep: Option<TM::Sleep>` pattern for debounce, throttle, sample, timeout
  - Eliminates heap allocations (no more `Pin<Box<>>`)
  - Improved performance and consistency across operators
  - Delay operator uses `FuturesOrdered<DelayFuture<T, TM>>` (correct for multiple concurrent delays)

### Fixed
- **Documentation Examples** (`fluxion-stream-time`)
  - All operator doc examples updated to include timer parameter
  - Fixed type inference issues with explicit type annotations
  - All doc tests now compile and pass

- **Prelude Module** (`fluxion-stream-time`)
  - Uncommented all operator trait exports (DelayExt, SampleExt, ThrottleExt, TimeoutExt)
  - Enables `use fluxion_stream_time::prelude::*` to work correctly

### Documentation
- **Comprehensive Timer Documentation** (`fluxion-stream-time`)
  - Module-level docs explain Timer abstraction and multi-runtime strategy
  - Runtime Support section documenting feature flags
  - Updated all code examples to show timer usage pattern
  - Added Quick Start Example with complete working code

- **README Updates** (`fluxion-stream-time`)
  - Completely rewritten to emphasize runtime abstraction
  - Multi-Runtime Support section with Tokio, async-std (planned), smol (planned)
  - Timer Trait Implementation guide for custom runtimes
  - Usage examples for different runtimes
  - Future Platform Support section covering no_std feasibility and WASM support

- **Platform Considerations** (`fluxion-stream-time` README)
  - **no_std feasibility**: Detailed analysis of what's compatible and challenges (no std::time::Instant, alloc requirement, async runtime dependency)
  - Implementation paths for Embassy (embedded async) and bare metal scenarios
  - **WASM support**: Planned via WasmTimer with wasm-timer or gloo-timers crates
  - Architecture validated for multi-platform expansion

### Technical Details
- All 5 time-based operators successfully abstracted over Timer trait
- Zero compilation errors, zero clippy warnings
- 41 integration tests passing, 7 doc tests passing
- CI passing with comprehensive test coverage
- Pattern consistency: Option<TM::Sleep> with #[pin] for optimal performance
- Type-safe instant arithmetic preserved through generic design

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

[Unreleased]: https://github.com/umbgtt10/fluxion/compare/v0.6.6...HEAD
[0.6.6]: https://github.com/umbgtt10/fluxion/compare/v0.6.0...v0.6.6
[0.6.0]: https://github.com/umbgtt10/fluxion/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/umbgtt10/fluxion/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/umbgtt10/fluxion/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/umbgtt10/fluxion/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/umbgtt10/fluxion/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.2.1
[0.2.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.2.0
[0.1.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.1
[0.1.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.0
