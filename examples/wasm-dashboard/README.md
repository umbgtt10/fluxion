# Fluxion WASM Dashboard Example

A WebAssembly-based real-time streaming dashboard demonstrating Fluxion's reactive stream operators in the browser.

## Overview

This example showcases:
- Real-time sensor data generation and visualization
- Stream combination with `combine_latest`
- Time-based operators (throttle, debounce, delay, sample, timeout)
- Canvas-based chart rendering
- WASM integration with web APIs

## Running the Application

### Prerequisites

- Rust (stable)
- wasm-pack: `cargo install wasm-pack`
- Node.js (for http-server)

### Build and Run

1. **Build the WASM module:**
   ```bash
   cd examples/wasm-dashboard
   wasm-pack build --target web --out-dir www/pkg
   ```

2. **Start the development server:**
   ```bash
   cd www
   npx http-server -p 8080 -c-1
   ```

3. **Open in browser:**
   ```
   http://127.0.0.1:8080/
   ```

## ⚠️ Current Architecture Issues

### Known Issues Requiring Refactoring

#### 1. **Sensor Data Timestamping**
**Problem:** Sensor data is manually timestamped with `InstantTimestamped::new()`.

**Current (Wrong):**
```rust
let timestamped = InstantTimestamped::new(sensor_value, timer.now());
tx.unbounded_send(timestamped)
```

**Should Be:**
```rust
tx.unbounded_send(sensor_value);
// Let Fluxion's timestamping operators handle it automatically
```

#### 2. **Stream Creation Pattern**
**Problem:** Using `rx.map(StreamItem::Value)` to manually wrap values.

**Current (Wrong):**
```rust
let stream1 = rx1.map(StreamItem::Value);
let stream2 = rx2.map(StreamItem::Value);
let stream3 = rx3.map(StreamItem::Value);
```

**Should Be:**
```rust
use fluxion_stream::prelude::*;
let stream1 = rx1.into_fluxion_stream();
let stream2 = rx2.into_fluxion_stream();
let stream3 = rx3.into_fluxion_stream();
```

#### 3. **Side Effects in Stream Pipeline**
**Problem:** Chart updates are performed inside `.map()` closures - side effects in the wrong place.

**Current (Wrong):**
```rust
let stream1 = rx1.map(move |timestamped| {
    chart1.add_point(timestamped.value.value);  // ❌ Side effect!
    let _ = chart1.render();                    // ❌ Side effect!
    StreamItem::Value(timestamped)
});
```

**Should Be:**
```rust
// Option 1: Use .inspect() for side effects
let stream1 = rx1
    .into_fluxion_stream()
    .inspect(|item| {
        if let StreamItem::Value(v) = item {
            chart1.add_point(v.value);
            let _ = chart1.render();
        }
    });

// Option 2: Fork the stream and consume separately
```

#### 4. **Sensor Frequencies Too High**
**Problem:** Sensors run at 100Hz, 50Hz, 25Hz - too fast to appreciate visualization.

**Current (Wrong):**
```rust
generate_sensor_stream(1, 100.0, tx1, running_clone.clone());  // 100Hz - too fast!
generate_sensor_stream(2, 50.0, tx2, running_clone.clone());   // 50Hz
generate_sensor_stream(3, 25.0, tx3, running_clone.clone());   // 25Hz
```

**Should Be:**
```rust
generate_sensor_stream(1, 2.0, tx1, running_clone.clone());   // 2Hz (500ms) - visible
generate_sensor_stream(2, 1.0, tx2, running_clone.clone());   // 1Hz (1s)
generate_sensor_stream(3, 0.5, tx3, running_clone.clone());   // 0.5Hz (2s)
```

#### 5. **Individual Sensor Charts Not Updating**
**Problem:** Only the combined output chart is updated. Individual sensor windows remain empty.

**Root Cause:** Chart updates mixed into stream transformation logic instead of proper visualization pipeline.

**Solution:** 
- Use `.inspect()` before combining to update individual charts
- Fork streams to separate visualization paths
- Or use broadcast channels to fan-out data

#### 6. **Limited Time Operator Usage**
**Problem:** Only `throttle` is currently implemented. The dashboard should showcase ALL time operators.

**Missing Operators:**
- **debounce**: Wait for quiet period before emitting
- **delay**: Add fixed delay to stream
- **sample**: Sample at fixed intervals  
- **timeout**: Error if no value within duration

**Should Have:**
```rust
let processed = combined_stream
    .debounce(Duration::from_millis(300))     // Wait for quiet
    .throttle(Duration::from_millis(100))     // Rate limit
    .delay(Duration::from_millis(500))        // Add latency
    .sample(Duration::from_millis(1000))      // Regular sampling
    .timeout(Duration::from_secs(5));         // Detect stalls
```

#### 7. **WasmTimer Not Exported**
**Problem:** `WasmTimer` is not publicly exported from `fluxion-stream-time` crate.

**Current Failure:**
```rust
use fluxion_stream_time::runtimes::wasm_implementation::WasmTimer;
// ❌ Error: "could not find `wasm_implementation` in `runtimes`"
```

**Required Library Fix:**
```rust
// In fluxion-stream-time/src/lib.rs
#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
pub use runtimes::wasm_impl::wasm_implementation::WasmTimer;

// Or provide a type alias:
#[cfg(all(feature = "runtime-wasm", target_arch = "wasm32"))]
pub type WasmTimer = runtimes::wasm_impl::wasm_implementation::WasmTimer;
```

## Recommended Refactoring Roadmap

### Phase 1: Core Architecture (High Priority)
1. Export `WasmTimer` publicly from library
2. Replace manual `InstantTimestamped` wrapping with automatic timestamping
3. Use `.into_fluxion_stream()` instead of `.map(StreamItem::Value)`
4. Remove side effects from stream transformations

### Phase 2: Visualization Pipeline
1. Move chart updates out of `.map()` closures
2. Use `.inspect()` or dedicated visualization consumers
3. Implement proper stream forking for individual sensor charts
4. Update all three sensor windows independently

### Phase 3: Operator Showcase  
1. Reduce sensor frequencies to 2Hz, 1Hz, 0.5Hz
2. Add UI controls for debounce, delay, sample, timeout
3. Wire up all time operators in pipeline
4. Add visual feedback showing each operator's effect

### Phase 4: Polish
1. Add per-operator metrics (events in/out, latency)
2. Improve error handling and user feedback
3. Optimize canvas rendering with `requestAnimationFrame`
4. Add export/logging functionality

## Project Structure

```
wasm-dashboard/
├── src/
│   ├── lib.rs              # WASM entry point
│   ├── app_streaming.rs    # Main dashboard logic (needs refactoring)
│   ├── stream_data.rs      # Sensor generation (needs refactoring)
│   ├── chart.rs            # Canvas rendering
│   ├── ui.rs               # UI components
│   └── sensors.rs          # Sensor simulator
├── www/
│   ├── index.html          # Dashboard UI
│   ├── styles.css          # Styling
│   ├── bootstrap.js        # WASM loader
│   └── pkg/                # Generated WASM (build output)
├── Cargo.toml
└── README.md               # This file
```

## Dependencies

- **fluxion-core**: Core streaming primitives (`StreamItem`, etc.)
- **fluxion-stream**: Operators (`combine_latest`, `filter_map`, etc.)
- **fluxion-stream-time**: Time operators (`throttle`, `debounce`, etc.)
- **wasm-bindgen**: Rust ↔ JavaScript interop
- **web-sys**: Web API bindings (DOM, Canvas, etc.)
- **gloo-timers**: WASM-compatible async timers
- **futures**: Async primitives and utilities

## Browser Compatibility

Requires WebAssembly support:
- ✅ Chrome/Edge 57+
- ✅ Firefox 52+
- ✅ Safari 11+

## Performance Considerations

- Current: 175 events/second (100Hz + 50Hz + 25Hz)
- Throttled to: ~10 UI updates/second
- Canvas rendering is synchronous and may block
- Consider `requestAnimationFrame` for smoother updates

## Contributing

When improving this example:
1. Follow the refactoring roadmap above
2. Fix architectural issues before adding features
3. Test in multiple browsers
4. Ensure all time operators are properly demonstrated
5. Document WASM-specific considerations

## License

Apache-2.0
