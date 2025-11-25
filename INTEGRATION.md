# Integration Guide

This guide describes the three fundamental patterns for integrating events with Fluxion's ordered stream processing.

## Table of Contents

1. [Overview](#overview)
2. [Pattern 1: Intrinsic Ordering (Production)](#pattern-1-intrinsic-ordering-production)
3. [Pattern 2: Extrinsic Ordering (Testing)](#pattern-2-extrinsic-ordering-testing)
4. [Pattern 3: Wrapper Ordering (Integration)](#pattern-3-wrapper-ordering-integration)
5. [Comparison Table](#comparison-table)
6. [Choosing the Right Pattern](#choosing-the-right-pattern)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

---

## Overview

Fluxion processes streams of timestamped events. The `HasTimestamp` trait is the core abstraction that defines how events are sequenced (read-only access), while `Timestamped` extends it with construction methods. There are three main patterns for providing timestamp information:

1. **Intrinsic Timestamps** - Events carry their own timestamps (production)
2. **Extrinsic Timestamps** - Test infrastructure controls timestamps (testing)
3. **Wrapper Timestamps** - Adapter adds timestamps at boundaries (integration)

## Pattern 1: Intrinsic Timestamps (Production)

**When to use:** Your domain events already have timestamps, sequence numbers, or other inherent ordering information.

**Typical scenarios:**
- Event sourcing systems
- Message queue consumers (Kafka, RabbitMQ)
- IoT sensor data
- Log aggregation
- Time-series data

### Implementation

```rust
use fluxion_core::HasTimestamp;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SensorReading {
    timestamp: u64,  // Events carry their own temporal order
    sensor_id: String,
    temperature: f64,
}

impl HasTimestamp for SensorReading {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp  // Use the event's intrinsic timestamp
    }
}
```

**Note:** For intrinsic ordering, you typically only need to implement `HasTimestamp`.
Implement `Timestamped` only if you need construction methods and access to an inner wrapped value (`with_timestamp`, `into_inner`, etc.).

### Usage

```rust
use fluxion_rx::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();

    // Events are already ordered by their timestamp
    let stream = FluxionStream::from_unbounded_receiver(rx)
        .filter_ordered(|reading| &*reading.temperature > 25.0);

    // Stream processing respects the intrinsic timestamp ordering
}
```

**Key points:**
- ✅ Events reflect real-world temporal order
- ✅ No additional infrastructure needed
- ✅ Ordering is a property of the domain model

## Pattern 2: Extrinsic Timestamps (Testing)

**When to use:** You need fine-grained control over event timestamps, typically in tests.

**Typical scenarios:**
- Unit testing ordered operators
- Simulating out-of-order delivery
- Testing reordering logic
- Reproducing specific sequence scenarios
- Testing duplicate detection

### Implementation

Fluxion provides the `Sequenced<T>` wrapper in the `fluxion-test-utils` crate:

```rust
use fluxion_test_utils::Sequenced;

// Your domain type doesn't need timestamp information
#[derive(Clone, Debug)]
struct Event {
    data: String,
}

// Test infrastructure adds timestamps
let event1 = Sequenced::with_timestamp(Event { data: "first".into() }, 100);
let event2 = Sequenced::with_timestamp(Event { data: "second".into() }, 200);
```

### Usage

```rust
use fluxion_rx::prelude::*;
use fluxion_test_utils::Sequenced;
use tokio::sync::mpsc;
use futures::StreamExt;

#[derive(Clone, Debug)]
struct Event {
    data: String,
}

#[tokio::test]
async fn test_ordered_filtering() {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut stream = FluxionStream::from_unbounded_receiver(rx)
        .filter_ordered(|e| &*e.data.starts_with('f'));

    // Push events in arbitrary order
    tx.send(Sequenced::with_timestamp(Event { data: "third".into() }, 300)).unwrap();
    tx.send(Sequenced::with_timestamp(Event { data: "first".into() }, 100)).unwrap();
    tx.send(Sequenced::with_timestamp(Event { data: "second".into() }, 200)).unwrap();
    drop(tx);

    // Stream will reorder by timestamp: 100, 200, 300
    // Then filter to only: 100 ("first")
    let result = stream.next().await.unwrap();
    assert_eq!(&*result.data, "first");
}
```

**Key points:**
- ✅ Full control over event sequencing
- ✅ Domain objects stay clean (no test-specific fields)
- ✅ Perfect for testing edge cases (duplicates, gaps, reordering)
- ✅ Uses `Sequenced<T>` wrapper from `fluxion-test-utils`

## Pattern 3: Wrapper Timestamps (Integration)

**When to use:** Integrating with external systems that don't provide timestamps, or when you need to add timestamps at ingestion boundaries.

**Typical scenarios:**
- REST API endpoints receiving events
- Legacy systems without timestamps
- Third-party integrations
- System boundaries where you add temporal metadata
- Adapters between timestamped and non-timestamped worlds

### Implementation

```rust
use fluxion_core::{HasTimestamp, Timestamped};
use std::time::{SystemTime, UNIX_EPOCH};

// External system provides non-timestamped events
#[derive(Clone, Debug)]
struct ThirdPartyEvent {
    id: String,
    payload: Vec<u8>,
}

// Your adapter adds timestamp at ingestion
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TimestampedEvent {
    timestamp: u64,  // Added by adapter
    event: ThirdPartyEvent,
}

impl TimestampedEvent {
    fn from_external(event: ThirdPartyEvent) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self { timestamp, event }
    }
}

impl HasTimestamp for TimestampedEvent {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl Timestamped for TimestampedEvent {
    type Inner = Self;
    
    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { timestamp, ..value }
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        Self::from_external(value.event)
    }

    fn into_inner(self) -> Self::Inner {
        self
    }
}
```

### Usage

```rust
use fluxion_rx::prelude::*;

async fn handle_webhook(external_event: ThirdPartyEvent, tx: mpsc::UnboundedSender<TimestampedEvent>) {
    // Add timestamp at the system boundary
    let timestamped = TimestampedEvent::from_external(external_event);

    // Send to timestamped stream processing pipeline
    tx.send(timestamped).unwrap();
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel();

    // Process timestamped events
    let stream = FluxionStream::from_unbounded_receiver(rx)
        .map_ordered(|e| e.clone().into_inner().event);

    // Now you have stream of ThirdPartyEvent with temporal ordering
}
```

**Key points:**
- ✅ Adds timestamps at system boundaries
- ✅ Bridges non-timestamped external systems with Fluxion's temporal processing
- ✅ Timestamp assigned at ingestion time
- ✅ Original event preserved inside wrapper

## Comparison Table

| Pattern | Timestamp Source | Use Case | Domain Impact | Infrastructure |
|---------|------------------|----------|---------------|----------------|
| **Intrinsic** | Event's own timestamp | Production event streams | Timestamp is part of domain | Minimal |
| **Extrinsic** | Test harness | Unit/integration testing | Domain stays clean | `Sequenced<T>` wrapper |
| **Wrapper** | Added at boundary | Third-party integration | Adapter layer adds field | Ingestion adapter |

## Choosing the Right Pattern

### Use Intrinsic Timestamps when:
- ✅ Events naturally have timestamps (sensors, logs, messages)
- ✅ Building production event processing pipelines
- ✅ Timestamps are part of your domain model
- ✅ You want minimal abstraction overhead

### Use Extrinsic Timestamps when:
- ✅ Writing unit tests for ordered operators
- ✅ Need to simulate specific timestamp scenarios
- ✅ Testing reordering, duplicates, or gaps
- ✅ Domain objects shouldn't know about temporal sequencing

### Use Wrapper Timestamps when:
- ✅ Integrating with systems that don't provide timestamps
- ✅ Adding temporal metadata at ingestion points
- ✅ Building adapters between timestamped/non-timestamped systems
- ✅ You control the integration boundary but not the source

## Best Practices

1. **Production Code**: Use intrinsic timestamps whenever possible - it's the most natural and efficient.

2. **Testing**: Use `Sequenced<T>` from `fluxion-test-utils` to keep tests clean and flexible.

3. **Integration Boundaries**: Add timestamps as early as possible in your pipeline, ideally at the point of ingestion.

4. **Timestamp Source**:
   - Use event time (when the event occurred) for intrinsic timestamps
   - Use processing time (when you received it) for wrapper timestamps
   - Be consistent within a stream

5. **Mixed Streams**: When combining multiple streams, ensure all use compatible timestamp sources (all event time or all processing time).

## Examples

### Pattern 1: Intrinsic Timestamps - stream-aggregation

See the **[stream-aggregation](examples/stream-aggregation/)** example for a complete production implementation using intrinsic timestamps:

**What makes this example valuable:**
- **Real-world architecture**: Demonstrates how to structure a multi-stream event processing system
- **Three data sources**: Sensor readings, metrics, and system events - all with different types and intrinsic timestamps
- **Type-safe aggregation**: Shows `UnboundedReceiverExt::into_fluxion_stream()` pattern for combining heterogeneous streams
- **Graceful shutdown**: Proper cleanup and resource management with `CancellationToken`
- **Production patterns**: Error handling, logging, and event filtering

Run it: `cargo run --bin rabbitmq_aggregator`

### Pattern 3: Wrapper Timestamps - legacy-integration

See the **[legacy-integration](examples/legacy-integration/)** example for a complete implementation of the wrapper pattern:

**What makes this example valuable:**
- **Adapter pattern**: Shows how to add timestamps at system boundaries for legacy sources
- **Three legacy sources**: Database (JSON), Message Queue (XML), File Watcher (CSV) - none have intrinsic timestamps
- **Stateful aggregation**: Uses `merge_with` to build a unified repository from heterogeneous event streams
- **Real-time processing**: Demonstrates `subscribe_async` for sequential event processing with analytics
- **Lifecycle management**: Ctrl+C handling, auto-timeout, and graceful shutdown of all adapters

Run it: `cargo run --bin legacy-integration`

### Pattern 2: Extrinsic Timestamps - Test Suites

For testing examples using `Sequenced<T>`, see the test suites in `fluxion-stream/tests/` which extensively use extrinsic timestamps for controlled scenario testing.
