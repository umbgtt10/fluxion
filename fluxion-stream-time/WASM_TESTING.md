# WASM Testing Guide

## Prerequisites

Install wasm-pack:
```powershell
cargo install wasm-pack
```

## Running WASM Tests

**Important**: Run from the `fluxion-stream-time` directory, not workspace root:
```powershell
cd fluxion-stream-time
```

### Browser Tests
```powershell
wasm-pack test --headless --chrome --features time-wasm
```

Or with Firefox:
```powershell
wasm-pack test --headless --firefox --features time-wasm
```

### Node.js Tests (recommended for CI)
```powershell
wasm-pack test --node --features time-wasm
```

### Alternative: From workspace root with manifest path
```powershell
wasm-pack test --node --features time-wasm --manifest-path fluxion-stream-time/Cargo.toml
```

## Test Configuration

WASM tests are configured with:
- **Feature flag**: `time-wasm` (enables gloo-timers dependency)
- **Test framework**: `wasm-bindgen-test`
- **Target**: `wasm32-unknown-unknown`

## Key Differences from Tokio Tests

1. **No time control**: WASM tests cannot use `tokio::time::pause()` or `advance()`
   - Must use real delays with `gloo_timers::future::sleep()`
   - Tests will be slower and less deterministic

2. **Single-threaded only**: WASM runtime is single-threaded
   - No `multi_threaded` test folder for WASM
   - No `tokio::spawn` tests

3. **Browser environment**: Tests run in actual browser (headless)
   - Uses `wasm_bindgen_test_configure!(run_in_browser);`
   - Has access to browser APIs

## Test Structure

```
tests/
└── wasm/
    └── single_threaded/
        ├── debounce/
        ├── delay/
        ├── sample/
        ├── throttle/
        └── timeout/
```

## Example Test

```rust
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_debounce_basic() {
    let timer = WasmTimer::new();
    // ... test code using real delays
    gloo_timers::future::sleep(Duration::from_millis(150)).await;
    // ... assertions
}
```

## Conditional Compilation

Tests are only compiled when:
- `feature = "time-wasm"` is enabled
- `target_arch = "wasm32"` is detected

This ensures WASM tests don't interfere with regular Tokio tests.
