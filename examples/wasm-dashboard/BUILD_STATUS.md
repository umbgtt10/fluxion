# WASM Dashboard Example - Build Instructions

This example demonstrates all 5 time operators from `fluxion-stream-time` running in a browser using WebAssembly.

## What's Included

✅ **Complete Project Structure**
- Cargo.toml with WASM-compatible dependencies
- Full Rust source code (lib.rs, app.rs, chart.rs, sensors.rs, ui.rs, stream_pipeline.rs)
- HTML/CSS/JavaScript interface
- Dark-themed, responsive UI

✅ **Time Operators Demonstrated**
1. **Debounce** - Delays emissions until activity stops
2. **Throttle** - Limits emission rate  
3. **Delay** - Introduces fixed delay
4. **Sample** - Emits at regular intervals
5. **Timeout** - Fails if no emission within duration

✅ **Compilation Status**
- ✅ Compiles successfully for `wasm32-unknown-unknown` target
- ✅ WASM module built with `wasm-pack`
- ✅ All dependencies resolved
- ⚠️  Some dead code warnings (expected for scaffolding)

## Build Commands

### Build WASM module:
```powershell
wasm-pack build --target web --release --out-dir www/pkg
```

### Serve locally:
```powershell
# Option 1: Using Python (if installed)
cd www
python -m http.server 8080

# Option 2: Using Node.js http-server (if installed)
cd www
npx http-server -p 8080

# Option 3: Using trunk (recommended for development)
trunk serve
```

Then open http://localhost:8080 in your browser.

## Current Implementation Status

**Phase 1: Scaffolding (COMPLETE ✅)**
- [x] Project structure created
- [x] Dependencies configured for WASM
- [x] HTML interface with controls
- [x] CSS styling (dark theme)
- [x] JavaScript bootstrap
- [x] Rust modules structure
- [x] WASM compilation working

**Phase 2: Implementation (TODO)**
- [ ] Create sensor data generators (100Hz, 50Hz, 25Hz)
- [ ] Wire up time operators to actual streams
- [ ] Connect UI controls to operator toggles
- [ ] Implement chart rendering with real data
- [ ] Add metrics calculation (event count, drop rate, latency)
- [ ] Test all operators interactively

**Phase 3: Polish (TODO)**
- [ ] Add pipeline visualization
- [ ] Implement JSON export functionality
- [ ] Add helpful tooltips
- [ ] Performance optimization
- [ ] Cross-browser testing

## Next Steps for Implementation

The scaffolding is complete. To finish the implementation:

1. **Implement sensor streams** in `stream_pipeline.rs`:
   - Create interval-based value generators
   - Use `gloo-timers` for browser-compatible timers
   
2. **Wire up operator pipeline**:
   - Apply time operators based on toggle state
   - Configure durations from slider values
   
3. **Connect to UI**:
   - Update charts with real stream data
   - Calculate and display metrics
   - Handle start/stop/reset actions

4. **Test in browser**:
   - Verify all operators work correctly
   - Check performance with all 3 sensors running
   - Test toggle/slider interactions

## Architecture

```
┌─────────────┐
│   Browser   │
│             │
│  HTML/CSS   │◄── User Interaction
│  JavaScript │
└──────┬──────┘
       │
       │ WASM Boundary
       ▼
┌─────────────────────────────┐
│  Rust (WASM)                │
│                             │
│  Sensor1 (100Hz) ────┐      │
│  Sensor2 (50Hz)  ────┼──►   │
│  Sensor3 (25Hz)  ────┘      │
│         │                   │
│         ▼                   │
│  ┌──────────────────┐       │
│  │ Time Operators   │       │
│  │ - Debounce       │       │
│  │ - Throttle       │       │
│  │ - Delay          │       │
│  │ - Sample         │       │
│  │ - Timeout        │       │
│  └────────┬─────────┘       │
│           │                 │
│           ▼                 │
│    Output Stream            │
│           │                 │
└───────────┼─────────────────┘
            │
            ▼
      Update Chart & Metrics
```

## File Overview

- **`Cargo.toml`** - WASM-compatible dependencies with `runtime-wasm` feature
- **`src/lib.rs`** - Entry point, exports `start_dashboard()` to JavaScript
- **`src/app.rs`** - Main Dashboard struct coordinating everything
- **`src/stream_pipeline.rs`** - Time operator configuration and application
- **`src/sensors.rs`** - Sensor simulation state
- **`src/chart.rs`** - Canvas-based chart rendering
- **`src/ui.rs`** - DOM manipulation and event handlers
- **`www/index.html`** - Main web interface
- **`www/styles.css`** - Dark theme styling (~400 lines)
- **`www/bootstrap.js`** - WASM module loader

## Notes

- Uses `gloo-timers` for browser-compatible timing
- No server-side components needed - pure client-side WASM
- Optimized for size with `opt-level = "s"` and LTO
- All fluxion dependencies use `alloc` features (no std required)
