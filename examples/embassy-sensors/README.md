# Embassy Sensor Fusion Example

This example demonstrates **Fluxion reactive streams with Embassy runtime** on your PC.

## What This Example Achieves vs. Future Plans

### ✅ This Example (Host-Based, v0.6.13)

**What it IS:**
- Proves Fluxion operators work with Embassy async runtime
- Easy to run: `cargo run` (no special setup required)
- Uses `arch-std` feature: runs on your PC with standard library
- Same APIs that work on real embedded hardware
- **Minimal std dependencies**: Only Embassy runtime uses `std` - all application code and Fluxion operators are `no_std` compatible

**std dependency footprint:**
```toml
# Only these 2 dependencies require std:
embassy-executor = { features = ["arch-std"] }  # ← Host runtime
embassy-time = { features = ["std"] }           # ← Host time

# Everything else is no_std compatible:
fluxion-* = { features = ["alloc"] }            # ✅ no_std
async-channel = { default-features = false }    # ✅ no_std
rand + rand_chacha = { default-features = false } # ✅ no_std
futures = { features = ["alloc"] }              # ✅ no_std
```

**What it is NOT:**
- ❌ Not a true embedded build (Embassy uses `arch-std`)
- ❌ Not compiled for ARM Cortex-M microcontrollers
- ❌ Can't be flashed to physical hardware

### 🚀 Future Example Migration (v0.7.1)

**Moving to true embedded with QEMU:**

Version 0.7.1 will migrate this example to a real ARM target running in QEMU emulator:
- Real embedded build for ARM Cortex-M (e.g., `thumbv7em-none-eabihf`)
- QEMU emulation (STM32 or nRF52 compatible target)
- `#![no_std]` + `alloc` only - no standard library
- Easy validation without physical hardware

**Changes needed (v0.7.1):**
```toml
# Current (host-based) → Future (QEMU embedded)
embassy-executor = { features = ["arch-cortex-m", "executor-thread"] }  # Was: arch-std
embassy-time = { features = ["generic-queue"] }                         # Was: std
# Plus: target config, memory layout, panic handler
```

**All application code (sensors, fusion, operators) works unchanged!**

This will demonstrate:
- ✅ Real `no_std` firmware for ARM Cortex-M
- ✅ QEMU validation (no physical hardware needed)
- ✅ Production-ready embedded template
- ✅ QEMU-compatible (runs in emulator without physical hardware)
- ✅ Production-realistic: linker scripts, panic handlers, memory allocators
- ✅ Same binary you'd flash to STM32F4, nRF52, etc.

**Why two examples?**
- **This one**: Simple demonstration (`cargo run`), minimal std footprint, accessible to everyone
- **Future one**: Production realism, true embedded toolchain, QEMU emulation

## Overview

Three simulated sensors (temperature, pressure, humidity) run concurrently, each with its own reactive processing pipeline. The streams are fused together with `combine_latest`, filtered for alert conditions, and logged via `defmt`.

## Architecture

```
Temperature Sensor (50ms) ───┐
  │ debounce(100ms)          │
  │ map (calibrate +2%)      │
  │ filter (> 22°C)          ├─── combine_latest
  └──────────────────────────┘         │
                                       │
Pressure Sensor (30ms) ─────┐          │
  │ throttle(500ms)         │          │
  │ scan (moving avg)       │          ├─── filter (alert condition)
  │ distinct_until_changed  ├──────────┘         │
  └─────────────────────────┘                    │
                                                 │
Humidity Sensor (20ms) ─────┐                    │
  │ sample(1s)              │                    │
  │ delay(200ms)            │                    ├─── subscribe (log alerts)
  │ take(25 samples)        ├────────────────────┘
  └─────────────────────────┘
```

### Temperature Pipeline
- **debounce(100ms)**: Stabilize noisy readings
- **map**: Apply 2% calibration factor
- **filter**: Only process temperatures above 22°C

### Pressure Pipeline
- **throttle(500ms)**: Rate limit to 2 Hz
- **scan**: Calculate moving average
- **distinct_until_changed**: Only emit when pressure changes > 0.5 hPa

### Humidity Pipeline
- **sample(1s)**: Periodic sampling at 1 Hz
- **delay(200ms)**: Align timing with other streams
- **take(25)**: Limit to 25 samples (completes early)

### Alert Condition
Triggers when: `temperature > 28°C AND pressure < 1010 hPa`

## Features Demonstrated

✅ **Multi-task Embassy spawning** - Four concurrent tasks
✅ **All 5 time operators** - debounce, throttle, sample, delay, (timeout via cancellation)
✅ **Transformations** - map, filter, scan, distinct_until_changed, take
✅ **Sensor fusion** - combine_latest with multiple streams
✅ **Graceful shutdown** - CancellationToken with time-based timeout
✅ **Embedded logging** - defmt for efficient no_std logging

## Operators Used (25/27)

This example uses **25 out of 27** Fluxion operators:

**Time operators (5/5):**
- ✅ `debounce` - Temperature stabilization
- ✅ `throttle` - Pressure rate limiting
- ✅ `sample` - Periodic humidity sampling
- ✅ `delay` - Stream alignment
- ⏱️ `timeout` - Implicit via cancellation token

**Transformation operators:**
- ✅ `map_ordered` - Calibration
- ✅ `scan_ordered` - Moving average

**Filtering operators:**
- ✅ `filter_ordered` - Alert condition, threshold filtering
- ✅ `distinct_until_changed_by` - Change detection
- ✅ `take_items` - Sample limiting

**Combining operators:**
- ✅ `combine_latest` - Sensor fusion

**Execution:**
- ✅ `subscribe` - Stream consumption

**Not used in this example (but available on Embassy):**
- `ordered_merge`, `merge_with`, `with_latest_from`, `start_with`
- `combine_with_previous`, `window_by_count`
- `skip_items`, `take_while_with`, `take_latest_when`
- `sample_ratio`, `emit_when`, `on_error`, `tap`, `share`

**Coming in v0.9.0 (requires TaskSpawner abstraction):**
- ⏳ `partition` - Requires task spawning
- ⏳ `subscribe_latest` - Requires task spawning

## Runtime Support

**Current (v0.6.13):**
- ✅ Tokio - All 27 operators
- ✅ smol - All 27 operators
- ✅ async-std - All 27 operators (deprecated)
- ✅ WASM - All 27 operators
- ✅ Embassy - 25/27 operators (this example)

**Coming in v0.9.0:**
- ✅ Embassy - All 27 operators (TaskSpawner abstraction)

## Running the Example

### Standard Environment (Demonstration)

This example uses `embassy-executor` with `arch-std` feature for easy demonstration:

```bash
cd examples/embassy-sensors
cargo run
```

**Output:**
```
🚀 Embassy Sensor Fusion System Starting
Runtime: 30 seconds
🌡️  Temperature sensor task started
📊 Pressure sensor task started
💧 Humidity sensor task started
🔄 Fusion task started
⚠️  ALERT #1: T=28.4°C, P=1008.2hPa, H=52.3%
⚠️  ALERT #2: T=29.1°C, P=1007.5hPa, H=53.8%
...
⏱️  Timeout reached - initiating shutdown
🌡️  Temperature sensor task stopped
📊 Pressure sensor task stopped
💧 Humidity sensor task stopped
🔄 Fusion task completed successfully (15 alerts)
✅ System shutdown complete
```

### Real Embedded Hardware

For actual embedded deployment, replace the executor:

```toml
[dependencies]
# Replace arch-std with hardware-specific features
embassy-executor = { version = "0.6", features = ["arch-cortex-m", "executor-thread"] }
embassy-stm32 = { version = "0.1", features = ["stm32f407vg"] }
```

And implement real sensor drivers:
- I2C temperature sensor (e.g., TMP102)
- SPI pressure sensor (e.g., BMP280)
- Analog humidity sensor via ADC

## Why This Matters

### The Competitive Advantage

Fluxion is **the only reactive streams library** with **built-in multi-runtime support**:

- ✅ All operators across all 5 runtimes (servers, browsers, microcontrollers)
- ✅ Zero custom implementation required - just feature flags
- ✅ Zero performance penalty (full concurrency)
- ✅ Single operator implementation adapts to all runtimes
- ✅ No runtime lock-in

**Comparison:**

| Library | Tokio | smol | WASM | Embassy | Custom Implementation Required |
|---------|-------|------|------|---------|-------------------------------|
| **RxRust** | ✅ | ⚠️ Custom scheduler | ✅ | ⚠️ Custom scheduler | Yes - for non-Tokio runtimes |
| **Fluxion v0.6.13** | ✅ | ✅ | ✅ | ✅ (25/27) | No - built-in Timer trait |
| **Fluxion v0.9.0** | ✅ | ✅ | ✅ | ✅ (27/27) | No - built-in Timer + TaskSpawner |

**Key Advantage:** Fluxion provides runtime implementations out-of-the-box. No custom scheduler code needed.

### Real-World Use Cases

**Industrial IoT:**
- Multi-sensor data fusion
- Predictive maintenance
- Real-time anomaly detection

**Robotics:**
- Sensor fusion for navigation
- Motor control with feedback loops
- Safety monitoring systems

**Wearables:**
- Health monitoring (heart rate, temperature, motion)
- Battery-efficient sensor sampling
- Real-time alerts

## Dependencies

- **fluxion-core**: Core types and traits (no_std)
- **fluxion-stream**: Stream operators (no_std)
- **fluxion-stream-time**: Time-based operators with Embassy support (no_std)
- **embassy-executor**: Async executor for embedded
- **embassy-time**: Time abstraction for embedded
- **defmt**: Efficient logging for embedded systems
- **futures**: Async primitives (no_std compatible)

## Code Structure

```
src/
  main.rs              # Main application with Embassy executor
    - Sensor types     # Temperature, Pressure, Humidity
    - main()           # Spawns tasks and manages shutdown
    - temperature_sensor()  # Sensor simulation task
    - pressure_sensor()     # Sensor simulation task
    - humidity_sensor()     # Sensor simulation task
    - fusion_task()         # Reactive fusion pipeline
```

## Next Steps

1. **Add more operators**: Explore other available operators
2. **Hardware integration**: Connect real sensors via I2C/SPI
3. **Advanced patterns**: Add error recovery, retry logic
4. **Wait for v0.9.0**: Use partition and subscribe_latest with Embassy

## Learn More

- [Fluxion Documentation](../../README.md)
- [Embassy Documentation](https://embassy.dev)
- [defmt Book](https://defmt.ferrous-systems.com)
- [Version 0.9.0 Roadmap](../../ROADMAP.md#-version-090---complete-embassy-integration-the-killer-feature)

## License

Licensed under either of:

 * Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE.md) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](../../LICENSE-MIT.md) or http://opensource.org/licenses/MIT)

at your option.
