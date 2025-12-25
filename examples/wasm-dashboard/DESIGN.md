# WASM Dashboard - Implementation Design

## Phase 1: Source Layer - Raw Sensor Streams

### Overview
Phase 1 implements the source layer that generates raw sensor data without timestamps. Three independent sensors emit values at random frequencies (1-5 Hz) within specified ranges.

### Key Design Decisions

#### 1. Task Management
- **Pattern**: `FluxionTask::spawn()` with cancellation token
- **Lifecycle**: Task stored in struct (`_task` field), auto-cancelled on drop
- **Cancellation**: Manual via `task.cancel()` or automatic on struct drop
- **Runtime**: WASM runtime (wasm_bindgen_futures)

```rust
let task = FluxionTask::spawn(|cancel| async move {
    loop {
        if cancel.is_cancelled() {
            break;
        }
        // Generate, send, delay
    }
});
```

#### 2. Channel Communication
- **Type**: `async_channel::unbounded()` - available for all runtimes
- **Reason**: `Receiver` is `Clone`, enabling multiple subscriptions
- **Pattern**: Sender owned by task, Receiver exposed by struct
- **Dependency**: Already in workspace, add to wasm-dashboard Cargo.toml

#### 3. Random Data Generation
- **Random values**: `js_sys::Math::random()` (WASM compatible)
- **Random delays**: `gloo_timers::future::TimeoutFuture`
- **Period calculation**: `base_ms + (random() * range_ms)` for 1-5 Hz (200-1000ms)
- **Value calculation**: `min + (random() * (max - min))`

#### 4. Architecture

**SensorStream** - Individual sensor with task and receiver
```rust
pub struct SensorStream {
    receiver: async_channel::Receiver<u32>,
    _task: FluxionTask,
}

impl SensorStream {
    pub fn new(period_range_ms: (u64, u64), value_range: (u32, u32)) -> Self {
        let (sender, receiver) = async_channel::unbounded();
        
        let task = FluxionTask::spawn(|cancel| async move {
            loop {
                if cancel.is_cancelled() {
                    break;
                }
                
                // Generate random value in range
                let value = generate_random_value(value_range);
                
                // Send value (ignore errors if receiver dropped)
                let _ = sender.send(value).await;
                
                // Wait random period (1-5 Hz)
                let delay_ms = generate_random_delay(period_range_ms);
                TimeoutFuture::new(delay_ms).await;
            }
        });
        
        Self {
            receiver,
            _task: task,
        }
    }
    
    /// Returns a cloneable receiver for this sensor's data
    pub fn receiver(&self) -> async_channel::Receiver<u32> {
        self.receiver.clone()
    }
}
```

**RawStreams** - Container for three sensor instances
```rust
pub struct RawStreams {
    pub sensor1: SensorStream,
    pub sensor2: SensorStream,
    pub sensor3: SensorStream,
}

impl RawStreams {
    pub fn new() -> Self {
        Self {
            sensor1: SensorStream::new((200, 1000), (1, 9)),
            sensor2: SensorStream::new((200, 1000), (10, 90)),
            sensor3: SensorStream::new((200, 1000), (100, 900)),
        }
    }
}
```

#### 5. Sensor Specifications
- **Sensor 1**: Values 1-9, Frequency 1-5 Hz (200-1000ms intervals)
- **Sensor 2**: Values 10-90, Frequency 1-5 Hz (200-1000ms intervals)
- **Sensor 3**: Values 100-900, Frequency 1-5 Hz (200-1000ms intervals)

#### 6. Task Lifecycle
- **Creation**: Tasks spawned in `SensorStream::new()`
- **Running**: Continuous loop generating values until cancelled
- **Cancellation**: 
  - Automatic when `SensorStream` is dropped
  - Manual via stored `FluxionTask` reference if needed
- **Cleanup**: Channel automatically closed when sender dropped

#### 7. File Structure
```
src/
  source/
    mod.rs          - Module exports
    sensor.rs       - SensorStream implementation
    raw_streams.rs  - RawStreams container
```

### Dependencies Required
```toml
[dependencies]
async-channel = { workspace = true }
gloo-timers = { version = "0.3", features = ["futures"] }
js-sys = "0.3"
```

### Testing Strategy
- Verify task spawns and generates values
- Confirm random delays fall within 200-1000ms range
- Validate random values fall within specified ranges
- Test receiver cloning enables multiple subscriptions
- Confirm task cancellation on drop

### Next Phase
Phase 2 will wrap these receivers in `FluxionStream`, add timestamps using `WasmTimer`, and apply the `share()` operator for dual subscription (GUI + processing pipeline).
