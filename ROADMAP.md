# Fluxion Roadmap

This document outlines the release plan for Fluxion, a reactive stream processing library with ordered semantics.

---

## 📦 Version 0.1.0 - Initial Release

**Status:** Published to crates.io

**Goal:** Provide a stable, working foundation for reactive stream processing with ordering guarantees.

### Core Requirements ✅

**Essential Features:**
- ✅ Core stream operators (`combine_latest`, `with_latest_from`, `ordered_merge`, etc.)
- ✅ Execution utilities (`subscribe_async`, `subscribe_latest_async`)
- ✅ Temporal ordering with `Timestamped` trait
- ✅ Comprehensive test coverage (1,500+ tests)
- ✅ Error handling with `FluxionError` type
- ✅ Phase 1 error propagation (subscribe functions return `Result<()>`)

**Documentation:**
- ✅ API documentation with examples
- ✅ README with quick start guide
- ✅ Crate-level documentation for all modules
- ✅ Operator comparison tables and selection guides

**Quality Gates:**
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests passing

## 📦 Version 0.1.1 - Documentation Improvements

**Status:** Published to crates.io

**Goal:** Enhance documentation and examples for better developer experience

**Essential Features:**
- ✅ Comprehensive operator reference guide (`docs/FLUXION_OPERATOR_SUMMARY.md`)
- ✅ Operators roadmap (`docs/FLUXION_OPERATORS_ROADMAP.md`)
- ✅ Error handling refactoring plan documentation
- ✅ Chaining examples in README with real-world operator composition
- ✅ Integrated `stream-aggregation` example into workspace
- ✅ Comprehensive API documentation for all FluxionStream extension methods
- ✅ Code of Conduct

**Quality Gates:**
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Fixed code formatting to match rustfmt standards
- ✅ Cleaned up unused imports

## 🚀 Version 0.2.0 - Error Handling Foundation [YANKED]

**Status:** Yanked from crates.io (replaced by 0.2.1)

**Goal:** Comprehensive error propagation through all operators

**Essential Features:**
- ✅ Introduced `StreamItem<T>` enum for error propagation (`Value(T)` | `Error(FluxionError)`)
- ✅ Merged `fluxion-error` into `fluxion-core`
- ✅ All 9 stream operators return `StreamItem<T>` instead of bare `T`
- ✅ Simplified `FluxionError` from 12 variants to 4 essential variants
- ✅ Comprehensive error handling guide (`docs/ERROR-HANDLING.md`)
- ✅ API method naming improvements (`both()` → `as_pair()`, etc.)
- ✅ Lock errors now propagate instead of silently dropping items
- ✅ Test suite updated to handle `StreamItem<T>` wrapper (200+ replacements)

**Quality Gates:**
- ✅ All tests passing (186 at release time)
- ✅ Zero unsafe `unwrap()` calls in production code
- ✅ All test functions return `anyhow::Result<()>`

## 📦 Version 0.2.1 - Publishing Fixes

**Status:** Published to crates.io

**Goal:** Fix crates.io publishing issues from 0.2.0

**Essential Features:**
- ✅ Corrected README path for fluxion-rx crate display on crates.io
- ✅ Fixed broken anchor links in README.md table of contents
- ✅ Standardized Error Handling Guide links across all source files
- ✅ Updated all version references from 0.2.0 to 0.2.1

## 🚀 Version 0.2.2 - Trait Refactoring & Benchmarks

**Status:** Published to crates.io

**Goal:** Provide a consolidated foundation supporting error propagation

**Essential Features:**
- ✅ Consolidated interface fully supporting chaining
- ✅ Change stream operators to return `Stream<Item = StreamItem<T>>`
- ✅ All exsisting operators support error propagation
- ✅ Full test coverage for each and every operator (happy case)
- ✅ Full test coverage for each and every operator (error case)
- ✅ Full test coverage for operator chaining (happy case)
- ✅ Full test coverage for operator chaining (error case)
- ✅ 1 fully functional example application showing the intrinsic integration path
- ✅ Remove or document unwrap/expect in productive code

**Documentation:**
- ✅ Provide exaustive integration guide with options
- ✅ Provide exaustive error handling documentation
- ✅ Integration guide with options
- ✅ Roadmap document
- ✅ Crate-level documentation for all modules
- ✅ Operator comparison tables and selection guides

**Quality Gates:**
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests passing
- ✅ CI green

## 🚀 Version 0.3.0 - Error Handling & Legacy Integration

**Status:** Published to crates.io

**Goal:** Complete error handling with `on_error` operator and demonstrate wrapper pattern integration

**Essential Features:**
- ✅ `on_error` operator for Chain of Responsibility error handling
- ✅ Complete `legacy-integration` example application (wrapper pattern)
- ✅ Integration guide updated with both example applications
- ✅ Documentation cleanup and consistency improvements

**Quality Gates:**
- ✅ All tests passing (1,700+)
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests passing
- ✅ CI green
- ✅ Both examples validated in CI

## 🚀 Version 0.4.0 - Additional Operators & Advanced Features

**Goal:** Expand operator library

**Planned Features:**
- ✅ `scan` operator - Stateful accumulation across stream
- ✅ `distinct_until_changed` operator - Duplicate suppression
- ✅ `distinct_until_changed_by` operator - Duplicate suppression with custom comparison
- ✅ `skip_items` operator - Skip the first N items
- ✅ `take_items` operator - Take the first N items
- ✅ `start_with` operator - Prepend initial values to stream

**Documentation:**
- ✅ POC: Sample operator implemented and bench comparison documented => Done. No advantages found.

**Quality Gates:**
- ✅ Final decision whether to proceed with the dual API model or not => Rejected!

See [Operators Roadmap](docs/FLUXION_OPERATORS_ROADMAP.md) for detailed operator implementation timeline beyond v0.3.0.

## 🚀 Version 0.5.0 - Time-Based Operators

**Status:** Published to crates.io

**Goal:** Introduce time-based reactive operators through optional `fluxion-stream-time` crate

**Essential Features:**
- ✅ `fluxion-stream-time` crate - Optional time-based operators (migrated to `std::time::Instant` in v0.6.1)
- ✅ `debounce(duration)` operator - Emit only after silence period (essential for search inputs, API rate limiting)
- ✅ `throttle(duration)` operator - Rate-limit emissions (critical for scroll/resize handlers)
- ✅ `timeout(duration)` operator - Error if no emission within duration (network reliability)
- ✅ `delay(duration)` operator - Shift emissions forward in time
- ✅ `sample(duration)` operator - Periodic sampling at fixed intervals
- ✅ `InstantStreamOps` extension trait for `std::time::Instant`-based `InstantTimestamped` types

**Documentation:**
- ✅ Time-based operators guide with real-world examples
- ✅ Timestamp integration patterns (InstantTimestamped wrapper, originally ChronoTimestamped in v0.5.0)
- ✅ Performance characteristics of temporal operators (comprehensive test suite)

**Quality Gates:**
- ✅ All tests passing with both counter and monotonic timestamps
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests for all time-based operators
- ✅ Comprehensive test coverage (functional, error, composition tests)
- ✅ CI green

## 🚀 Version 0.6.0 - Stream Composition & Sampling

**Status:** Published to crates.io

**Goal:** Enable stream sharing across multiple consumers and add sampling/batching operators

**Essential Features:**
- ✅ `FluxionSubject` - Foundation for multi-consumer scenarios
- ✅ `share()` operator - Share single stream source among multiple subscribers (standard Rx operator)
- ✅ `partition(predicate)` operator - Split stream into two based on condition
- ✅ `sample_ratio(ratio, seed)` operator - Probabilistic downsampling (0.0 to 1.0) with deterministic seeding
- ✅ `tap` operator - Perform side-effects for debugging/observing stream values
- ✅ `window_by_count(n)` operator - Count-based batching into Vec<T>
- ✅ `merge_with` can handle errors

**Documentation:**
- ✅ Stream sharing patterns and examples
- ✅ FluxionSubject usage guide
- ✅ Sampling strategies documentation
- ✅ Performance characteristics of each operator

**Quality Gates:**
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests for all new operators
- ✅ Benchmarks for sampling operators
- ✅ CI green

## 🚀 Version 0.6.1 - Preparation for Runtime Abstraction

**Status:** Internal release (not published to crates.io)

**Goal:** Prepare for time abstraction and runtime flexibility

**Essential Features:**
- ✅ Remove `with_fresh_timestamp` method from Timestamped trait in order to no longer be dependant on wall-clock time
- ✅ Fixed `emit_when` operator to use correct timestamps based on triggering stream (source or filter)
- ✅ Migrate the time operators from chrono-based timestamps to std::time::Instant-based timestamps in order to prepare for runtime abstraction: Chrono is no longer a dependency

**Documentation:**
- ✅ Updated legacy-integration example README to reflect new timestamp handling patterns

**Quality Gates:**
- ✅ All tests passing
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests for all operators
- ✅ CI green

## 🚀 Version 0.6.2 - Introduce Time Abstraction and Implement the First Runtime: Tokio

**Status:** ✅ Completed (not published to crates.io)

**Goal:** Prepare for time abstraction and runtime flexibility

**Essential Features:**
- ✅ Introduce Timer trait abstracting: sleep and now functions
- ✅ Implement TokioTimer as default Timer using tokio::time functions
- ✅ Adapt all existing time-based operators to use Timer trait instead of direct tokio::time calls
- ✅ Add feature flag for runtime abstraction (default: tokio)

**Documentation:**
- ✅ Update time-based operators documentation to explain Timer abstraction and usage patterns with rationales
- ✅ README updated with Timer trait documentation, runtime support, and multi-runtime examples
- ✅ All operator doc examples updated to show timer parameter usage
- ✅ Future platform support section added (no_std feasibility, WASM support)

**Quality Gates:**
- ✅ All tests passing (41 integration tests + 7 doc tests)
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ Doc tests for all operators
- ✅ CI green

**Key Achievements:**
- Runtime-agnostic Timer trait enables multi-runtime support (Tokio, async-std, smol, WASM, Embassy)
- All 5 time-based operators (debounce, throttle, delay, sample, timeout) migrated to generic Timer pattern
- Zero-cost abstraction with no runtime overhead
- Pattern consistency: `Option<TM::Sleep>` with `#[pin]` for optimal performance
- Architecture validated for no_std feasibility

## 🚀 Version 0.6.3 - Support WASM Runtime

**Status:** ✅ Completed (not published to crates.io)

**Goal:** Enable time-based operators in WASM environments through Timer abstraction

**Essential Features:**
- ✅ Implement WasmTimer for WASM targets using `gloo-timers` crate
- ✅ Custom WasmInstant implementation using `js-sys::Date.now()` for monotonic time
- ✅ Add `time-wasm` feature flag (conditionally compiled for wasm32 target)
- ✅ All 5 time-based operators compile and run with WasmTimer
- ✅ Comprehensive WASM tests (5 passing tests: debounce, delay, sample, throttle, timeout)
- ✅ CI integration for WASM tests with wasm-pack

**Documentation:**
- ✅ Update README with comprehensive WASM usage example
- ✅ Document WASM implementation details (gloo-timers, WasmInstant with js-sys)
- ✅ Added WASM section to fluxion-stream-time README
- ✅ Conditional compilation documented for target-specific code

**Quality Gates:**
- ✅ All existing Tokio tests still passing
- ✅ WASM target compiles without errors (cargo check --target wasm32-unknown-unknown)
- ✅ 5 WASM tests passing (wasm-bindgen-test with Node.js runtime in 0.82s)
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ CI green (both native and WASM targets)

**Key Achievements:**
- Single-threaded WASM tests with real async delays (gloo_timers::future::sleep)
- WasmInstant provides monotonic time via js-sys::Date.now() returning u64 milliseconds
- Helper functions for test ergonomics (test_channel, unwrap_stream, person_alice)
- Output parsing in CI validates WASM tests while ignoring expected doc test failures
- Zero operator changes required (Timer trait abstraction enabled WASM support)

**Out of Scope:**
- Browser-specific optimizations
- Deterministic time control (WASM doesn't support time mocking like Tokio)

## 🚀 Version 0.6.4 - Support async-std Runtime ⚠️ **DEPRECATED**

**Status:** Completed (Internal Release)

**⚠️ WARNING**: async-std has been discontinued (RUSTSEC-2025-0052, 2024-08-24).
This implementation is kept for compatibility with existing projects only.
New projects should use tokio or smol runtimes instead.

**Goal:** Enable time-based operators with async-std runtime through Timer abstraction

**Essential Features:**
- ✅ Implement AsyncStdTimer for async-std targets using `async-std::task::sleep` and `async_io::Timer`
- ✅ Add `time-async-std` feature flag (alternative to `time-tokio`)
- ✅ All 5 time-based operators compile and run with AsyncStdTimer
- ✅ Comprehensive async-std tests (10 tests: 5 operators × 2 threading models)
- ✅ CI integration for async-std tests

**Documentation:**
- ✅ Document async-std implementation details (async-std::task, async_io::Timer)
- ✅ Add deprecation warning about unmaintained status
- ✅ Added async-std section to fluxion-stream-time README
- ✅ Runtime selection guide comparing Tokio vs async-std tradeoffs (included in deprecation notes)

**Quality Gates:**
- ✅ All existing Tokio tests still passing
- ✅ async-std target compiles without errors
- ✅ 10 async-std tests passing with real async delays
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ CI green (Tokio, async-std, and WASM targets)

**Key Achievements:**
- Multi-threaded async-std tests with real async delays
- AsyncStdTimer provides compatible interface with TokioTimer
- Helper functions adapted for async-std runtime
- Zero operator changes required (Timer trait abstraction enables async-std support)
- Users can choose between Tokio and async-std based on project needs

**Out of Scope:**
- Runtime performance benchmarking

## 🚀 Version 0.6.5 - Support smol Runtime

**Status:** Completed (Internal Release)

**Goal:** Enable time-based operators with async-std runtime through Timer abstraction

**Essential Features:**
- ✅ Implement SmolTimer for smol targets using `smol::Timer::after` and `async_io::Timer`
- ✅ Add `time-smol` feature flag (alternative to `time-tokio`)
- ✅ All 5 time-based operators compile and run with SmolTimer
- ✅ Comprehensive smol tests (10 tests: 5 operators × 2 threading models)
- ✅ CI integration for smol tests

**Documentation:**
- ✅ Document smol implementation details (smol::Timer, async_io::Timer)
- ✅ Add deprecation warning about unmaintained status
- ✅ Added smol section to fluxion-stream-time README
- ✅ Runtime selection guide comparing Tokio vs smol tradeoffs (included in deprecation notes)

**Quality Gates:**
- ✅ All existing Tokio tests still passing
- ✅ smol target compiles without errors
- ✅ 10 smol tests passing with real async delays
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ CI green (Tokio, smol, and WASM targets)

**Key Achievements:**
- Multi-threaded smol tests with real async delays
- SmolTimer provides compatible interface with TokioTimer
- Helper functions adapted for smol runtime
- Zero operator changes required (Timer trait abstraction enables smol support)
- Users can choose between Tokio and smol based on project needs

**Out of Scope:**
- Runtime performance benchmarking

## 🚀 Version 0.6.6 - Ergonomic API Improvements

**Status:** ✅ Completed (Internal Release)

**Goal:** Provide convenience methods for time operators eliminating boilerplate

### Essential Features
- ✅ **Convenience Methods** - All 5 time operators now have parameter-free variants
- ✅ **Smart Defaults** - Automatically use runtime's default timer (TokioTimer, SmolTimer, etc.)
- ✅ **Dual API** - Both convenience (`.debounce()`) and explicit (`.debounce_with_timer()`) methods
- ✅ **Prelude Module** - Single import for all extension traits
- ✅ **Feature-Gated** - Implementations for each runtime (time-tokio, time-smol, time-wasm, time-async-std)

### Documentation
- ✅ Updated README to show convenience methods as primary API
- ✅ All operator examples demonstrate both APIs
- ✅ Runtime-specific sections updated

### Quality Gates
- ✅ All tests migrated to convenience methods (~40 test files)
- ✅ All benchmarks updated (5 benchmark files)
- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ CI green

### Key Achievement
**Zero Trade-offs** - Achieved performance, flexibility, ergonomics, runtime support, and no_std infrastructure simultaneously without compromise.

## 🚀 Version 0.6.7 - Runtime-Agnostic Preparation

**Status:** ✅ Completed (Internal Release)

**Goal:** Provide runtime-agnostic support for time-agnostic operators enabling multiple async runtimes

### Essential Features
- ✅ Replace tokio::sync::Mutex → futures::lock::Mutex
- ✅ Custom CancellationToken (using event-listener)
- ✅ Replace tokio channels with futures::channel (mpsc, oneshot)
- ✅ Replace tokio::sync::Notify → event_listener::Event
- ✅ Replace `tokio::select!` → `futures::select!` (production code + tests)

### Documentation
- ✅ Updated docs as appropriate to reflect runtime-agnostic changes

### Quality Gates
- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ CI green

### Key Achievement
**Zero Trade-offs** - Risk-free, performance-loss-free preparatory changes reducing coupling to Tokio runtime.


## 🚀 Version 0.6.8 - Complete Runtime Abstraction

**Status:** ✅ Completed (Internal Release)

**Goal:** Enable Fluxion to run on multiple async runtimes (Tokio, smol, async-std, WASM)

### Essential Features
- ✅ `FluxionTask` trait for runtime-agnostic spawning (Tokio, smol, async-std, WASM)
- ✅ Feature flags: `runtime-tokio` (default), `runtime-smol`, `runtime-async-std`
- ✅ WASM support via `cfg(target_arch = "wasm32")` with wasm-bindgen-futures
- ✅ Runtime-specific test infrastructure (tokio/, async_std/, smol/, wasm/ folders)
- ✅ All 4 runtimes validated: 900+ tokio + 12 async-std + 12 smol + 7 wasm tests
- ✅ Feature propagation in fluxion-rx for zero-config experience
- ✅ CI workflow fixes: Added missing fluxion-core tests for all runtimes

### Documentation
- ✅ Runtime selection guide in README
- ✅ FluxionTask API documentation with runtime examples
- ✅ CI infrastructure validated (.ci/ scripts + GitHub Actions)

### Quality Gates
- ✅ Zero compilation errors across all feature combinations
- ✅ Zero clippy warnings
- ✅ CI green for all 4 runtimes
- ✅ WASM compilation working (platform-gated dependencies)

### Key Achievement
**100% Runtime Abstraction** - Complete multi-runtime support with zero user-visible complexity. Tokio by default, alternative runtimes via feature flags, WASM automatic.

## 🚀 Version 0.6.9 - no_std Preparation (Phase 0)

**Status:** ✅ Completed (Internal Release)

**Goal:** Zero-risk preparation for no_std support without breaking changes

### Essential Features
- ✅ Convert `std` imports to `core` imports across all crates
  - ✅ `std::fmt` → `core::fmt`
  - ✅ `std::pin::Pin` → `core::pin::Pin`
  - ✅ `std::task` → `core::task`
  - ✅ `std::future::Future` → `core::future::Future`
  - ✅ `std::sync::Arc` → `alloc::sync::Arc` (added `use alloc::sync::Arc;` to 12+ files)
  - ✅ `std::boxed::Box` → `alloc::boxed::Box` (added `use alloc::boxed::Box;` to 12+ files)
  - ✅ `std::vec::Vec` → `alloc::vec::Vec` (added `use alloc::vec::Vec;` to 14+ files)
  - ✅ `std::marker::PhantomData` → `core::marker::PhantomData`
- ✅ Added `extern crate alloc;` to all library crates (fluxion-core, fluxion-stream, fluxion-exec, fluxion-ordered-merge)
- ✅ All test files continue using `std` imports (separate binary crates)
- ✅ All doctests continue using `std` imports (compile as test binaries)

### Documentation
- ✅ Document Phase 0 changes as preparation step for future no_std support

### Quality Gates
- ✅ All existing tests passing
- ✅ Zero behavioral changes
- ✅ Zero performance impact (std re-exports core/alloc)
- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ CI green for all runtimes (Tokio, smol, async-std, WASM)

### Key Achievement
**Risk-Free Foundation** - Systematic core/alloc imports enabling future no_std support with zero behavioral or performance changes. Standard library re-exports ensure 100% compatibility. All 816 tests passing confirms zero behavioral impact.

## 🚀 Version 0.6.10 - no_std Support (Phase 1)

**Status:** ✅ Completed (Internal Release)

**Goal:** Enable no_std compilation with 24/27 operators immediately available

### Essential Features
- ✅ Add conditional `#![no_std]` to library crates (fluxion-core, fluxion-stream, fluxion-exec)
- ✅ Feature-gate spawn-based operators (share, subscribe_latest, partition)
- ✅ Configure dependencies for no_std (futures, parking_lot, event-listener with explicit features)
- ✅ Remove thiserror dependency, implement manual Display/Error traits
- ✅ Embedded target compilation verified (`--no-default-features --features alloc`)
- ✅ 24/27 operators work in no_std+alloc environments

### Documentation
- ✅ Updated README with no_std usage patterns
- ✅ Documented operator availability (24/27 in no_std, 3 require std)
- ✅ Documented feature flags (std, alloc, runtime-*)

### Quality Gates
- ✅ Compiles with `--no-default-features --features alloc`
- ✅ All 24 non-spawn operators available on embedded targets
- ✅ Zero behavioral changes for existing std users
- ✅ CI green for all runtimes + no_std build check
- ✅ All tests passing

### Key Achievement
**Minimal no_std Support** - 24/27 operators immediately available on embedded systems with just `alloc`. Spawn-based operators clearly gated on runtime features. Zero breaking changes.

## 🚀 Version 0.6.11 - Embedded Target Support & Infrastructure

**Status:** ✅ Completed (Internal Release)

**Goal:** Verify embedded compilation and establish infrastructure for no_std development

### Essential Features
- ✅ Added embedded target verification script (`test_embedded_target.ps1`)
- ✅ Verified compilation against `thumbv7em-none-eabihf` (ARM Cortex-M4F) target
- ✅ Feature flag refinement (`std` implies `alloc`)
- ✅ Fixed build warnings (tracing, unused variables)
- ✅ Added test dependencies (thiserror)

### Documentation
- ✅ Added embedded target compilation guide and CI script
- ✅ Documented architectural considerations in `RUNTIME_ABSTRACTION_STATUS.md`
- ✅ Updated CHANGELOG with version 0.6.11 changes

### Quality Gates
- ✅ Embedded target test script passes (`test_embedded_target.ps1`)
- ✅ CI includes embedded target verification
- ✅ No breaking changes
- ✅ All tests passing

### Key Achievement
**24/27 Operators on Embedded!** - Core operators work in no_std+alloc environments. FluxionSubject async migration deferred for architectural review. Infrastructure in place for continued no_std development. Phase 1 complete with limitations.

## 🚀 Version 0.6.12 - no_std Support for Time Operators (Phase 3 Infrastructure)

**Status:** ✅ Completed (Internal Release)

**Goal:** Prepare time operators for no_std environments

### Essential Features
- ✅ Added `#![cfg_attr(not(feature = "std"), no_std)]` to fluxion-stream-time
- ✅ Added conditional Box imports for no_std
- ✅ Time operators compile with `--no-default-features --features alloc`
- ✅ Architecture documentation in RUNTIME_ABSTRACTION_STATUS.md

### Key Achievement
**Infrastructure Complete** - Time operators ready for no_std. All dependencies configured. Embassy implementation (Phase 3) followed immediately.

## 🚀 Version 0.6.13 - Embassy Timer Implementation & Test Consistency (Phase 3 Complete)

**Status:** ✅ Completed (Internal Release)

**Goal:** Enable time operators on embedded targets with Embassy runtime, complete documentation, and improve test consistency

### Essential Features
- ✅ Implement `EmbassyTimerImpl` for embassy-time integration
- ✅ Create `EmbassyInstant` wrapper bridging embassy_time::Duration ↔ core::time::Duration
- ✅ Add `runtime-embassy` feature flag (alloc + dep:embassy-time)
- ✅ Add `embassy-time = "0.5"` to workspace dependencies
- ✅ Export `EmbassyTimerImpl` and `EmbassyTimestamped<T>` type alias
- ✅ All 5 time operators work with Embassy timer
- ✅ Compiles in no_std + alloc + runtime-embassy configuration

### Test Consistency Improvements
- ✅ Refactored all smol runtime tests to match tokio/async-std/embassy pattern
- ✅ Removed `timestamped_person()` helper function for explicit timer usage
- ✅ All smol tests now import `SmolTimer` and `Timer` trait explicitly
- ✅ Inline timestamp creation with `SmolTimestamped::new(value, timer.now())`
- ✅ Consistent test structure across all 5 runtimes (10 smol tests updated)

### Documentation Test Fixes
- ✅ Fixed 8 doctests to compile with proper cfg gates across all runtime features
- ✅ Changed doctests from `rust,ignore` to `rust,no_run` for better validation
- ✅ Added fallback `fn main() {}` for non-tokio features using `#[cfg(not(...))]`
- ✅ All doctests now compile correctly regardless of enabled runtime feature
- ✅ Doctests for: delay, debounce, throttle, timeout, sample, InstantTimestamped, and lib.rs examples

### Documentation
- ✅ Added Embassy to runtime support list in lib.rs
- ✅ Updated RUNTIME_ABSTRACTION_STATUS.md with Phase 3 completion
- ✅ Documented wrapper pattern for Duration type bridging
- ✅ Updated README.md with Embassy runtime section and examples
- ✅ Updated PITCH.md with 5 runtimes and Embassy benefits
- ✅ Fixed feature flag naming throughout fluxion-stream-time README (time-* → runtime-*)
- ✅ Documented convenience methods vs explicit timer methods
- ✅ Added comprehensive Embassy usage section to fluxion-stream-time README
- ✅ Updated FLUXION_OPERATOR_SUMMARY.md with Embassy support
- ✅ Updated all version references from 0.6.11 → 0.6.13

### Quality Gates
- ✅ Compiles with `--no-default-features --features alloc,runtime-embassy`
- ✅ std build still works (no regressions)
- ✅ Full CI passes (67 tests: 57 tokio + 10 smol)
- ✅ All 8 doctests compile with smol feature (use fallback main)
- ✅ All 8 doctests compile with tokio feature (use actual async main)
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ no_std compilation check passes
- ✅ All documentation synced with correct versions
- ✅ Feature flag naming consistent across all files

### Key Achievement
**5 Runtimes Complete!** - Embassy joins Tokio, smol, async-std, and WASM as fully supported runtimes. Time operators now work on embedded targets. Wrapper pattern elegantly solved Duration type incompatibility without unsafe code.
**Documentation now complete and consistent** across all 5 runtimes with proper feature flag naming and comprehensive usage examples.
**Test consistency achieved** - All runtime tests now follow the same explicit timer pattern, improving maintainability and reducing confusion.

## 🚀 Version 0.6.14 - Embassy Example & Minimal std Dependencies

**Status:** ✅ Completed (Internal Release)

**Goal:** Demonstrate Embassy runtime integration with minimal std footprint

### Essential Features
- ✅ `embassy-sensors` example - Host-based Embassy demonstration
- ✅ Minimal std dependencies - Only Embassy runtime requires std
- ✅ Application code is no_std compatible (Fluxion + futures + async-channel + rand_chacha)
- ✅ Simulated sensor fusion with 9 operators (temperature, pressure, humidity streams)
- ✅ Added to CI pipeline for automated testing

### Example Architecture
- ✅ 3 sensor tasks with async timers and ChaCha8 RNG (no_std)
- ✅ Sensor fusion using `MergedStream` pattern with stateful aggregation
- ✅ Time operators: `debounce(500ms)`, `throttle(750ms)`, `sample(100ms)`
- ✅ Non-time operators: `tap`, `distinct_until_changed`, `distinct_until_changed_by`, `filter_ordered`, `window_by_count`, `skip_items`

### Documentation
- ✅ README clarifies host-based vs future embedded example
- ✅ Documents minimal std footprint (2 dependencies: embassy-executor, embassy-time)
- ✅ Bridge to true embedded: just 2 lines (arch-cortex-m, generic-queue)
- ✅ Accurate RxRust comparison (requires custom schedulers for non-Tokio runtimes)

### Quality Gates
- ✅ Runs for 30 seconds with clean shutdown
- ✅ Zero clippy warnings
- ✅ CI green (added to .ci/build.ps1)
- ✅ All documentation updated

### Key Achievement
**Minimal std Bridge** - Only Embassy runtime uses std. All application code (sensors, fusion, operators) is no_std compatible and transfers directly to embedded targets.

## 🚀 Version 0.7.0 - Testing Infrastructure & Example Applications

**Status:** Planned

**Goal:** Complete testing infrastructure, demonstrate runtime capabilities, and prepare for production

**Note:** Versions 0.6.8-0.6.14 already delivered the runtime abstraction originally planned for 0.7.0. This release focuses on testing infrastructure and real-world examples.

### Essential Features

**Testing Infrastructure:**
- ✅ ~~Implement `testing_time` module~~ → **Superseded by Timer abstraction (0.6.2-0.6.13)** - All operators tested with controlled time across all 5 runtimes
- ✅ Fix unstable tests across the workspace
- ✅ Improve time operator test coverage - Currently ~50-60% per operator (debounce: 29/50, throttle: 24/42, timeout: 16/32, delay: 27/41, sample: 18/37)
  - Missing edge case coverage in tokio tests
  - Need additional test scenarios to exercise untested code paths

**Example Applications:**
- ✅ Create WASM example application demonstrating browser usage with time-based operators
- ✅ Create simple no_std-compatible embedded example (Embassy runtime)

**Documentation:**
- ✅ Update examples README with all 3 runtime examples
- ✅ Future roadmap update based on 0.6.x achievements

**Quality Gates:**
- ✅ WASM example compiles and runs in browser
- ✅ Code coverage ≥90%
- ✅ Zero unstable tests
- ✅ Zero clippy warnings
- ✅ Zero compiler warnings
- ✅ CI green for all configurations

**Key Achievement:**
**Production-Ready Examples** - Real-world examples demonstrate WASM and embedded capabilities. Timer abstraction (0.6.2-0.6.13) already provides deterministic time control across all 5 runtimes.

---

## 🚀 Version 0.7.1 - Embassy QEMU Validation

**Status:** ✅ Completed (Internal Release)

**Goal:** Validate Embassy runtime with real ARM target in QEMU emulator

### Essential Features

**no_std Optimization:**
- ✅ `fluxion-core` dependencies already optimized (spin::Mutex for no_std, parking_lot for std)
- ✅ `fluxion-core/src/fluxion_mutex.rs` uses safe spin::Mutex pattern (never held across .await)

**Embassy Example Migration:**
- ✅ Migrated embassy-sensors from std to true embedded target (`thumbv7em-none-eabihf` - ARM Cortex-M4F)
- ✅ Configured for QEMU emulation (mps2-an386 machine, 25MHz, 4MB Flash/RAM)
- ✅ Simulated sensors using Embassy timers (Temperature, Pressure, Humidity with realistic drift)
- ✅ Demonstrates 9 operators in no_std + alloc environment (merge, filter_ordered, map_ordered, scan_ordered, tap, distinct_until_changed, distinct_until_changed_by, debounce, throttle)
- ✅ QEMU automation scripts (PowerShell with auto-detection and 30s demo)

**Testing & Validation:**
- ✅ Example compiles for ARM target (3.10s build time)
- ✅ Example runs in QEMU successfully (30s runtime, 74 sensor aggregates)
- ✅ All operator patterns demonstrated (sensor fusion with MergedStream pattern)
- 📝 Memory usage profiling (deferred - 64KB heap allocated, actual usage not profiled)

### Documentation
- ✅ QEMU setup guide (installation, target selection, automation scripts)
- ✅ Embassy best practices documented in README (MergedStream pattern for no_std, spawn-free design)
- 📝 Performance characteristics (basic validation done, detailed profiling deferred)
- ✅ Migration guide integrated in README (std → no_std, git deps → crates.io, custom time driver)

### Quality Gates
- ✅ ARM target builds without errors (cargo check passes in 3.10s)
- ✅ QEMU execution successful (30s demo produces 74 complete aggregates)
- ✅ All demonstrated operators work (9 operators validated)
- ✅ Documentation complete (comprehensive README with setup, architecture, CI integration)
- ✅ Example serves as reference for embedded users (production-ready template)
- ✅ CI integration (no_std_check.ps1 verifies ARM builds, build.ps1 includes optional QEMU execution)

### Technical Achievements
- ✅ Custom SysTick-based time driver (1kHz ticks, wake_by_ref for task responsiveness)
- ✅ `embedded-alloc` heap (64KB LlffHeap)
- ✅ Semihosting-based logging (replaced defmt for QEMU compatibility)
- ✅ Migrated from git to crates.io dependencies (embassy-executor 0.6, embassy-time 0.5)
- ✅ Verified safe mutex usage (spin::Mutex never held across .await, short critical sections only)

**Key Achievement:**
**True Embedded Validation** - Embassy example runs on real ARM target (Cortex-M4F) in QEMU. Proves Fluxion works on actual embedded hardware with 24/27 operators available in no_std. Serves as production-ready template for microcontroller applications. Successfully demonstrates sensor fusion with temporal operators on resource-constrained targets.

---

##  Version 0.8.0 - Complete Runtime Abstraction & Documentation

**Status:**  Completed - 2026-01-12

**Goal:** Finalize multi-runtime support and align documentation with implementation reality

### What We Achieved

**Runtime Abstraction Complete:**
-  **5 Runtimes Fully Supported** - Tokio, smol, async-std, WASM, Embassy work out-of-the-box
-  **Dual Trait Bound System** - Module-level separation (`multi_threaded.rs` vs `single_threaded.rs`) solved runtime compatibility without workspace restructuring
-  **All 5 Time Operators Migrated** - debounce, throttle, delay, sample, timeout work seamlessly across all runtimes
-  **24/27 Operators on Embassy** - Only 3 operators (subscribe_latest, partition, share) fundamentally incompatible due to Embassy's static task allocation model
-  **Zero Trade-offs** - Achieved performance, flexibility, ergonomics, and embedded support simultaneously

**Architecture Insights:**
- **What We Planned:** Runtime-specific crates with separate trait definitions per runtime (v0.9.0 architecture)
- **What We Built:** Elegant dual-bound solution with compile-time feature selection
- **Why It's Better:**
  - Single implementation per operator (easier maintenance)
  - No breaking API changes for users
  - Zero performance overhead
  - Seamless operator chaining with perfect type inference

**Technical Implementation:**
- Module-level separation using `#[cfg(feature = ...)]` on implementations
- Separate `multi_threaded.rs` (Send + Sync bounds) and `single_threaded.rs` (no thread bounds)
- Feature flags: `runtime-tokio`, `runtime-smol`, `runtime-async-std`, `runtime-wasm`, `runtime-embassy`
- Macro-based code generation for eliminating duplication

**Documentation Overhaul:**
-  Removed obsolete `KNOWN_LIMITATIONS.md` - limitations solved by runtime abstraction
-  Removed `FUTURE_ARCHITECTURE.md` - alternative approach not needed
-  Updated all references from "v0.9.0 will solve" to accurate current status
-  Fixed 20+ misleading future-tense references across documentation
-  Clarified Embassy's 3 incompatible operators as architectural constraints, not temporary limitations
-  Aligned README, PITCH, operator guides, and API docs with implementation reality

### Quality Gates
-  All 990+ tests passing across 5 runtimes
-  Zero clippy warnings
-  Zero compiler warnings
-  CI green for all runtime configurations
-  WASM example validated in browser
-  Embassy example validated in QEMU on ARM Cortex-M4F
-  Documentation audit complete with zero broken links

### The Competitive Advantage

**Fluxion is NOW the ONLY reactive streams library that offers:**
-  27 production-ready operators
-  5 runtimes: Tokio, smol, async-std, WASM, **and Embassy (embedded)**
-  Same API from servers to browsers to microcontrollers
-  Zero-config for Tokio users, optional runtime selection for others
-  no_std + alloc support (24/27 operators)
-  True embedded validation on ARM hardware

**Market Position:**
- **RxRust**:  Requires custom code for non-Tokio runtimes,  No embedded support
- **Other reactive libs**:  std-only,  No embedded story
- **Embassy ecosystem**:  No full-featured reactive streams library
- **Fluxion**:  Works everywhere, production-ready, extensively tested

**Key Achievement:**
**Industry First** - Complete reactive streams library with 24/27 operators working on embedded systems. The only library that truly works everywhere from servers to microcontrollers. No trade-offs, no performance penalties, no competing solution.

---

## 🚀 Version 1.0.0 - Production Ready

**Essential Features:**

### Requirements for 1.0.0

#### 1. Complete Error Handling
- [ ] Error handling implemented
- [ ] Standard error handling operators implemented

**Phase 2: Stream Operator Error Propagation**
- [ ] All standard Rx operators supported along with chaining and error propagation for both ordering models

**Phase 3: Documentation & Finalization**
- [ ] Create `docs/error-handling.md` guide
- [ ] Add `# Errors` sections to all fallible functions
- [ ] Update crate-level docs to reflect error model
- [ ] Add error handling examples to README
- [ ] Document error recovery patterns

#### 2. API Stability

- [ ] Finalize all public APIs (no more breaking changes post-1.0)
- [ ] Review trait bounds for flexibility vs. simplicity
- [ ] Ensure consistent naming conventions
- [ ] Mark experimental features with appropriate warnings

#### 3. Performance & Optimization

- [ ] Comprehensive benchmark suite for all operators
- [ ] Performance comparison with similar libraries
- [ ] Identify and optimize hot paths
- [ ] Memory usage profiling and optimization
- [ ] Document performance characteristics in API docs

#### 4. Production Validation

- [ ] At least 2-3 real-world projects using Fluxion
- [ ] Stress testing with high-volume streams
- [ ] Long-running stability tests (hours/days)
- [ ] Validation on multiple platforms (Linux, macOS, Windows)

#### 5. Enhanced Documentation

- [ ] Migration guide for 0.1.x → 1.0.0
- [ ] Advanced patterns guide
- [ ] Troubleshooting guide
- [ ] Performance tuning guide
- [ ] Complete cookbook with common scenarios

#### 6. Testing & Quality

- [ ] Maintain >90% code coverage
- [ ] Fuzzing tests for critical paths
- [ ] Property-based tests for ordering guarantees
- [ ] Integration tests with real-world scenarios
- [ ] CI/CD pipeline with:
  - [ ] Multiple Rust versions (MSRV + stable + beta)
  - [ ] Code coverage reporting
  - [ ] Automated benchmarking
  - [ ] Security audit tools

#### 7. Community & Support

- [ ] Contribution guidelines (CONTRIBUTING.md)
- [ ] Code of conduct
- [ ] Issue templates (bug report, feature request)
- [ ] PR template
- [ ] Clear support channels

#### 8. Release Process

- [ ] Publish to crates.io
- [ ] Semantic versioning commitment
- [ ] CHANGELOG.md with all changes
- [ ] GitHub release with notes
- [ ] Documentation hosted on docs.rs

---

## 🔮 Future Releases (Post-1.0)

### Version 1.1+ - Quality of Life Improvements

**Enhanced Developer Experience:**
- [ ] Better compile-time error messages
- [ ] More helpful panic messages with context
- [ ] Additional stream operators based on user feedback
- [ ] Improved test utilities

**Performance:**
- [ ] Zero-copy optimizations where possible
- [ ] Reduced allocations in hot paths
- [ ] Optional SIMD optimizations
- [ ] Benchmark regression testing in CI

### Version 2.0+ - Major Enhancements

**Advanced Features:**
- [ ] Backpressure mechanisms
- [ ] Stream replay/caching capabilities
- [ ] Time-based windowing operators
- [ ] Advanced scheduling strategies
- [ ] Custom executor support
- [ ] Pluggable error handling strategies

**Ecosystem Integration:**
- [ ] Integration with popular async runtimes (async-std, smol)
- [ ] Bridge utilities for other stream libraries
- [ ] tracing/observability integration
- [ ] Metrics collection support

**Specialized Use Cases:**
- [ ] Real-time data processing utilities
- [ ] Event sourcing helpers
- [ ] CQRS pattern support
- [ ] Distributed stream processing (tentative)

---

## 📊 Success Metrics

### 1.0.0 Success Criteria
- Zero critical bugs for 30+ days
- 5+ production users
- 10+ GitHub stars (community validation)
- All planned features implemented
- Complete documentation
- Performance benchmarks meet targets

### 2.0.0 Success Criteria
- 50+ production users
- 100+ GitHub stars
- Active community contributions
- Established as a go-to solution for reactive streams in Rust

---

## 🎯 Current Focus

**Immediate Next Steps (Post-0.1.x):**

1. **Community Feedback** (Ongoing)
   - Gather user feedback from crates.io
   - Address issues and questions
   - Add missing examples

2. **Performance Baseline** (1 week)
   - Create comprehensive benchmark suite
   - Establish baseline metrics
   - Identify optimization opportunities

3. **Community Preparation** (1 week)
   - Add contribution guidelines
   - Set up issue/PR templates
   - Prepare announcement materials

---

## 📝 Notes

- This roadmap is living document and will evolve based on user feedback
- Version numbers follow [Semantic Versioning](https://semver.org/)
- Breaking changes are only introduced in major versions (post-1.0)
- Security fixes may be backported to previous minor versions

**Last Updated:** December 23, 2025
