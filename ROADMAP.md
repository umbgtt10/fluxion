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
- ✅ All 186 tests passing
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

**Goal:** Expand operator library and add advanced streaming capabilities

**Planned Features:**
- ✅ `scan` operator - Stateful accumulation across stream
- [ ] `debounce` operator - Time-based event throttling
- [ ] `throttle` operator - Rate limiting
- [ ] `buffer` operator - Windowing and batching
- ✅ `distinct_until_changed` operator - Duplicate suppression
- ✅ `distinct_until_changed_by` operator - Duplicate suppression with custom comparison
- [ ] Additional error handling operators

See [Operators Roadmap](docs/FLUXION_OPERATORS_ROADMAP.md) for detailed operator implementation timeline beyond v0.3.0.

## 🚀 Version 0.5.0 - Double-Time POC
**Essential Features:**
- [ ] POC: three operators implemented without ordering guarantees as per design doc
- [ ] All existing and new tests green
- [ ] New benches implemented for new operators

**Documentation:**
- [ ] POC: Dual/Expanded documentation implemented

**Quality Gates:**
- [ ] No blockers for the final migration to the dual ordering models

## 🚀 Version 0.6.0 - Double-Time
**Essential Features:**
- [ ] All operators migrated to dual ordering models as per design doc

**Documentation:**
- [ ] Dual/Expanded documentation implemented and consistent across all operators
- [ ] Bench comparison available and deployed with the CI

**Quality Gates:**
- [ ] Fully-fledged test suite up and running for both ordering models
- [ ] CI green

---

## 🚀 Version 0.6.0 - Cloning
**Essential Features:**
- [ ] Investigate the best way to clone or share streams between multiple consumers
- [ ] Investigate FluxionSubject

**Documentation:**

**Quality Gates:**

## 🚀 Version 0.7.0 - Wasm & Runtime abstraction
**Essential Features:**
- [ ] Implement runtime abstraction

**Documentation:**

**Quality Gates:**


## 🚀 Version 1.0.0 - Production Ready

**Essential Features:**

### Requirements for 1.0.0

#### 1. Complete Error Handling
- [ ] Error handling implemented
- [ ] Standard error handling operators implemented

**Phase 2: Stream Operator Error Propagation**
- [ ] All standard Rx operators supported along with chaining and error propagation for both ordering mmodels

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

**Breaking Changes (2.0):**
- [ ] Resolve task lifecycle management for `UnboundedReceiverExt`
  - Decision: Simple API (orphaned tasks) vs Explicit control (return JoinHandle)
  - See orphaned task discussion in development notes
- [ ] API refinements based on 1.x usage patterns
- [ ] Removal of deprecated APIs from 1.x

---

## 📊 Success Metrics

### 0.1.x Achievements ✅
- ✅ Library compiles and all tests pass
- ✅ Documentation covers all public APIs
- ✅ Published to crates.io

### 0.2.x Achievements
- Example project demonstrate usage

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

**Last Updated:** November 17, 2025
