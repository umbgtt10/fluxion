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

## 🚀 Version 0.6.5 - smol Runtime Support

**Status:** Completed (Not yet published)

**Goal:** Enable time-based operators with smol runtime through Timer abstraction

### Essential Features
- ✅ **SmolTimer Implementation** - Zero-sized type implementing Timer trait using async-io
- ✅ **Feature Flag** - `time-smol` for smol runtime support
- ✅ **Test Suite** - 10 comprehensive tests (5 operators × 2 threading models)
- ✅ **CI Integration** - Automated testing with `.ci/smol_tests.ps1`
- ✅ **Documentation** - Usage examples and implementation details
- ✅ **Public API** - `SmolTimer` and `SmolTimestamped<T>` exports

### Documentation
- ✅ smol usage examples with SmolTimer
- ✅ Implementation notes and platform support details
- ✅ Updated runtime support lists across all READMEs

### Quality Gates
- ✅ All tests passing (10/10 smol tests)
- ✅ Zero compilation errors for smol feature
- ✅ Zero clippy warnings
- ✅ CI green

### Why smol?
smol provides a lightweight, actively-maintained alternative to tokio with full multi-threading support, unlike WASM. This validates the Timer trait abstraction works across diverse runtime architectures.

## 🚀 Version 0.6.4 - Support async-std Runtime ⚠️ **DEPRECATED**

**Status:** Completed (Unmaintained Runtime)

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

**Status:** Completed

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

## 🚀 Version 0.7.0 - Full Runtime Abstraction

**Status:** Planned

**Goal:** Enable Fluxion to run on multiple async runtimes and in WASM environments

**Essential Features:**
- [ ] Runtime abstraction layer - Abstract spawn, channels, and time primitives
- [ ] Feature flags for runtime selection (`tokio`, `async-std`, `wasm-bindgen`)
- [ ] WASM compatibility - Remove OS-specific dependencies
- [ ] `wasm-bindgen-futures` integration for browser environments
- [ ] Conditional compilation for different runtime backends
- [ ] Time/scheduling abstraction compatible with WASM constraints
- [ ] Implement `testing_time` module for simulating time in tests across runtimes

**Documentation:**
- [ ] Runtime selection guide (choosing between Tokio, async-std, etc.)
- [ ] WASM integration examples
- [ ] Browser-based reactive streams example
- [ ] Migration guide for runtime-specific code
- [ ] Performance characteristics across runtimes

**Quality Gates:**
- [ ] All time-related tests are parameterized over every possible runtime and pass
- [ ] WASM example compiles and runs in browser
- [ ] CI tests against multiple runtimes
- [ ] Zero clippy warnings across all feature combinations
- [ ] Zero compiler warnings
- [ ] Doc tests for all runtime configurations
- [ ] CI green


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

**Last Updated:** December 18, 2025
