# Fluxion Roadmap

This document outlines the release plan for Fluxion, a reactive stream processing library with ordered semantics.

---

## 📦 Version 0.1.0 - Initial Release (Ready for Review)

**Status:** Feature-complete, pending final review

**Goal:** Provide a stable, working foundation for reactive stream processing with ordering guarantees.

### Core Requirements ✅

**Essential Features:**
- ✅ Core stream operators (`combine_latest`, `with_latest_from`, `ordered_merge`, etc.)
- ✅ Execution utilities (`subscribe_async`, `subscribe_latest_async`)
- ✅ Temporal ordering with `Sequenced<T>` wrapper
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

### What's NOT in 0.1.0

- Stream operators don't yet propagate errors (they log and drop)
- No published crates.io release
- Limited benchmark coverage (6 operators benchmarked)
- Limited real-world usage validation

### Pre-Release Checklist

- [x] Final documentation review for consistency
- [x] Review and cleanup TODOs in README
- [x] Verify all examples compile and run
- [ ] Add CHANGELOG.md with 0.1.0 entry
- [ ] Tag release: `git tag v0.1.0`

---

## 🚀 Version 1.0.0 - Production Ready

**Status:** Planned

**Goal:** Battle-tested, production-grade library with complete error handling and proven stability.

### Requirements for 1.0.0

#### 1. Complete Error Handling (Phase 2 & 3)

**Phase 2: Stream Operator Error Propagation**
- [ ] Change stream operators to return `Stream<Item = Result<T, FluxionError>>`
- [ ] `combine_latest` propagates lock errors instead of dropping items
- [ ] `with_latest_from` propagates errors
- [ ] `ordered_merge` propagates errors
- [ ] All operators handle and propagate internal failures
- [ ] Update all operator tests to handle `Result` streams
- [ ] Add specific error-triggering tests (lock poisoning, etc.)

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

### 0.1.0 Success Criteria
- Library compiles and all tests pass
- Documentation covers all public APIs
- At least one example project successfully uses the library

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

**Immediate Next Steps (Post-0.1.0):**

1. **Review & Polish** (1-2 weeks)
   - Clean up documentation duplication
   - Resolve TODO items in README
   - Add missing examples

2. **Phase 2 Error Handling** (2-3 weeks)
   - Implement stream operator error propagation
   - Update all tests
   - Validate with real scenarios

3. **Performance Baseline** (1 week)
   - Create comprehensive benchmark suite
   - Establish baseline metrics
   - Identify optimization opportunities

4. **Community Preparation** (1 week)
   - Add contribution guidelines
   - Set up issue/PR templates
   - Prepare announcement materials

---

## 📝 Notes

- This roadmap is living document and will evolve based on user feedback
- Version numbers follow [Semantic Versioning](https://semver.org/)
- Breaking changes are only introduced in major versions (post-1.0)
- Security fixes may be backported to previous minor versions

**Last Updated:** November 15, 2025
