# Fluxion WASM Dashboard

A live WebAssembly dashboard demonstrating Fluxion's reactive stream operators with real-time sensor data visualization.

## Quick Start

### Prerequisites

Ensure you have the following tools installed:
- **Rust** (stable toolchain)
- **wasm-pack**: `cargo install wasm-pack`
- **Node.js** and **npm** (for the development server)

### Running the Dashboard

From the project root, run:

```powershell
.\scripts\run-dashboard.ps1
```

This script will:
1. Build the WASM module
2. Start the development server
3. Open the dashboard in your default browser at `http://localhost:8080`

## Concept

This example demonstrates **reactive stream processing** in a browser environment:

- **Three simulated sensors** emit values at different frequencies (500-4000ms intervals)
- Each value flows through a **processing pipeline** with multiple operators
- **Real-time visualization** shows the effect of each transformation
- The UI updates reactively as new values arrive

The dashboard serves as both a **working application** and an **educational tool** for understanding how reactive operators transform data streams over time.

## Technologies Used

- **Fluxion** - Reactive stream processing library with composable operators
- **WebAssembly (WASM)** - Rust code compiled to run in the browser
- **web-sys** - Rust bindings for Web APIs (DOM manipulation)
- **gloo-timers** - WASM-compatible async timer implementation
- **wasm-bindgen** - Interoperability layer between Rust and JavaScript

## Architecture

The application follows a **four-layer architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│          UI Layer (Web)                 │
│     DashboardUI (DashboardSink)         │
└─────────────────────────────────────────┘
                  ▲
                  │ trait boundary
                  │
┌─────────────────────────────────────────┐
│      Orchestration Layer                │
│  DashboardOrchestrator<P, S>            │
│    (Generic over StreamProvider         │
│         & DashboardSink)                │
└─────────────────────────────────────────┘
                  ▲
                  │ trait boundary
                  │
┌─────────────────────────────────────────┐
│       Processing Layer                  │
│   ProcessingLayer (StreamProvider)      │
│  • CombinedStream                       │
│  • ResultStreams                        │
└─────────────────────────────────────────┘
                  ▲
                  │
┌─────────────────────────────────────────┐
│         Source Layer                    │
│  • Sensors (raw data generation)        │
│  • SensorStreams (timestamping)         │
│  • SourceLayer (encapsulation)          │
└─────────────────────────────────────────┘
```

### Layer Responsibilities

**Source Layer** ([`src/source/`](src/source/))
- Generates raw sensor data at configurable intervals
- Adds timestamps to values using `WasmTimer`
- Provides shareable streams via `FluxionShared`

**Processing Layer** ([`src/processing/`](src/processing/))
- **CombinedStream**: Merges sensors with `combine_latest`, filters, and maps
- **ResultStreams**: Applies time-based operators (debounce, delay, sample, throttle, timeout)
- Implements the `StreamProvider` trait to expose 9 output streams

**Orchestration Layer** ([`src/processing/dashboard_orchestrator.rs`](src/processing/dashboard_orchestrator.rs))
- **Pure orchestrator** - knows nothing about concrete implementations
- Generic over `StreamProvider` (stream source) and `DashboardSink` (UI target)
- Wires each stream to its corresponding UI update method
- Manages task lifecycle and cancellation

**UI Layer** ([`src/gui/`](src/gui/))
- Web-based interface with 9 display windows
- Implements the `DashboardSink` trait with 10 update methods
- Uses `web-sys` for DOM manipulation
- Handles button events (start/stop/close)

### Design Principles

**Dependency Inversion**
- The orchestrator depends on abstractions (`StreamProvider`, `DashboardSink`), not concrete types
- Makes the system testable with mock implementations

**Separation of Concerns**
- Each layer has a single, well-defined responsibility
- Business logic isolated from UI code
- Stream creation separated from stream consumption

**Trait Boundaries**
- Layers communicate through trait interfaces
- Enables independent evolution of implementations
- Supports multiple UI backends (web, terminal, etc.)

## Fluxion Operators in Action

This dashboard demonstrates 8 reactive stream operators. Each operator's behavior is **visible in real-time** through dedicated display windows.

### Stream Combination

#### `combine_latest`
**Purpose:** Merges multiple streams, emitting whenever any source produces a value.

**Implementation:** [`src/processing/combined_stream.rs:35-44`](src/processing/combined_stream.rs#L35-L44)

```rust
sensors
    .sensor1()
    .subscribe()
    .unwrap()
    .combine_latest(
        vec![
            sensors.sensor2().subscribe().unwrap(),
            sensors.sensor3().subscribe().unwrap(),
        ],
        |state| {
            let sum: u32 = state.values().iter().map(|v| v.value).sum();
            sum % 3 == 0  // Filter predicate
        },
    )
```

**Why:** Combines three independent sensor streams into a single stream containing the latest value from each sensor. This enables downstream operators to work with aggregated data.

**Output:** Combined state with latest values from all three sensors, emitted whenever any sensor produces a value.

---

### Filtering & Transformation

#### `filter` (via `combine_latest` predicate)
**Purpose:** Allows only values that satisfy a predicate to pass through.

**Implementation:** [`src/processing/combined_stream.rs:44-47`](src/processing/combined_stream.rs#L44-L47)

```rust
|state| {
    let sum: u32 = state.values().iter().map(|v| v.value).sum();
    sum % 3 == 0  // Only sums divisible by 3
}
```

**Why:** Demonstrates conditional stream processing built into `combine_latest`. Only sums divisible by 3 proceed to the dashboard.

---

#### `map_ordered`
**Purpose:** Transforms each value using a function while preserving order.

**Implementation:** [`src/processing/combined_stream.rs:49-52`](src/processing/combined_stream.rs#L49-L52)

```rust
.map_ordered(|state| {
    let sum: u32 = state.values().iter().map(|v| v.value).sum();
    WasmTimestamped::with_timestamp(sum, state.timestamp())
})
```

**Why:** Reduces the three-sensor state to a single timestamped sum value for simpler downstream processing.

---

### Time-Based Operators

All time operators are applied to the combined stream in [`src/processing/result_streams.rs`](src/processing/result_streams.rs).

#### `debounce` (700ms)
**Purpose:** Emits a value only after the stream has been idle for the specified duration.

**Implementation:** [`src/processing/result_streams.rs:42-51`](src/processing/result_streams.rs#L42-L51)

```rust
self.combined_stream
    .subscribe()
    .debounce(Duration::from_millis(700))
```

**Why:** Useful for handling "bursty" data. Only processes values after activity settles. Common in search-as-you-type scenarios.

**Behavior:** If values arrive continuously, debounce waits for a 700ms quiet period before emitting the last value.

---

#### `delay` (1000ms)
**Purpose:** Time-shifts all values by a fixed duration.

**Implementation:** [`src/processing/result_streams.rs:54-63`](src/processing/result_streams.rs#L54-L63)

```rust
self.combined_stream
    .subscribe()
    .delay(Duration::from_millis(1000))
```

**Why:** Demonstrates temporal shifting of entire streams. Useful for synchronization or controlled processing delays.

**Behavior:** Every value is delayed by exactly 1 second before appearing in the delayed window.

---

#### `sample` (1000ms)
**Purpose:** Periodically emits the most recent value at fixed intervals.

**Implementation:** [`src/processing/result_streams.rs:66-75`](src/processing/result_streams.rs#L66-L75)

```rust
self.combined_stream
    .subscribe()
    .sample(Duration::from_millis(1000))
```

**Why:** Reduces data rate while maintaining current state. Ideal for periodic polling or reducing UI update frequency.

**Behavior:** Every 1 second, emits whatever the latest combined value is, regardless of how many values arrived in that interval.

---

#### `throttle` (800ms)
**Purpose:** Enforces a maximum emission rate by dropping values that arrive too quickly.

**Implementation:** [`src/processing/result_streams.rs:78-87`](src/processing/result_streams.rs#L78-L87)

```rust
self.combined_stream
    .subscribe()
    .throttle(Duration::from_millis(800))
```

**Why:** Rate limiting for downstream consumers. Prevents overwhelming slow processors or APIs with requests.

**Behavior:** After emitting a value, ignores all subsequent values for 800ms. First value after the window reopens is emitted immediately.

---

#### `timeout` (2000ms)
**Purpose:** Emits an error if no value arrives within the specified duration.

**Implementation:** [`src/processing/result_streams.rs:90-99`](src/processing/result_streams.rs#L90-L99)

```rust
self.combined_stream
    .subscribe()
    .timeout(Duration::from_millis(2000))
```

**Why:** Detects stream stalls or network failures. Essential for building robust systems that handle missing data.

**Behavior:** If more than 2 seconds pass without an emission, the stream produces an error. The error window displays "TIMEOUT!" in red.

---

## Project Structure

```
wasm-dashboard/
├── src/
│   ├── lib.rs                    # WASM entry point
│   ├── source/                   # Source Layer
│   │   ├── sensor.rs             # Raw sensor generation
│   │   ├── sensor_streams.rs     # Timestamped shared streams
│   │   ├── source_layer.rs       # Layer encapsulation
│   │   └── ...
│   ├── processing/               # Processing Layer
│   │   ├── combined_stream.rs    # Stream combination
│   │   ├── result_streams.rs     # Time operators
│   │   ├── processing_layer.rs   # StreamProvider impl
│   │   ├── stream_provider.rs    # Trait definition
│   │   └── dashboard_orchestrator.rs  # Orchestration
│   └── gui/                      # UI Layer
│       ├── dashboard_ui.rs       # Web UI implementation
│       └── dashboard_sink.rs     # Trait definition
├── www/
│   ├── styles.css                # Dashboard styling
│   └── pkg/                      # Generated WASM (gitignored)
├── index.html                    # HTML structure
├── scripts/
│   └── run-dashboard.ps1         # Build & serve automation
└── Cargo.toml
```

## Learning Resources

- **Fluxion Documentation:** [Main README](../../README.md)
- **Operator Details:** See inline documentation in [`src/processing/`](src/processing/)
- **Architecture Pattern:** This example demonstrates the Dependency Inversion Principle and trait-based design

---

**Built with Fluxion** - A composable reactive streams library for Rust
