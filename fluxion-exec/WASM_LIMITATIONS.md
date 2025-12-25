# WASM Limitations in fluxion-exec

## SubscribeExt Not Available on WASM Targets

### Status: Known Limitation (as of v0.6.13)

The `SubscribeExt` trait is currently **not functional on WASM targets** (`wasm32-unknown-unknown`), despite having platform-specific implementations.

### Root Cause

The limitation stems from an architectural dependency chain:

1. **SubscribeExt Error Collection**: When no error callback is provided, `SubscribeExt::subscribe()` collects errors and returns them via `FluxionError::from_user_errors()`

2. **FluxionError Send+Sync Requirement**: The `from_user_errors()` method requires:
   ```rust
   pub fn from_user_errors<E>(errors: Vec<E>) -> Self
   where
       E: core::error::Error + Send + Sync + 'static,
   ```

3. **WASM Single-Threaded Nature**: WASM runs in a single-threaded environment using `Rc<RefCell<T>>` patterns, which are **not Send+Sync**

4. **Type Mismatch**: The WASM `SubscribeExt` trait correctly removes Send+Sync from closures and futures:
   ```rust
   #[cfg(target_arch = "wasm32")]
   async fn subscribe<F, Fut, E, OnError>(...) -> Result<()>
   where
       E: Error + 'static,  // ❌ Only 'static, not Send+Sync
   ```

   But it still calls `FluxionError::from_user_errors(collected_errors)`, which **requires E: Send + Sync**.

5. **Compilation Failure**: This creates an impossible constraint - WASM errors cannot be Send+Sync, but `from_user_errors` requires them to be.

### Attempted Solutions

#### Approach 1: Remove Send+Sync from FluxionError ❌
Making `FluxionError::UserError` non-Send on WASM cascades through the entire codebase:
- `StreamItem<T>` becomes non-Send
- All stream operators returning `Stream<Item = StreamItem<T>> + Send` fail
- `FluxionSubject` boxed streams require Send+Sync
- 35+ compilation errors across fluxion-stream, fluxion-core

**Verdict**: Entire library architecture assumes Send+Sync for multi-runtime support.

#### Approach 2: Platform-Specific Error Types ❌
Creating separate `FluxionError` definitions for WASM vs non-WASM:
- Requires duplicating error handling logic
- Breaks type compatibility across features
- Makes generic code impossible (can't be generic over Send when it's conditionally present)

**Verdict**: Architecturally unsound, breaks library composability.

#### Approach 3: Add Send+Sync Back to WASM SubscribeExt ❌
Making WASM error type `E: Error + Send + Sync + 'static`:
- Compiles successfully
- But defeats the entire purpose - WASM errors (using Rc<RefCell<>>) **cannot** be Send+Sync
- Users cannot actually use the trait with WASM-compatible types

**Verdict**: Type signature compiles but is unusable in practice.

### Current Workaround

**Use `wasm_bindgen_futures::spawn_local` directly:**

```rust
use wasm_bindgen_futures::spawn_local;
use futures::StreamExt;

fn wire_stream(
    mut stream: SharedBoxStream<T>,
    ui: Rc<RefCell<UI>>,
    cancel_token: CancellationToken,
) {
    spawn_local(async move {
        while !cancel_token.is_cancelled() {
            match stream.next().await {
                Some(StreamItem::Value(value)) => {
                    ui.borrow_mut().update(value);
                }
                Some(StreamItem::Error(e)) => {
                    web_sys::console::error_1(&format!("Error: {:?}", e).into());
                }
                None => break,
            }
        }
    });
}
```

**Advantages:**
- ✅ Works perfectly with Rc<RefCell<>> patterns
- ✅ Explicit task spawning
- ✅ No Send+Sync constraints
- ✅ Clear control flow

**Disadvantages:**
- ❌ More boilerplate than `.subscribe()`
- ❌ Manual stream loop
- ❌ No automatic error collection

### Future Solutions

#### Option 1: Error-Free Callback Variant (Preferred)
Add a WASM-specific variant that doesn't collect errors:

```rust
#[cfg(target_arch = "wasm32")]
async fn subscribe_infallible<F, Fut>(
    self,
    on_next_func: F,
    cancellation_token: Option<CancellationToken>,
) -> ()  // Never fails, no error collection
where
    F: Fn(T, CancellationToken) -> Fut + Clone + 'static,
    Fut: Future<Output = ()> + 'static,  // No error type
```

#### Option 2: String-Based Error Collection
Collect errors as strings instead of typed errors:

```rust
#[cfg(target_arch = "wasm32")]
pub fn from_user_error_messages(errors: Vec<String>) -> Self {
    Self::MultipleErrors {
        count: errors.len(),
        errors: errors.into_iter()
            .map(|msg| Self::StreamProcessingError { context: msg })
            .collect(),
    }
}
```

#### Option 3: Architecture Redesign (Major Breaking Change)
Make error handling completely optional at the type level:
- `StreamItem<T>` → `StreamItem<T, E = FluxionError>`
- Platform-specific error trait bounds
- Would require Fluxion 2.0

### Impact Assessment

**Severity**: Medium
- Core functionality (stream processing) works on WASM
- Only affects convenience trait (SubscribeExt)
- Workaround is straightforward

**Affected Users**:
- WASM applications wanting to use SubscribeExt
- Projects requiring both WASM and non-WASM builds with shared code

**Timeline**:
- **v0.7.0**: Document limitation (this document)
- **v0.8.0**: Implement Option 1 or 2
- **v2.0**: Consider Option 3 if architectural redesign happens

### Related Issues

- Runtime abstraction for WASM: ✅ Complete (v0.6.8)
- FluxionTask WASM support: ✅ Complete (v0.6.8)
- Time operators on WASM: ✅ Complete (v0.6.3)
- SubscribeExt on WASM: ❌ Blocked by error type constraints

### References

- fluxion-exec/src/subscribe.rs - SubscribeExt trait implementation
- fluxion-core/src/fluxion_error.rs - FluxionError type definition
- examples/wasm-dashboard - Working WASM example using spawn_local

---

**Last Updated:** December 25, 2025
**Affects:** Fluxion v0.6.13 and earlier
