# WASM Support in fluxion-exec

## SubscribeExt Now Available on WASM Targets ✅

### Status: **RESOLVED** (as of v0.7.0)

The `SubscribeExt` and `SubscribeLatestExt` traits are now **fully functional on WASM targets** (`wasm32-unknown-unknown`).

### What Changed

The architectural limitation has been resolved by removing error collection from the subscription traits:

1. **Error Callback Now Required**: Both `subscribe()` and `subscribe_latest()` now require an error callback parameter instead of making it optional.

2. **No More Error Collection**: The traits no longer collect errors into `Vec<E>` and return `FluxionError::MultipleErrors`. Errors are immediately passed to the callback.

3. **Simplified FluxionError**: Removed `UserError` and `MultipleErrors` variants, eliminating all Send+Sync constraints on user error types.

### Breaking Changes

```rust
// Before (v0.6.x):
stream.subscribe(handler, None, None::<fn(E)>).await?;

// After (v0.7.0):
stream.subscribe(handler, None, ignore_errors).await?;
// or with error handling:
stream.subscribe(handler, None, |err| eprintln!("Error: {}", err)).await?;
```

### Benefits

- ✅ Works on WASM without any constraints
- ✅ Works on no-std environments
- ✅ No Send+Sync requirements on error types
- ✅ Immediate error feedback (no delayed aggregation)
- ✅ No unbounded memory growth from error collection
- ✅ Cleaner separation: FluxionError for library errors, user callbacks for user errors

### Helper Function

Use `fluxion_exec::ignore_errors` for cases where you want to explicitly discard errors:

```rust
use fluxion_exec::{SubscribeExt, ignore_errors};

stream.subscribe(
    |item, _token| async move {
        process(item).await
    },
    None,
    ignore_errors  // Explicitly ignore errors
).await?;
```

### Migration Guide

1. Replace `None::<fn(E)>` with `ignore_errors` for cases where you don't care about errors
2. Replace `None::<fn(E)>` with an error callback `|err| { /* handle */ }` where you do care
3. Remove any code checking for `FluxionError::MultipleErrors` or `FluxionError::UserError`

---

## Historical Context

Previously blocked by architectural dependency on `FluxionError::from_user_errors()` which required `E: Send + Sync`, incompatible with WASM's `Rc<RefCell<T>>` error types. This limitation was resolved by requiring error callbacks and removing error collection entirely, simplifying the architecture for all platforms.

---

**Last Updated:** December 26, 2025
**Status:** Resolved in v0.7.0
