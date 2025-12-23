# Fluxion WASM Dashboard

An interactive browser-based dashboard demonstrating Fluxion's time-based stream operators.

## Features

This example showcases:
- 🔄 **Real-time data streams** - Three simulated sensors updating at different rates
- ⏱️ **Time operators** - debounce, throttle, delay, sample, and timeout
- 📊 **Live visualization** - Interactive charts showing operator effects
- 🎮 **Interactive controls** - Toggle operators and adjust parameters in real-time
- 📈 **Performance metrics** - Event counts, drop rates, and latency tracking

## Time Operators Demonstrated

1. **Debounce** (300ms) - Filters rapid changes, emits only after stability period
2. **Throttle** (100ms) - Rate limiting, maximum emission frequency
3. **Delay** (500ms) - Artificial latency for testing/simulation
4. **Sample** (1000ms) - Regular interval sampling regardless of source rate
5. **Timeout** (5000ms) - Detects stale streams and provides fallbacks

## Building

### Prerequisites

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Or with cargo
cargo install wasm-pack
```

### Build the WASM module

```bash
cd examples/wasm-dashboard
wasm-pack build --target web --release
```

## Running

### Option 1: Simple HTTP Server (Python)

```bash
cd www
python -m http.server 8080
# Open http://localhost:8080 in your browser
```

### Option 2: Simple HTTP Server (Node.js)

```bash
cd www
npx http-server -p 8080
# Open http://localhost:8080 in your browser
```

### Option 3: Using trunk (recommended for development)

```bash
# Install trunk
cargo install trunk

# Serve with hot reload
trunk serve
# Opens automatically in browser at http://127.0.0.1:8080
```

## Usage

1. **Open the dashboard** in your web browser
2. **Click "Start Sensors"** to begin data generation
3. **Toggle operators** to see their effects:
   - Click operator buttons to enable/disable
   - Adjust duration sliders to change timing parameters
4. **Observe the visualization**:
   - Raw events (blue) vs. processed events (green)
   - Event counters and metrics
   - Real-time charts showing data flow
5. **Export data** to download event logs as JSON

## Architecture

```
Sensor Streams (100Hz, 50Hz, 25Hz)
    ↓
combine_latest
    ↓
[Debounce] → [Throttle] → [Delay] → [Sample] → [Timeout]
    ↓
Output Stream → Canvas Visualization
```

Each operator can be toggled independently to see its individual effect on the stream.

## Browser Compatibility

- ✅ Chrome/Edge (recommended)
- ✅ Firefox
- ✅ Safari
- ⚠️ Requires WebAssembly support (all modern browsers)

## Performance Notes

The dashboard simulates high-frequency data streams (up to 100Hz) to demonstrate operator effectiveness. In production, adjust rates based on your use case.

## Next Steps

- Experiment with different operator combinations
- Adjust timing parameters to see behavior changes
- Study the code in `src/lib.rs` to see operator usage
- Extend with your own custom visualizations

## License

Licensed under the Apache License, Version 2.0. See LICENSE file.
