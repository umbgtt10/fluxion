# Runtime Abstraction & no_std Status

**Last Updated:** December 22, 2025

## Executive Summary

✅ **Runtime abstraction COMPLETE** - 5 runtimes supported (Tokio, smol, async-std, WASM, Embassy)
✅ **no_std Phase 1 COMPLETE** - 24/27 operators work in embedded environments
✅ **no_std Phase 3 COMPLETE** - All time operators work with Embassy runtime

**Current State:**
- ✅ 24/27 operators work in no_std + alloc environments (89%)
- ✅ All 5 time operators work on embedded targets with Embassy
- ✅ 1,790+ tests passing across all runtimes and configurations
- ✅ Zero breaking API changes
- ✅ CI-protected no_std compilation

**Remaining Work:**
- 📋 Phase 2: Poll-based partition() implementation (3 days effort)
- 📋 Optional: publish() operator for lazy multi-subscriber pattern (3 days effort)

---

## 📋 Remaining Tasks

### Phase 2: Poll-Based partition() [3 days effort]

**Goal:** Make partition() work on no_std via poll-based state machine

**Current Status:**
- ❌ partition() requires std (spawn-based concurrent implementation)
- ✅ 24/27 other operators work on no_std

**Implementation Plan:**

```rust
// Same API, different implementations
#[cfg(feature = "std")]
pub fn partition<F>(self, predicate: F) -> PartitionedStream<T, S, F> {
    // Current spawn-based implementation (concurrent branch progress)
}

#[cfg(not(feature = "std"))]
pub fn partition<F>(self, predicate: F) -> PartitionedStream<T, S, F> {
    // New poll-based state machine (sequential branch progress)
}
```

**Trade-offs:**
- ✅ Same API and semantics across std/no_std
- ⚠️ Sequential branch progress in no_std (vs concurrent in std)
- ✅ Clearly documented performance characteristics

**Tasks:**
1. Implement poll-based state machine (2 days)
2. Test on embedded target (0.5 days)
3. Document performance differences (0.5 days)

**Success Criteria:**
- ✅ partition() compiles and runs on no_std
- ✅ Same function signature and semantics
- ✅ Tests pass on thumbv7em-none-eabihf target
- ✅ Performance characteristics documented

---

### Phase 4 (Optional): publish() Operator [3 days effort]

**Goal:** Lazy multi-subscriber pattern without background task

**Current Status:**
- ❌ share() requires std (spawn-based hot broadcast)
- ✅ FluxionSubject provides hot pattern (works on no_std)

**Problem:**
- share() needs background task → requires std
- Users want multi-subscriber without spawn overhead

**Proposed Solution:**

```rust
/// Lazy shared execution - no spawn needed
/// Multiple subscribers poll the same source
/// First subscriber to poll triggers upstream fetch
/// All subscribers receive the same cached value
pub fn publish(self) -> Published<T, S>
```

**Semantics Comparison:**
- `share()` - Hot broadcast (background task, immediate propagation)
- `publish()` - Lazy multicast (pull-based, shared polling)

**Tasks:**
1. Design Published stream state machine (1 day)
2. Implement lazy polling coordinator (1.5 days)
3. Tests and documentation (0.5 days)

**Success Criteria:**
- ✅ Multiple subscribers without spawn
- ✅ Works on all runtimes including no_std
- ✅ Clear documentation of lazy vs hot semantics

---

## 🎯 Alternative Solutions

### subscribe_latest() - Already Solved ✅

**Use Case:** "I'm slow, skip intermediate values"

**Solution:** Use existing time operators:
- ✅ `throttle(duration)` - rate-limit processing
- ✅ `sample(duration)` - sample at intervals
- ✅ `debounce(duration)` - skip rapid-fire updates

**Example:**
```rust
// Instead of subscribe_latest (std-only)
stream.throttle(Duration::from_millis(100))
      .subscribe(slow_processor)  // Works on all runtimes
```

**Decision:** No new operator needed ✅

### share() - Use FluxionSubject ✅

**Current Solution:**
- FluxionSubject already provides hot multi-subscriber pattern
- Works on all runtimes including no_std
- Zero additional code needed

**Optional Enhancement:**
- publish() operator for lazy multi-subscriber (Phase 4 above)

### 4. Workspace Feature Management

**Current Pattern:**
```toml
# Workspace root
[workspace.dependencies]
fluxion-core = { ..., default-features = false }

# Every dependent crate must add:
fluxion-core = { workspace = true, features = ["std"] }
```

**Impact:**
- 13 files modified for no_std support
- 7/9 workspace crates need explicit std feature
- Manual process prone to errors

**Assessment:**
- ✅ This is standard Rust ecosystem pattern
- ✅ CI tests protect against mistakes
- ❌ No better alternative exists in current Rust

**Decision:**
- Accept as cost of no_std support
- Document pattern in CONTRIBUTING.md
- Maintain CI protection

**Effort:** 0.5 days (documentation only)

**Priority:** Low - this is idiomatic Rust, not a problem to fix

---

## 📊 Effort Summary

### Critical Path (Required)

| Phase | Duration | Status | Deliverable |
|-------|----------|--------|-------------|
| **Phase 2** | 3 days | 📋 Pending | Poll-based partition() (25/27 operators) |

### Optional Enhancements

| Task | Duration | Priority | Benefit |
|------|----------|----------|---------|
| **publish() operator** | 3 days | Optional | Lazy multi-subscriber without spawn |
| **Documentation** | 0.5 days | Low | CONTRIBUTING.md updates |

**Total Optional:** 3.5 days

---

## 🎯 Decision Framework

### When to Implement Phase 2 (partition)?

**Implement if:**
- ✅ Users need partition() on embedded targets
- ✅ Want to claim "all operators work on no_std"
- ✅ Willing to accept sequential vs concurrent trade-off

**Skip if:**
- ❌ No user demand for no_std partition()
- ❌ 24/27 operators sufficient for use cases
- ❌ Prefer to wait for user feedback

**Recommendation:** Wait for user demand - 24/27 is already excellent coverage

---

### When to Implement Phase 4 (publish)?

**Implement if:**
- ✅ Users need multi-subscriber without spawn overhead
- ✅ FluxionSubject API too low-level for common use case
- ✅ Want composable lazy multi-subscriber operator

**Skip if:**
- ❌ FluxionSubject sufficient for current users
- ❌ share() adequate for std environments
- ❌ No clear use case for lazy multicast

**Recommendation:** Wait for user demand - alternatives exist


## ✅ Success Criteria

**Phase 2 (partition):**
- ✅ Compiles with `--no-default-features --features alloc`
- ✅ Same API signature as std version
- ✅ Tests pass on embedded target
- ✅ Performance characteristics documented
- ✅ 25/27 operators work on no_std (93%)

**Phase 4 (publish):**
- ✅ Works on all 5 runtimes including no_std
- ✅ No spawn or background task required
- ✅ Clear documentation vs share() semantics
- ✅ Tests demonstrate lazy multi-subscriber pattern

**Technical Debt:**
- ✅ FluxionSubject uses parking_lot in std
- ✅ CONTRIBUTING.md documents workspace pattern
- ✅ All tests passing after changes

---

## 📝 Next Steps

### Immediate (Ready to Start)

2. **Documentation update** [0.5 days]
   - Add workspace feature pattern to CONTRIBUTING.md
   - Document no_std operator compatibility
   - Add embedded usage examples

### On-Demand (Wait for User Feedback)

3. **Phase 2: partition() implementation** [3 days]
   - Wait for user request for no_std partition
   - Current 24/27 coverage likely sufficient
   - Can be added without breaking changes

4. **Phase 4: publish() operator** [3 days]
   - Wait for user request for lazy multicast
   - FluxionSubject provides alternative
   - Can be added without breaking changes

### Future (Major Version)

5. **FluxionError refactor** [1-2 days]
   - Breaking change - defer to 0.7.0 or 1.0.0
   - Split enum by feature for cleaner API
   - Low priority - current implementation works
