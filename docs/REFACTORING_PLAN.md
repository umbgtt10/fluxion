# Fluxion Error Handling Implementation Plan

> **Note**: For the overall project roadmap and release planning, see [ROADMAP.md](../ROADMAP.md) at the repository root.

This document provides technical implementation details for the error handling refactor currently in progress.

## Status: Phase 1 Complete ✅

**Completed:** November 15, 2025

### Phase 1: Stabilize `fluxion-exec` API

**✅ Task 1.1: Modify Signatures to Return `Result<()>`**
- Changed `subscribe_async` return type from `()` to `fluxion_error::Result<()>`
- Changed `subscribe_latest_async` return type from `()` to `fluxion_error::Result<()>`
- Updated trait bounds to require `E: Send + Sync + 'static`

**✅ Task 1.2: Implement Error Aggregation**
- Added `Arc<Mutex<Vec<E>>>` to collect errors from spawned tasks
- When no `on_error_callback` is provided, errors are stored in collection
- Returns `FluxionError::MultipleErrors` if errors exist, else `Ok(())`
- Added `FluxionError::from_user_errors()` helper method

**✅ Task 1.3: Update Tests**
- Updated all 8 tests in `subscribe_async_tests.rs` to handle `Result`
- Updated all 9 tests in `subscribe_latest_async_tests.rs` to handle `Result`
- Added `test_subscribe_async_error_aggregation_without_callback`
- Added `test_subscribe_latest_async_error_aggregation_without_callback`
- All 19 tests passing

---

## Phase 2: Enable Error Propagation in Stream Operators (Planned)

**Objective:** Transform stream operators from silently dropping items on internal errors to propagating them through the stream.

### Task 2.1: Change Stream Item Types

Identify all operators in `fluxion-stream` that can fail and change their output to propagate errors:

**Affected Operators:**
- `combine_latest` - Uses `safe_lock`, can fail on lock poisoning
- `with_latest_from` - Uses `safe_lock`
- `ordered_merge` - May have internal failures
- `take_latest_when` - Uses locks internally
- Other operators using shared state

**Changes Required:**
```rust
// Before
impl Stream<Item = T>

// After  
impl Stream<Item = Result<T, FluxionError>>
```

### Task 2.2: Propagate Errors via Yield

Replace silent failures with error propagation:

**Current Pattern (Drop on Error):**
```rust
let guard = match safe_lock(&state, "combine_latest") {
    Ok(g) => g,
    Err(_) => {
        error!("Lock failed, dropping item");
        return None; // Silent failure!
    }
};
```

**New Pattern (Propagate Error):**
```rust
let guard = match safe_lock(&state, "combine_latest") {
    Ok(g) => g,
    Err(e) => {
        yield Err(FluxionError::LockError { 
            context: format!("combine_latest: {}", e)
        });
        return;
    }
};
```

### Task 2.3: Update Stream Tests

**Test Updates Required:**
- All tests must handle `Stream<Item = Result<T, FluxionError>>`
- Use `.map(|r| r.unwrap())` for tests expecting success
- Add specific error tests:
  - Test lock poisoning scenarios
  - Test error propagation through operator chains
  - Test error recovery patterns

**Example Test Pattern:**
```rust
#[tokio::test]
async fn test_combine_latest_propagates_lock_error() {
    // Setup: Poison the lock somehow
    // Act: Trigger the operator
    // Assert: Verify Err(FluxionError::LockError) is yielded
}
```

### Files to Modify

**fluxion-stream/src/**
- `combine_latest.rs`
- `with_latest_from.rs`
- `ordered_merge.rs`
- `take_latest_when.rs`
- `emit_when.rs`
- `take_while_with.rs`

**fluxion-stream/tests/**
- `combine_latest_tests.rs`
- `with_latest_from_tests.rs`
- `ordered_merge_tests.rs`
- `take_latest_when_tests.rs`
- `emit_when_tests.rs`
- `take_while_with_tests.rs`

### Breaking Changes

This is a **major breaking change**:
- All stream consumers must handle `Result` items
- Requires version bump (0.2.0 or 1.0.0)
- Migration guide needed

---

## Phase 3: Documentation and Finalization (Planned)

### Task 3.1: Create Error Handling Guide

**File:** `docs/error-handling.md`

**Contents:**
1. **Philosophy**: Why we propagate rather than hide errors
2. **Error Types**: Complete `FluxionError` reference
3. **Handling Patterns**:
   - Unwrapping when errors are impossible
   - Propagating with `?` operator
   - Recovery strategies
   - Error callbacks vs. Result returns
4. **Examples**:
   - Basic error handling
   - Error recovery in operators
   - Testing error paths

### Task 3.2: Update API Documentation

**For Every Fallible Function:**
- Add comprehensive `# Errors` section
- Document all error conditions
- Provide examples of error handling
- Link to error handling guide

**Crate-Level Updates:**
- Update `fluxion-exec/src/lib.rs`
- Update `fluxion-stream/src/lib.rs`
- Add error handling examples to main docs

### Task 3.3: Full Workspace Review

**Quality Gates:**
```bash
# All tests must pass
cargo test --workspace --all-features

# Zero clippy warnings
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Documentation builds without warnings
cargo doc --workspace --no-deps

# Formatting check
cargo fmt --all -- --check
```

**Manual Review Checklist:**
- [ ] All public APIs documented
- [ ] Error sections complete and accurate
- [ ] Examples compile and run
- [ ] Breaking changes documented
- [ ] Migration guide created
- [ ] CHANGELOG.md updated

---

## Implementation Notes

### Error Aggregation Pattern

```rust
// In subscribe functions
let errors: Arc<Mutex<Vec<E>>> = Arc::new(Mutex::new(Vec::new()));

// In spawned task
if let Err(error) = result {
    if let Some(callback) = on_error_callback {
        callback(error);
    } else {
        if let Ok(mut errs) = errors.lock() {
            errs.push(error);
        }
    }
}

// At function end
let collected_errors = {
    let mut guard = errors.lock().unwrap_or_else(|e| e.into_inner());
    std::mem::take(&mut *guard)
};

if !collected_errors.is_empty() {
    Err(FluxionError::from_user_errors(collected_errors))
} else {
    Ok(())
}
```

### Testing Error Paths

```rust
#[tokio::test]
async fn test_error_without_callback() {
    let result = stream
        .subscribe_async(
            |_item, _| async { Err(MyError::new("fail")) },
            None,
            None  // No error callback
        )
        .await;
    
    assert!(result.is_err());
    match result.unwrap_err() {
        FluxionError::MultipleErrors { count, .. } => {
            assert_eq!(count, expected_count);
        }
        _ => panic!("Expected MultipleErrors"),
    }
}
```

---

## Timeline

- **Phase 1**: ✅ Complete (Nov 15, 2025)
- **Phase 2**: 🚧 Planned (2-3 weeks)
- **Phase 3**: 📝 Planned (1 week after Phase 2)

See [ROADMAP.md](../ROADMAP.md) for overall project timeline.

---

**Last Updated:** November 15, 2025
