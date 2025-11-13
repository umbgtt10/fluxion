# Fluxion Error Handling Improvement Plan

## Executive Summary

This document outlines a comprehensive strategy to address all error handling concerns in the Fluxion codebase:
1. Eliminate `.expect()` calls and panics in production code
2. Create custom error types for library-specific failures
3. Add proper error trait bounds to `subscribe_async` generic error type
4. Eliminate silent failures in stream operators

---

## Phase 1: Create Core Error Infrastructure

### 1.1 Create `fluxion-error` Crate

**Location:** `fluxion-error/`

**Purpose:** Centralized error handling for all Fluxion crates

**Implementation:**

```toml
# fluxion-error/Cargo.toml
[package]
name = "fluxion-error"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
thiserror = "2.0"
```

```rust
// fluxion-error/src/lib.rs
use std::fmt;

/// Root error type for all Fluxion operations
#[derive(Debug, thiserror::Error)]
pub enum FluxionError {
    /// Error acquiring a lock on shared state
    #[error("Failed to acquire lock: {context}")]
    LockError { context: String },

    /// Channel send error
    #[error("Channel send failed: receiver dropped")]
    ChannelSendError,

    /// Channel receive error
    #[error("Channel receive failed: {reason}")]
    ChannelReceiveError { reason: String },

    /// Stream processing error
    #[error("Stream processing error: {context}")]
    StreamProcessingError { context: String },

    /// User-provided callback panicked
    #[error("User callback panicked: {context}")]
    CallbackPanic { context: String },

    /// Subscription error
    #[error("Subscription error: {context}")]
    SubscriptionError { context: String },

    /// Invalid state error
    #[error("Invalid state: {message}")]
    InvalidState { message: String },

    /// Custom error for user-defined errors
    #[error("User error: {0}")]
    UserError(Box<dyn std::error::Error + Send + Sync>),
}

/// Specialized Result type for Fluxion operations
pub type Result<T> = std::result::Result<T, FluxionError>;

/// Trait for converting errors into FluxionError
pub trait IntoFluxionError {
    fn into_fluxion_error(self, context: &str) -> FluxionError;
}

impl<E: std::error::Error + Send + Sync + 'static> IntoFluxionError for E {
    fn into_fluxion_error(self, context: &str) -> FluxionError {
        FluxionError::UserError(Box::new(self))
    }
}
```

**Files to create:**
- `fluxion-error/Cargo.toml`
- `fluxion-error/src/lib.rs`

**Workspace changes:**
```toml
# Cargo.toml
[workspace]
members = [
    "fluxion-core",
    "fluxion-error",  # NEW
    "fluxion-ordered-merge",
    # ... rest
]

[workspace.dependencies]
fluxion-error = { path = "fluxion-error" }
thiserror = "2.0"
```

---

## Phase 2: Fix `fluxion-exec` Error Handling

### 2.1 Update `subscribe_async.rs`

**Current Issues:**
- Generic error type `E` lacks error trait bounds
- Panics when no error callback provided
- No way to propagate errors to caller

**Solution:**

```rust
// fluxion-exec/src/subscribe_async.rs
use async_trait::async_trait;
use fluxion_error::{FluxionError, Result};
use futures::stream::Stream;
use futures::stream::StreamExt;
use std::future::Future;
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait SubscribeAsyncExt<T>: Stream<Item = T> + Sized {
    /// Subscribe to stream with async processing
    ///
    /// Returns Ok(()) if all items processed successfully, or Err if:
    /// - User callback returns an error and no error handler is provided
    /// - Stream processing encounters an internal error
    async fn subscribe_async<F, Fut, E, OnError>(
        self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + Sync + 'static;  // ADDED ERROR BOUND
}

#[async_trait]
impl<S, T> SubscribeAsyncExt<T> for S
where
    S: Stream<Item = T> + Send + Unpin + 'static,
    T: Send + 'static,
{
    async fn subscribe_async<F, Fut, E, OnError>(
        mut self,
        on_next_func: F,
        cancellation_token: Option<CancellationToken>,
        on_error_callback: Option<OnError>,
    ) -> Result<()>
    where
        F: Fn(T, CancellationToken) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = std::result::Result<(), E>> + Send + 'static,
        OnError: Fn(E) + Clone + Send + Sync + 'static,
        T: std::fmt::Debug + Send + Clone + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let cancellation_token = cancellation_token.unwrap_or_default();
        let error_occurred = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        while let Some(item) = self.next().await {
            if cancellation_token.is_cancelled() {
                break;
            }

            let on_next_func = on_next_func.clone();
            let cancellation_token = cancellation_token.clone();
            let on_error_callback = on_error_callback.clone();
            let error_occurred = error_occurred.clone();
            let item_debug = format!("{:?}", item);

            tokio::spawn(async move {
                match on_next_func(item.clone(), cancellation_token).await {
                    Ok(_) => {},
                    Err(error) => {
                        if let Some(handler) = on_error_callback {
                            handler(error);
                        } else {
                            // Mark that an error occurred but don't panic
                            error_occurred.store(true, std::sync::atomic::Ordering::SeqCst);
                            tracing::error!(
                                "Unhandled error in subscribe_async for item {}: {}",
                                item_debug,
                                error
                            );
                        }
                    }
                }
            });
        }

        // Check if any errors occurred
        if error_occurred.load(std::sync::atomic::Ordering::SeqCst) {
            Err(FluxionError::SubscriptionError {
                context: "One or more items failed to process".to_string(),
            })
        } else {
            Ok(())
        }
    }
}
```

**Changes:**
- Add `std::error::Error` bound to generic `E`
- Return `Result<()>` instead of `()`
- Replace panic with error logging + error return
- Use atomic bool to track errors across spawned tasks
- Add optional `tracing` support for better debugging

### 2.2 Update `subscribe_latest_async.rs`

**Similar changes:**

```rust
// Key changes:
1. Add error trait bound: E: std::error::Error + Send + Sync + 'static
2. Return Result<()> instead of ()
3. Replace panic with error logging
4. Track errors with atomic bool
5. Return aggregated error status
```

**Files to modify:**
- `fluxion-exec/src/subscribe_async.rs`
- `fluxion-exec/src/subscribe_latest_async.rs`
- `fluxion-exec/Cargo.toml` (add dependencies)

---

## Phase 3: Fix `fluxion-stream` Lock/Unwrap Issues

### 3.1 Replace `.lock().unwrap()` with Error Handling

**Files affected:**
- `fluxion-stream/src/combine_latest.rs` (line 62)
- `fluxion-stream/src/take_latest_when.rs` (line 89)
- `fluxion-stream/src/take_while_with.rs` (line 77)

**Strategy:** Create helper function for lock acquisition

```rust
// fluxion-stream/src/util.rs (NEW FILE)
use fluxion_error::{FluxionError, Result};
use std::sync::{Arc, Mutex, MutexGuard};

/// Safe lock acquisition that returns FluxionError instead of panicking
pub(crate) fn acquire_lock<T>(
    mutex: &Arc<Mutex<T>>,
    context: &str,
) -> Result<MutexGuard<T>> {
    mutex.lock().map_err(|e| FluxionError::LockError {
        context: format!("{}: {}", context, e),
    })
}

/// Alternative: Use poisoning-resilient lock acquisition
pub(crate) fn acquire_lock_resilient<T>(
    mutex: &Arc<Mutex<T>>,
) -> Result<MutexGuard<T>> {
    match mutex.lock() {
        Ok(guard) => Ok(guard),
        Err(poisoned) => {
            // Log the poison but recover by accessing the inner data
            tracing::warn!("Mutex was poisoned, recovering");
            Ok(poisoned.into_inner())
        }
    }
}
```

**Apply to operators:**

```rust
// fluxion-stream/src/combine_latest.rs
// OLD:
let mut state = state
    .lock()
    .expect("Failed to acquire lock on combine_latest state");

// NEW:
let mut state = match state.lock() {
    Ok(guard) => guard,
    Err(poisoned) => {
        // Recover from poison - the data is still valid
        poisoned.into_inner()
    }
};
```

**Rationale:**
- Mutex poisoning is rare and usually indicates a panic in a lock holder
- The data is still valid - we can recover by using `into_inner()`
- This prevents cascading panics while maintaining safety

### 3.2 Handle Stream Termination Gracefully

**Issue:** Some operators may silently fail or drop items

**Solution:** Add explicit error signaling in stream combinators

```rust
// Example for combine_latest:
// Add error channel to propagate issues
enum CombineLatestItem<T> {
    Value(T),
    Error(FluxionError),
}

// Modify stream to emit errors when lock fails
// Consumers can decide how to handle
```

**Files to modify:**
- `fluxion-stream/src/combine_latest.rs`
- `fluxion-stream/src/take_latest_when.rs`
- `fluxion-stream/src/take_while_with.rs`
- `fluxion-stream/src/util.rs` (NEW)

---

## Phase 4: Fix `fluxion-test-utils` Error Handling

### 4.1 Update `FluxionChannel::push()`

**Current code:**
```rust
pub fn push(&self, value: T) {
    self.sender.send(value).expect("receiver dropped");
}
```

**New code:**
```rust
use fluxion_error::{FluxionError, Result};

pub fn push(&self, value: T) -> Result<()> {
    self.sender.send(value).map_err(|_| FluxionError::ChannelSendError)
}

/// Panicking version for tests that don't care about error handling
pub fn push_unchecked(&self, value: T) {
    self.sender.send(value).expect("receiver dropped");
}
```

**Rationale:**
- Main API returns `Result` for production use
- Keep `push_unchecked()` for existing tests to minimize changes
- Gradually migrate tests to use `push()` with proper error handling

### 4.2 Update Helper Functions

**Files:**
- `fluxion-test-utils/src/helpers.rs`

**Changes:**
```rust
// OLD:
pub async fn assert_next_eq<T, S>(stream: &mut S, expected: T)
where
    S: Stream<Item = T> + Unpin,
    T: PartialEq + Debug,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item, expected);
}

// NEW:
pub async fn assert_next_eq<T, S>(stream: &mut S, expected: T) -> Result<()>
where
    S: Stream<Item = T> + Unpin,
    T: PartialEq + Debug,
{
    let item = stream
        .next()
        .await
        .ok_or_else(|| FluxionError::StreamProcessingError {
            context: "Expected item but stream ended".to_string(),
        })?;

    if item == expected {
        Ok(())
    } else {
        Err(FluxionError::InvalidState {
            message: format!("Expected {:?}, got {:?}", expected, item),
        })
    }
}

// Keep unchecked versions for backward compatibility
pub async fn assert_next_eq_unchecked<T, S>(stream: &mut S, expected: T)
where
    S: Stream<Item = T> + Unpin,
    T: PartialEq + Debug,
{
    let item = stream.next().await.expect("expected next item");
    assert_eq!(item, expected);
}
```

**Files to modify:**
- `fluxion-test-utils/src/fluxion_channel.rs`
- `fluxion-test-utils/src/helpers.rs`

---

## Phase 5: Add Error Recovery Operators

### 5.1 Create Error Recovery Extensions

**New file:** `fluxion-stream/src/error_recovery.rs`

```rust
use futures::Stream;
use fluxion_error::FluxionError;

/// Extension trait for error recovery in streams
pub trait ErrorRecoveryExt: Stream + Sized {
    /// Retry a failed operation with exponential backoff
    fn retry_with_backoff(
        self,
        max_retries: usize,
        initial_delay: std::time::Duration,
    ) -> impl Stream<Item = Self::Item>;

    /// Catch errors and replace with default value
    fn catch_error_with<F>(
        self,
        handler: F,
    ) -> impl Stream<Item = Self::Item>
    where
        F: Fn(FluxionError) -> Option<Self::Item>;

    /// Log errors but continue processing
    fn log_errors(self) -> impl Stream<Item = Self::Item>;
}
```

---

## Phase 6: Update Documentation

### 6.1 Add Error Handling Guide

**New file:** `docs/error-handling.md`

```markdown
# Error Handling in Fluxion

## Overview
Fluxion uses a comprehensive error handling strategy...

## Error Types
- FluxionError: Root error type
- Result<T>: Type alias for Result<T, FluxionError>

## Best Practices
1. Always use Result return types for fallible operations
2. Provide error callbacks in subscribe operations
3. Use error recovery operators for resilient streams
4. Log errors with tracing for debugging

## Migration from Panic-based Code
...
```

### 6.2 Update API Documentation

Add `# Errors` sections to all public APIs that can fail.

---

## Implementation Timeline

### Phase 1: Core Infrastructure (2-3 days)
- [ ] Create `fluxion-error` crate
- [ ] Define error types
- [ ] Add to workspace
- [ ] Write unit tests for error types

### Phase 2: Fix fluxion-exec (2-3 days)
- [ ] Add error bounds to subscribe_async
- [ ] Replace panics with error returns
- [ ] Add error tracking
- [ ] Update tests
- [ ] Update documentation

### Phase 3: Fix fluxion-stream (3-4 days)
- [ ] Create lock helper utilities
- [ ] Replace all .lock().unwrap() calls
- [ ] Add error propagation to operators
- [ ] Update tests
- [ ] Verify no silent failures

### Phase 4: Fix fluxion-test-utils (1-2 days)
- [ ] Update push() to return Result
- [ ] Add push_unchecked() for tests
- [ ] Update helper functions
- [ ] Migrate critical tests

### Phase 5: Error Recovery (2-3 days)
- [ ] Implement retry operators
- [ ] Implement error logging
- [ ] Add recovery examples
- [ ] Write comprehensive tests

### Phase 6: Documentation (1-2 days)
- [ ] Write error handling guide
- [ ] Update API docs
- [ ] Create migration guide
- [ ] Add examples

### Phase 7: Testing & Integration (2-3 days)
- [ ] Run full test suite
- [ ] Fix breaking changes
- [ ] Verify error propagation
- [ ] Performance testing
- [ ] Update CHANGELOG

**Total Estimated Time: 13-20 days**

---

## Breaking Changes & Migration Strategy

### Breaking Changes
1. `subscribe_async` now returns `Result<()>`
2. Generic error type `E` now requires `std::error::Error` bound
3. `FluxionChannel::push()` returns `Result<()>`
4. Helper functions may return `Result<()>`

### Migration Strategy

#### Option A: Semantic Versioning (Recommended)
- Release as `0.2.0` with breaking changes
- Provide migration guide
- Keep `0.1.x` branch for critical fixes

#### Option B: Gradual Migration
- Add new APIs alongside old ones
- Deprecate old APIs with warnings
- Remove in next major version

#### Option C: Feature Flags
```toml
[features]
default = ["error-handling"]
error-handling = []
legacy-panics = []
```

**Recommendation:** Option A - clean break with good documentation

---

## Testing Strategy

### Unit Tests
- Test each error variant
- Test error conversion
- Test error propagation

### Integration Tests
- Test error handling across crate boundaries
- Test error recovery operators
- Test backward compatibility with `_unchecked` variants

### Stress Tests
- Test lock contention scenarios
- Test error handling under high load
- Test graceful degradation

---

## Monitoring & Observability

### Tracing Integration
```rust
// Add optional tracing
#[cfg(feature = "tracing")]
use tracing::{error, warn, debug};

#[cfg(not(feature = "tracing"))]
macro_rules! error { ($($arg:tt)*) => {} }
```

### Metrics
- Count of errors by type
- Error recovery success rate
- Lock contention metrics

---

## Success Criteria

### Must Have
- ✅ Zero panics in production code paths
- ✅ All errors are typed and recoverable
- ✅ Generic error bounds properly specified
- ✅ No silent failures in operators
- ✅ All tests passing
- ✅ Documentation updated

### Nice to Have
- ✅ Error recovery operators implemented
- ✅ Tracing integration
- ✅ Performance benchmarks showing minimal overhead
- ✅ Migration guide with examples

---

## Risk Assessment

### Low Risk
- Creating error types (no breaking changes to existing code)
- Adding error recovery operators (new functionality)

### Medium Risk
- Changing return types to Result<()>
- Updating lock acquisition patterns
- May require test updates

### High Risk
- Adding error trait bounds (breaks existing generic code)
- Changing FluxionChannel::push() signature
- Will break downstream users

### Mitigation
- Comprehensive testing
- Staged rollout
- Clear migration documentation
- Maintain backward compatibility variants

---

## Open Questions

1. **Should we use `anyhow` for error context chains?**
   - Pros: Better error context, easier debugging
   - Cons: Additional dependency
   - **Recommendation:** Use `thiserror` only, add context manually

2. **Should errors implement `Clone`?**
   - Pros: Easier to work with in async contexts
   - Cons: Not all errors are clonable
   - **Recommendation:** Don't require Clone, use Arc when needed

3. **Should we provide error hooks for monitoring?**
   - Pros: Better observability
   - Cons: More complex API
   - **Recommendation:** Add as optional feature

4. **How to handle poisoned mutexes?**
   - Current plan: Recover by using `into_inner()`
   - Alternative: Propagate as error
   - **Recommendation:** Recover when safe, propagate if corruption suspected

---

## Appendix A: Error Type Hierarchy

```
FluxionError (root)
├── LockError
├── ChannelSendError
├── ChannelReceiveError
├── StreamProcessingError
├── CallbackPanic
├── SubscriptionError
├── InvalidState
└── UserError(Box<dyn Error>)
```

---

## Appendix B: File Checklist

### New Files
- [ ] `fluxion-error/Cargo.toml`
- [ ] `fluxion-error/src/lib.rs`
- [ ] `fluxion-stream/src/util.rs`
- [ ] `fluxion-stream/src/error_recovery.rs`
- [ ] `docs/error-handling.md`

### Modified Files
- [ ] `Cargo.toml` (workspace)
- [ ] `fluxion-exec/Cargo.toml`
- [ ] `fluxion-exec/src/subscribe_async.rs`
- [ ] `fluxion-exec/src/subscribe_latest_async.rs`
- [ ] `fluxion-stream/Cargo.toml`
- [ ] `fluxion-stream/src/lib.rs`
- [ ] `fluxion-stream/src/combine_latest.rs`
- [ ] `fluxion-stream/src/take_latest_when.rs`
- [ ] `fluxion-stream/src/take_while_with.rs`
- [ ] `fluxion-test-utils/Cargo.toml`
- [ ] `fluxion-test-utils/src/fluxion_channel.rs`
- [ ] `fluxion-test-utils/src/helpers.rs`
- [ ] All test files (potentially)

---

## Appendix C: Dependencies to Add

```toml
# fluxion-error/Cargo.toml
thiserror = "2.0"

# fluxion-exec/Cargo.toml
fluxion-error = { workspace = true }
tracing = { version = "0.1", optional = true }

# fluxion-stream/Cargo.toml
fluxion-error = { workspace = true }
tracing = { version = "0.1", optional = true }

# fluxion-test-utils/Cargo.toml
fluxion-error = { workspace = true }
```
