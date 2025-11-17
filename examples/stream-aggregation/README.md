# Stream Aggregation Example

> **Part of [Fluxion](../../README.md)** - A reactive stream processing library for Rust

A practical example demonstrating event-driven data aggregation using Fluxion stream operators.

## Overview

This example shows how to build a real-time data aggregation system that:

- Combines multiple event streams (sensor readings, alerts, commands)
- Maintains aggregated state using `combine_latest`
- Handles temporal ordering of events
- Processes events asynchronously with backpressure

## Architecture

```
┌─────────────┐
│   Sensors   │──┐
└─────────────┘  │
                 ├──► combine_latest ──► Aggregator ──► Output
┌─────────────┐  │
│   Alerts    │──┤
└─────────────┘  │
                 │
┌─────────────┐  │
│  Commands   │──┘
└─────────────┘
```

## Key Concepts

### Event Aggregation

The aggregator combines multiple streams using `combine_latest`, which:
- Maintains latest value from each stream
- Emits aggregated state on any stream update
- Preserves temporal ordering via sequence numbers

### Stream Combination

```rust
let aggregated = sensor_stream
    .combine_latest(alert_stream)
    .combine_latest(command_stream)
    .map(|combined| process_events(combined));
```

## Running the Example

```bash
cargo run --example stream-aggregation
```

## Use Cases

This pattern is useful for:
- IoT sensor data aggregation
- Real-time analytics dashboards
- Event-driven monitoring systems
- Multi-source data fusion

## License

MIT
