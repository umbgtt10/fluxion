# Fluxion AI Coding Instructions

## Project Overview

Fluxion is a **Rust reactive streams library** with temporal ordering guarantees. It's a multi-runtime (Tokio/smol/async-std/WASM/Embassy), workspace-based project emphasizing correctness, zero unsafe code, and exceptional test coverage (990+ tests, 10.8:1 test-to-code ratio).

**Key Principle**: All stream operators preserve **temporal ordering** via timestamps - items are processed in timestamp order, not arrival order.

## Architecture & Workspace Structure

This is a **7-crate Cargo workspace** with clear separation of concerns:

```
fluxion-rx            → Convenience facade (re-exports all)
fluxion-core          → Core traits: Timestamped, HasTimestamp, StreamItem, FluxionError
fluxion-stream        → 27 stream operators (combine_latest, ordered_merge, etc.)
fluxion-stream-time   → Time-based operators (debounce, throttle, delay, sample, timeout)
fluxion-exec          → Execution: subscribe, subscribe_latest
fluxion-ordered-merge → Generic ordered merge algorithm
fluxion-test-utils    → Sequenced<T> wrapper, test fixtures, helpers
```

**Critical files**:
- [fluxion-core/src/timestamped.rs](../fluxion-core/src/timestamped.rs) - Core trait definitions
- [fluxion-stream/src/lib.rs](../fluxion-stream/src/lib.rs) - Operator extension traits
- [docs/FLUXION_OPERATOR_SUMMARY.md](../docs/FLUXION_OPERATOR_SUMMARY.md) - All 27 operators explained

## Core Traits & Patterns

### HasTimestamp vs Timestamped

**Two separate traits** for ordering (read [fluxion-core/src/has_timestamp.rs](../fluxion-core/src/has_timestamp.rs) and [fluxion-core/src/timestamped.rs](../fluxion-core/src/timestamped.rs)):

- `HasTimestamp` - Minimal read-only timestamp access (use for ordering)
- `Timestamped` - Extends HasTimestamp, adds `type Inner` + construction methods (use for wrapper types)

```rust
// Ordering-only (most common)
impl<T> HasTimestamp for MyType<T> {
    type Timestamp = u64;
    fn timestamp(&self) -> u64 { self.ts }
}

// Wrapper types with inner value
impl<T: Clone> Timestamped for MyWrapper<T> {
    type Inner = T;
    fn with_timestamp(value: T, ts: u64) -> Self { /* ... */ }
    fn into_inner(self) -> T { self.value }
}
```

### StreamItem<T> - Not Result<T, E>

**Never use `Result` in stream operators**. Use `StreamItem<T>`:

```rust
pub enum StreamItem<T> {
    Value(T),
    Error(FluxionError),
}
```

Errors **flow through streams** like data (no short-circuiting). Handle with `.on_error()` operator.

## Testing Standards (Non-Negotiable)

### Test File Organization

- **Integration tests only** in `tests/` directory (avoid `#[cfg(test)]` modules)
- One test file per operator: `tests/operator_name_tests.rs`
- Use `#[tokio::test]` for all async tests

### Test Patterns

Always use `Sequenced<T>` from `fluxion-test-utils` for deterministic timestamps:

```rust
use fluxion_test_utils::{Sequenced, unwrap_stream, test_channel};

#[tokio::test]
async fn test_operator_behavior() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<i32>>();
    let mut stream = rx.operator();

    // Act
    tx.try_send(Sequenced::with_timestamp(10, 1))?;
    tx.try_send(Sequenced::with_timestamp(20, 2))?;

    // Assert
    assert_eq!(unwrap_stream(&mut stream, 500).await.unwrap().value, expected);
}
```

**Test structure rules**:
- Structure tests with `// Arrange`, `// Act`, `// Assert` comments
- **No error messages in assertions** - assertions are self-documenting
- **No other comments** - test name and structure should be clear enough
- Use descriptive test function names: `test_broadcasts_to_multiple_subscribers`
- **Test file naming**: All test files must end with `_tests` (e.g., `stream_item_tests.rs`)
- **Multiple Act/Assert pairs allowed**: A test can have multiple `// Act` / `// Assert` pairs, but only one `// Arrange` section

**Never use `tokio::time::sleep()` in tests** - use `Sequenced<T>` for determinism.

## Development Workflows

### Building & Testing

```powershell
# Full CI check (recommended before commits)
.\.ci\ci.ps1

# Fast iteration
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all

# Runtime-specific tests
.\.ci\tokio_tests.ps1    # Tokio with nextest
.\.ci\wasm_tests.ps1     # WASM (requires Node.js)
.\.ci\smol_tests.ps1     # smol runtime
.\.ci\embassy_tests.ps1  # Embassy timers
```

**CI script locations**: `.ci/` directory contains all automation (see [.ci/ci.ps1](../.ci/ci.ps1))

### Runtime Selection & Features

**Default**: Tokio (zero config)
**Alternatives**: `runtime-smol`, `runtime-async-std`, `runtime-wasm`, `runtime-embassy`

```toml
# Switch runtimes
fluxion-rx = { version = "0.8.0", default-features = false, features = ["runtime-smol"] }
```

**Embassy notes**: Requires `no_std` + `alloc`, manual timer trait implementation. Only 24/27 operators work (subscribe_latest, partition, share incompatible due to static task allocation).

## Code Style Requirements

### Documentation

**Every public item must have docs** (no exceptions):

```rust
/// Brief one-line summary.
///
/// Detailed explanation with context and behavior guarantees.
///
/// # Arguments
/// * `param` - Description
///
/// # Examples
/// ```rust
/// // Runnable example
/// use fluxion_stream::prelude::*;
/// ```
///
/// # Errors / # Panics (if applicable)
```

### File Structure Pattern

**Every productive code file** follows this strict two-section comment structure:

1. **Header comment** (identical for all files):
   ```rust
   // Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
   // Licensed under the Apache License, Version 2.0
   // http://www.apache.org/licenses/LICENSE-2.0
   ```

2. **Imports** (use statements)

3. **Commented examples** (one or more `///` doc comments with code examples) - placed **between imports and first type definition**

4. **Code** (trait/struct/enum definitions and implementations)

**Example pattern** (see [fluxion-core/src/timestamped.rs](../fluxion-core/src/timestamped.rs)):
```rust
// Copyright header...

use crate::HasTimestamp;

/// Trait documentation with description...
///
/// # Examples
///
/// ```rust
/// use fluxion_core::{Timestamped, HasTimestamp};
/// // ... example code ...
/// ```
pub trait Timestamped: HasTimestamp { /* ... */ }
```

**Do not add** inline `// comment` explanations in the code body - examples in doc comments are sufficient.

### Safety & Quality Standards

- ❌ **Zero `unsafe` code** (project policy)
- ❌ **Zero `.unwrap()` calls** - use `?` operator or explicit error handling
- ✅ **All clippy warnings resolved** (`-D warnings` in CI)
- ✅ **Comprehensive error propagation** via `StreamItem<T>` and `FluxionError`

### Operator Implementation Pattern

Operators live in `fluxion-stream/src/{operator_name}/` with:
- `mod.rs` - Public docs + re-exports
- `implementation.rs` - Extension trait + stream struct
- `tests/` in workspace root

See [fluxion-stream/src/map_ordered/](../fluxion-stream/src/map_ordered/) as canonical example.

## Common Patterns

### Enum-Based Multi-Type Streams

Use enums to combine different types in single stream (common in tests):

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Value {
    Int(i32),
    Str(String),
}

// Now can send both types through one channel
tx.send(Sequenced::new(Value::Int(42)))?;
tx.send(Sequenced::new(Value::Str("hello".into())))?;
```

### Temporal Ordering Guarantee

All operators use `ordered_merge` internally to buffer and reorder items by timestamp. Late items with earlier timestamps are correctly sequenced before newer items.

## Error Messages & Debugging

- Check [docs/ERROR-HANDLING.md](../docs/ERROR-HANDLING.md) for error patterns
- Use `.on_error()` operator for selective error handling
- `FluxionError` variants: `LockError`, `StreamProcessingError`, `UserError`, `MultipleErrors`

## Documentation References

- [README.md](../README.md) - Quick start + all 27 operators
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Detailed contribution guidelines
- [docs/FLUXION_OPERATOR_SUMMARY.md](../docs/FLUXION_OPERATOR_SUMMARY.md) - Operator reference
- [fluxion-stream/README.md](../fluxion-stream/README.md) - Stream operators deep dive
- [fluxion-test-utils/README.md](../fluxion-test-utils/README.md) - Testing utilities

## Version & Dependencies

**Current version**: 0.8.0 (workspace-wide in [Cargo.toml](../Cargo.toml))
All crates use workspace inheritance: `version.workspace = true`

**Dependency management**: Use `.ci/build.ps1` for `cargo upgrade` with workspace-wide compatible version updates.
