# WASM Dashboard Architecture

## Overview

The dashboard is now organized into clean, layered components following separation of concerns principles.

## File Structure

### Layer 1: Raw Data Generation
**File:** [raw_sensor.rs](src/raw_sensor.rs)
- **Purpose:** Generate raw sensor data without any Fluxion-specific concerns
- **Key Type:** `SensorValue` - Simple struct with `sensor_id` and `value`
- **Function:** `generate_raw_sensor()` - Async task that generates sine wave data
- **Output:** Sends to `mpsc::UnboundedSender<SensorValue>`

### Layer 2: Integration
**File:** [integration.rs](src/integration.rs)
- **Purpose:** Augment raw sensor data with timestamps for Fluxion
- **Key Type:** `WasmTimestamped<T> = InstantTimestamped<T, WasmTimer>`
- **Function:** `timestamped_sensor_stream()` - Wraps receiver and adds timestamps
- **Input:** `mpsc::UnboundedReceiver<SensorValue>`
- **Output:** `Stream<Item = WasmTimestamped<SensorValue>>`

### Layer 3: Aggregation
**File:** [aggregator.rs](src/aggregator.rs)
- **Purpose:** Combine multiple streams using Fluxion operators
- **Function:** `aggregate_sensors()` - Combines 3 streams, calculates average, throttles output
- **Operations:**
  - Wraps streams in `StreamItem::Value`
  - Uses `combine_latest()` to synchronize
  - Calculates average of 3 sensor values
  - Applies `throttle()` for rate limiting
- **Input:** 3 timestamped sensor streams
- **Output:** `Stream<Item = StreamItem<f64>>`

### Application Layer
**File:** [app_streaming.rs](src/app_streaming.rs)
- **Purpose:** Wire up the layers and handle UI interaction
- **Start Button Logic:**
  ```rust
  // Layer 1: Raw data
  let (tx1, rx1) = mpsc::unbounded();
  spawn_local(generate_raw_sensor(1, 100.0, tx1, running));

  // Layer 2: Integration
  let stream1 = timestamped_sensor_stream(rx1);

  // Layer 3: Aggregation
  let aggregated = aggregate_sensors(stream1, stream2, stream3, 100);

  // Process output
  spawn_local(process_aggregated_stream(aggregated, ...));
  ```

## Benefits of This Architecture

### 1. **Separation of Concerns**
- Raw data generation is independent of Fluxion
- Timestamping isolated in integration layer
- Stream operations cleanly separated

### 2. **Testability**
- Each layer can be tested independently
- Mock channels for testing integration layer
- Mock streams for testing aggregator

### 3. **Reusability**
- Raw sensor generator can be used in non-Fluxion contexts
- Integration layer reusable for any data type
- Aggregator patterns applicable to other stream combinations

### 4. **Clarity**
- Clear data flow: Raw → Timestamped → Aggregated → UI
- Each file has single responsibility
- Easy to understand and modify

## Data Flow

```
┌─────────────────┐
│ Raw Sensor 1    │──┐
│ (100Hz)         │  │
└─────────────────┘  │
                     ├──> Integration ──┐
┌─────────────────┐  │   (Timestamps)   │
│ Raw Sensor 2    │──┤                  │
│ (50Hz)          │  │                  ├──> Aggregator ──> UI
└─────────────────┘  │                  │   (combine_latest,
                     ├──> Integration ──┘    throttle)
┌─────────────────┐  │   (Timestamps)
│ Raw Sensor 3    │──┘
│ (25Hz)          │
└─────────────────┘
```

## Future Improvements

- Add `.inspect()` for side effects instead of `.map()`
- Reduce sensor frequencies (2Hz, 1Hz, 0.5Hz)
- Update individual sensor charts
- Add more time operators (debounce, delay, sample)
- Automatic timestamping in raw layer
