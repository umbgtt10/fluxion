# Integration Guide

This guide describes the three fundamental patterns for integrating events with Fluxion's ordered stream processing.

## Overview

Fluxion processes streams of ordered events. The `Ordered` trait is the core abstraction that defines how events are sequenced. There are three main patterns for providing ordering information:

1. **Intrinsic Ordering** - Events carry their own timestamps (production)
2. **Extrinsic Ordering** - Test infrastructure controls order (testing)
3. **Wrapper Ordering** - Adapter adds timestamps at boundaries (integration)

## Pattern 1: Intrinsic Ordering (Production)

**When to use:** Your domain events already have timestamps, sequence numbers, or other inherent ordering information.

**Typical scenarios:**
- Event sourcing systems
- Message queue consumers (Kafka, RabbitMQ)
- IoT sensor data
- Log aggregation
- Time-series data

### Implementation

```rust
use fluxion::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SensorReading {
    timestamp: u64,  // Events carry their own temporal order
    sensor_id: String,
    temperature: f64,
}

impl Ordered for SensorReading {
    type Inner = Self;
    
    fn order(&self) -> u64 {
        self.timestamp  // Use the event's intrinsic timestamp
    }
    
    fn get(&self) -> &Self::Inner {
        self
    }
    
    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value  // Order is immutable, already part of the event
    }
}
```

### Usage

```rust
use fluxion::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();
    
    // Events are already ordered by their timestamp
    let stream = FluxionStream::from_unbounded_receiver(rx)
        .filter_ordered(|reading| reading.get().temperature > 25.0);
    
    // Stream processing respects the intrinsic timestamp ordering
}
```

**Key points:**
- ✅ Events reflect real-world temporal order
- ✅ No additional infrastructure needed
- ✅ Uses `auto_ordered()` to enable ordered stream operations
- ✅ Ordering is a property of the domain model

## Pattern 2: Extrinsic Ordering (Testing)

**When to use:** You need fine-grained control over event ordering, typically in tests.

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

// Your domain type doesn't need ordering information
#[derive(Clone, Debug)]
struct Event {
    data: String,
}

// Test infrastructure adds sequence numbers
let event1 = Sequenced::with_sequence(Event { data: "first".into() }, 100);
let event2 = Sequenced::with_sequence(Event { data: "second".into() }, 200);
```

### Usage

```rust
use fluxion::prelude::*;
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
        .filter_ordered(|e| e.get().data.starts_with('f'));

    // Push events in arbitrary order
    tx.send(Sequenced::with_sequence(Event { data: "third".into() }, 300)).unwrap();
    tx.send(Sequenced::with_sequence(Event { data: "first".into() }, 100)).unwrap();
    tx.send(Sequenced::with_sequence(Event { data: "second".into() }, 200)).unwrap();
    drop(tx);
    
    // Stream will reorder by sequence: 100, 200, 300
    // Then filter to only: 100 ("first")
    let result = stream.next().await.unwrap();
    assert_eq!(result.get().data, "first");
}
```

**Key points:**
- ✅ Full control over event sequencing
- ✅ Domain objects stay clean (no test-specific fields)
- ✅ Perfect for testing edge cases (duplicates, gaps, reordering)
- ✅ Uses `Sequenced<T>` wrapper from `fluxion-test-utils`

## Pattern 3: Wrapper Ordering (Integration)

**When to use:** Integrating with external systems that don't provide timestamps, or when you need to add ordering at ingestion boundaries.

**Typical scenarios:**
- REST API endpoints receiving events
- Legacy systems without timestamps
- Third-party integrations
- System boundaries where you add temporal metadata
- Adapters between ordered and unordered worlds

### Implementation

```rust
use fluxion::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// External system provides unordered events
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

impl Ordered for TimestampedEvent {
    type Inner = Self;
    
    fn order(&self) -> u64 {
        self.timestamp
    }
    
    fn get(&self) -> &Self::Inner {
        self
    }
    
    fn with_order(value: Self::Inner, _order: u64) -> Self {
        value
    }
}
```

### Usage

```rust
use fluxion::prelude::*;

async fn handle_webhook(external_event: ThirdPartyEvent, tx: mpsc::UnboundedSender<TimestampedEvent>) {
    // Add timestamp at the system boundary
    let timestamped = TimestampedEvent::from_external(external_event);
    
    // Send to ordered stream processing pipeline
    tx.send(timestamped).unwrap();
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel();
    
    // Process timestamped events
    let stream = FluxionStream::from_unbounded_receiver(rx)
        .map_ordered(|e| e.get().event.clone());
    
    // Now you have ordered stream of ThirdPartyEvent
}
```

**Key points:**
- ✅ Adds ordering at system boundaries
- ✅ Bridges unordered external systems with Fluxion's ordered processing
- ✅ Timestamp assigned at ingestion time
- ✅ Original event preserved inside wrapper

## Comparison Table

| Pattern | Order Source | Use Case | Domain Impact | Infrastructure |
|---------|--------------|----------|---------------|----------------|
| **Intrinsic** | Event's own timestamp | Production event streams | Timestamp is part of domain | Minimal |
| **Extrinsic** | Test harness | Unit/integration testing | Domain stays clean | `Sequenced<T>` wrapper |
| **Wrapper** | Added at boundary | Third-party integration | Adapter layer adds field | Ingestion adapter |

## Choosing the Right Pattern

### Use Intrinsic Ordering when:
- ✅ Events naturally have timestamps (sensors, logs, messages)
- ✅ Building production event processing pipelines
- ✅ Timestamps are part of your domain model
- ✅ You want minimal abstraction overhead

### Use Extrinsic Ordering when:
- ✅ Writing unit tests for ordered operators
- ✅ Need to simulate specific ordering scenarios
- ✅ Testing reordering, duplicates, or gaps
- ✅ Domain objects shouldn't know about sequencing

### Use Wrapper Ordering when:
- ✅ Integrating with systems that don't provide timestamps
- ✅ Adding temporal metadata at ingestion points
- ✅ Building adapters between ordered/unordered systems
- ✅ You control the integration boundary but not the source

## Best Practices

1. **Production Code**: Use intrinsic ordering whenever possible - it's the most natural and efficient.

2. **Testing**: Use `Sequenced<T>` from `fluxion-test-utils` to keep tests clean and flexible.

3. **Integration Boundaries**: Add timestamps as early as possible in your pipeline, ideally at the point of ingestion.

4. **Timestamp Source**: 
   - Use event time (when the event occurred) for intrinsic ordering
   - Use processing time (when you received it) for wrapper ordering
   - Be consistent within a stream

5. **Mixed Streams**: When combining multiple streams, ensure all use compatible timestamp sources (all event time or all processing time).

## Examples

See the `examples/stream-aggregation` directory for a **complete production example** using intrinsic ordering with sensor readings, metrics, and system events.

**What makes this example valuable:**
- **Real-world architecture**: Demonstrates how to structure a multi-stream event processing system
- **Three data sources**: Sensor readings, metrics, and system events - all with different types
- **Type-safe aggregation**: Shows `UnboundedReceiverExt::into_fluxion_stream()` pattern for combining heterogeneous streams
- **Graceful shutdown**: Proper cleanup and resource management with `CancellationToken`
- **Production patterns**: Error handling, logging, and event filtering

Run it: `cargo run --example stream-aggregation`

For testing examples, see the test suites in `fluxion-stream/tests/` which extensively use `Sequenced<T>` for controlled ordering scenarios.
