# Embassy Sensor Fusion Example

This example demonstrates **Fluxion reactive streams in a pure `no_std` embedded environment** with Embassy runtime, running on ARM Cortex-M4F in QEMU.

## Overview

Three simulated sensors (temperature, pressure, humidity) run concurrently, each with its own reactive processing pipeline. The streams are merged with stateful aggregation and logged via semihosting.

**Key Features:**
- ✅ Pure `no_std` + `alloc` - no standard library
- ✅ ARM Cortex-M4F target (`thumbv7em-none-eabihf`)
- ✅ Embassy async runtime (`arch-cortex-m`)
- ✅ Custom SysTick-based time driver
- ✅ QEMU emulation (no physical hardware needed)
- ✅ All Fluxion time-based operators working
- ✅ 64KB heap via `embedded-alloc`

## Architecture

```
Temperature Sensor ───────────┐
  │ tap (log raw value)       │
  │ distinct_until_changed    │
  │ debounce(500ms)           │
  └───────────────────────────┤
                              │
Pressure Sensor ──────────────┤
  │ distinct_until_changed_by │    merge_with
  │ filter_ordered (>1000hPa) ├──► (stateful aggregation)
  │ throttle(750ms)           │         │
  └───────────────────────────┤         │
                              │         ├──► Complete aggregates
Humidity Sensor ──────────────┤         │     (when all 3 sensors ready)
  │ window_by_count(2)        │         │
  │ skip_items(1)             │         │
  │ sample(100ms)             │         │
  └───────────────────────────┘         │
                                        │
                                        └──► Log: "Temp: 280K | Press: 1031hPa | Hum: 77%"
```

### Pipeline Details

**Temperature Stream:**
- `tap`: Log raw sensor values
- `distinct_until_changed`: Skip duplicate readings
- `debounce(500ms)`: Stabilize noisy readings

**Pressure Stream:**
- `distinct_until_changed_by`: Custom comparison (delta > 10 hPa)
- `filter_ordered`: Only process pressures > 1000 hPa
- `throttle(750ms)`: Rate limit emissions

**Humidity Stream:**
- `window_by_count(2)`: Batch readings into pairs
- `skip_items(1)`: Skip first incomplete window
- `sample(100ms)`: Sample at fixed intervals

**Fusion:**
- `merge_with`: Combine streams with shared state
- Stateful aggregation tracks latest values from all sensors
- Emits complete aggregate when all 3 sensors have data

## Requirements

### 1. Install Rust Toolchain

```powershell
# Install rustup if not already installed
# https://rustup.rs/

# Add ARM Cortex-M target
rustup target add thumbv7em-none-eabihf
```

### 2. Install QEMU

**Windows (Scoop):**
```powershell
scoop install qemu
```

**Windows (Chocolatey):**
```powershell
choco install qemu
```

**macOS (Homebrew):**
```bash
brew install qemu
```

**Linux (apt):**
```bash
sudo apt install qemu-system-arm
```

Verify installation:
```powershell
qemu-system-arm --version
```

### 3. Verify Dependencies

The example uses these embedded-specific dependencies:
- `embassy-executor` with `arch-cortex-m` (no std)
- `embassy-time` with `generic-queue-8` (no std)
- `embedded-alloc` for heap allocation
- `cortex-m-semihosting` for console output in QEMU
- Custom time driver using SysTick

## Building

From the `embassy-sensors` directory:

```powershell
# Build for ARM Cortex-M4F
cargo build --release --target thumbv7em-none-eabihf
```

The binary will be at:
```
target/thumbv7em-none-eabihf/release/embassy-sensors
```

## Running in QEMU

### Option 1: Run Script (Recommended)

```powershell
.\scripts\run_qemu.ps1
```

This script:
1. Builds the release binary
2. Launches QEMU with semihosting enabled
3. Shows console output
4. Exits cleanly after 30 seconds

### Option 2: Manual QEMU Launch

```powershell
qemu-system-arm `
    -cpu cortex-m4 `
    -machine mps2-an386 `
    -nographic `
    -semihosting-config enable=on,target=native `
    -kernel target/thumbv7em-none-eabihf/release/embassy-sensors
```

**QEMU Arguments:**
- `-cpu cortex-m4`: Cortex-M4 processor
- `-machine mps2-an386`: ARM MPS2-AN386 board (25MHz)
- `-nographic`: No graphical window
- `-semihosting-config`: Enable console I/O
- `-kernel`: Load the ELF binary

### Expected Output

```
Embassy Sensor Fusion System Starting
Runtime: 30 seconds
Time driver initialized
Fusion task started - demonstrating operators and merge_with
Building reactive pipeline with multiple operators...

Temperature: tap -> distinct_until_changed -> debounce(500ms)
Pressure: distinct_until_changed_by -> filter_ordered(>1000hPa) -> throttle(750ms)
Humidity: window_by_count(2) -> skip_items(1) -> sample(100ms)

Merging streams with stateful aggregation...

Humidity sensor task started
Sensor: 24%
timeout: 200 ms
Pressure sensor task started
Sensor: 1024 hPa
timeout: 622 ms
Temperature sensor task started
Sensor: 297 C
timeout: 522 ms
Raw temperature: 297 C
...
Complete sensor aggregate received: Temp: 280K | Press: 1031hPa | Hum: 77% | Delta: 20 | Upd: 74
Timeout reached - initiating shutdown
Pressure sensor task stopped
Temperature sensor task stopped
Humidity sensor task stopped
Stream ended
System shutdown complete
```

The example runs for **30 seconds**, producing aggregate sensor readings every few hundred milliseconds.

## Project Structure

```
embassy-sensors/
├── Cargo.toml              # Dependencies (Embassy, Fluxion, embedded libs)
├── .cargo/
│   └── config.toml         # Target and runner configuration
├── memory.x                # Linker script (Flash: 0x00000000, RAM: 0x20000000)
├── build.rs                # Build script for linker arguments
├── scripts/
│   └── run_qemu.ps1        # Automated build + QEMU launcher
└── src/
    ├── main.rs             # Entry point, heap init, task spawning
    ├── time_driver.rs      # Custom SysTick-based embassy-time driver
    ├── logging.rs          # Custom info!/warn!/error! macros (semihosting)
    ├── fusion.rs           # Stream fusion with merge_with
    ├── sensors/            # Temperature, pressure, humidity tasks
    ├── types/              # Sensor data types (with timestamps)
    └── aggregate/          # SensorAggregate state + Display impl
```

## Implementation Details

### Memory Layout

The example uses a custom linker script (`memory.x`) for QEMU's MPS2-AN386:
```ld
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 4096K
  RAM   : ORIGIN = 0x20000000, LENGTH = 4096K
}
```

### Time Driver

A custom `embassy-time` driver (`src/time_driver.rs`) uses SysTick:
- Runs at **1 kHz** (1ms ticks)
- MPS2-AN386 clock: 25 MHz → SysTick reload: 25,000
- Implements `embassy_time_driver::Driver` trait
- `now()`: Returns tick counter
- `schedule_wake()`: Wakes executor immediately (SysTick handles polling)

**Key insight**: Calling `waker.wake_by_ref()` in `schedule_wake()` ensures the executor stays responsive even in QEMU's emulation environment.

### Heap Allocation

Uses `embedded-alloc` with a **64 KB** static heap:
```rust
static mut HEAP_MEM: [MaybeUninit<u8>; 64 * 1024] = ...;
unsafe { HEAP.init(addr, size) }
```

Required for:
- `async-channel` queues
- `Vec` in stream operators (e.g., `window_by_count`)
- Dynamic stream state

### Panic Handler

Uses `panic-semihosting` to print panic messages to QEMU console before halting.

## Troubleshooting

### Build Fails: "can't find crate for `std`"
✅ **Solution**: Ensure `#![no_std]` is at the top of `main.rs` and all dependencies have `default-features = false`.

### QEMU Hangs After Launch
✅ **Solution**: Check that semihosting is enabled: `-semihosting-config enable=on,target=native`

### No Console Output
✅ **Solution**: Ensure `cortex-m-semihosting` is in `Cargo.toml` and the `info!` macro uses `hprintln!`.

### Linker Error: "undefined symbol: _embassy_time_now"
✅ **Solution**: The custom time driver must implement `embassy_time_driver::time_driver_impl!` macro.

### Tasks Don't Run After First Sensor Reading
✅ **Solution**: `schedule_wake()` must call `waker.wake_by_ref()` to ensure executor polling.

### QEMU Doesn't Exit (Ctrl+C Doesn't Work)
✅ **Solution**: Call `cortex_m_semihosting::debug::exit(EXIT_SUCCESS)` at the end of `main()`.

## Features Demonstrated

### Core Features
- ✅ **`no_std` + `alloc`**: Pure embedded environment
- ✅ **Embassy async runtime**: `arch-cortex-m` executor
- ✅ **Custom time driver**: SysTick-based implementation
- ✅ **Heap allocation**: 64KB static heap with `embedded-alloc`
- ✅ **QEMU emulation**: Runs without physical hardware

### Fluxion Operators
- ✅ **Time-based**: `debounce`, `throttle`, `sample`, `delay`, `timeout`
- ✅ **Transformations**: `map`, `filter`, `scan`, `tap`
- ✅ **Filtering**: `filter_ordered`, `distinct_until_changed`, `distinct_until_changed_by`
- ✅ **Windowing**: `window_by_count`, `skip_items`, `take`
- ✅ **Fusion**: `merge_with` with stateful aggregation
- ✅ **Cancellation**: `CancellationToken` for graceful shutdown

### Embassy Features
- ✅ **Task spawning**: `#[embassy_executor::task]` macro
- ✅ **Async timers**: `Timer::after(Duration)`
- ✅ **Concurrency**: Multiple sensor tasks running simultaneously
- ✅ **Time tracking**: Monotonic timestamps with `EmbassyInstant`

## Next Steps

### Run on Real Hardware

To run on physical ARM Cortex-M hardware:

1. **Update `memory.x`** with your MCU's memory map
2. **Choose appropriate Embassy HAL**:
   ```toml
   # For STM32F4
   embassy-stm32 = { features = ["stm32f429zi", "time-driver-any"] }

   # For nRF52
   embassy-nrf = { features = ["nrf52840", "time-driver-rtc1"] }
   ```
3. **Remove custom time driver** (use HAL's built-in driver)
4. **Flash with `probe-rs` or OpenOCD**:
   ```bash
   cargo flash --release --chip STM32F429ZITx
   ```

### Modify the Example

**Add a new sensor:**
1. Create `src/sensors/your_sensor.rs`
2. Define a data type in `src/types/`
3. Add a channel and spawn task in `main.rs`
4. Add `merge_with` clause in `fusion.rs`

**Change timing:**
- Adjust `Duration::from_millis()` in sensor tasks
- Modify operator durations (debounce, throttle, sample)
- Change runtime: `Timer::after(Duration::from_secs(30))`

**Add more operators:**
- See [fluxion-stream](../../fluxion-stream) for available operators
- Chain operators in sensor pipelines
- Add filtering/transformation logic

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## References

- [Embassy Async Framework](https://embassy.dev/)
- [Fluxion Reactive Streams](https://github.com/yourusername/fluxion)
- [QEMU ARM Emulation](https://www.qemu.org/docs/master/system/arm/mps2.html)
- [ARM Cortex-M4 Technical Reference](https://developer.arm.com/documentation/100166/0001)

