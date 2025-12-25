# WASM Dashboard Architecture

## Overview

The dashboard demonstrates Fluxion's time-bound operators using a clean, layered architecture with strategic use of the **share** operator. Three raw sensor streams are timestamped and **shared** (one for GUI, one for processing), then combined/filtered/summed into a merged stream that is **shared** again (one for GUI, five for time-bound operators). All 9 streams are wired to dedicated GUI windows via **subscribe**.

## Architecture Layers

### Source Layer (Phase 1)
**Object 1: `RawStreams`**
- Contains 3 independent sensor streams (no timestamps)
- **Frequency:** 1-5 Hz (200-1000ms intervals)
- **Purpose:** Generate realistic sensor data with natural timing variation
- **Type:** `struct RawStreams { sensor1, sensor2, sensor3 }`
- **Data:** Plain sensor values without temporal information
- **Value Ranges:**
  - Sensor 1: 1-9
  - Sensor 2: 10-90
  - Sensor 3: 100-900

### Processing Layer (Phase 2)

**Object 2: `SharedTimestampedStreams`**
- Augments raw streams with timestamps and shares them
- **Purpose:** Add temporal information and enable dual subscription (GUI + processing)
- **Pattern:** Wrapper pattern using WasmTimer for timestamp generation, then `.share()`
- **Type:** `struct SharedTimestampedStreams { sensor1, sensor2, sensor3 }` (each is FluxionShared)
- **Transform:** `SensorValue` → `Timestamped<SensorValue>`
- **Subscribers per stream:** 2 (1 for GUI window, 1 for combine_latest)

**Object 3: `SharedMergedStream`**
- Combines the 3 shared timestamped streams, filters, sums, and shares the result
- **Purpose:** Create a single aggregated value stream and enable multiple subscriptions
- **Operators:** `.combine_latest()` + `.filter_ordered()` (even numbers only) + `.map_ordered()` (sum) + `.share()`
- **Type:** `struct SharedMergedStream { merged: FluxionShared<Timestamped<SummedValue>> }`
- **Transform:** `(Sensor1, Sensor2, Sensor3)` → `SummedValue` (even numbers only, range: 112-998)
- **Subscribers:** 6 total (1 for GUI window, 5 for time-bound operators)

**Object 4: `TimeBoundStreams`**
- Applies 5 different time-bound operators to the shared merged stream
- **Purpose:** Demonstrate time-bound operator behavior in isolation
- **Operators:** `debounce`, `throttle`, `buffer`, `sample`, `window`
- **Type:** 5 independent FluxionStreams created from SharedMergedStream subscriptions
- **Subscribers:** 1 per stream (each wired to dedicated GUI window)

### Presentation Layer (Phase 3)

**Object 5: `DashboardUI`**
- Contains all GUI elements with update hooks
- **Purpose:** Provide structured access to DOM elements for stream subscriptions
- **Type:** `Rc<RefCell<DashboardUI>>` - shared among 9 subscribe closures
- **Hooking Points:** 9 updateable windows (3 sensors + 1 combined + 5 operators) + 2 buttons
- **Lifecycle:** Controlled by CancellationToken from start/stop buttons

**Usage Pattern:**
```rust
let ui = DashboardUI::new(&document)?;  // Returns Rc<RefCell<DashboardUI>>

// Clone Rc for each subscribe closure
let ui_clone = ui.clone();
sensor1_stream.subscribe(move |value| {
    ui_clone.borrow_mut().update_sensor1(value.inner());
}).unwrap();
```

**Layout Structure:**

**Top Section: Controls**
- Start/Stop buttons (control CancellationToken)
- Sensor frequency configuration (only enabled when stopped)
  - Sensor 1 frequency (1-5 Hz)
  - Sensor 2 frequency (1-5 Hz)
  - Sensor 3 frequency (1-5 Hz)

**Top Section: Individual Sensor Windows**
- 3 windows displaying real-time data from each sensor
  - Window: Sensor 1 (values 1-9)
  - Window: Sensor 2 (values 10-90)
  - Window: Sensor 3 (values 100-900)

**Middle Section: Combined Stream Window**
- 1 window displaying real-time combined data from all sensors
  - Shows result of combine_latest + filter (even numbers) + sum operation
  - Values displayed: 112-998 (only even sums)

**Bottom Section: Time-Bound Operator Windows**
- 5 windows displaying results from each time-bound operator
  - Window 1: Debounce operator
  - Window 2: Throttle operator
  - Window 3: Buffer operator (displays Vec<T>)
  - Window 4: Sample operator
  - Window 5: Window operator (displays Window<T>)

**Subscription Wiring:**
- Each of the 9 windows receives updates via `.subscribe()` with closure
- Closure receives `DashboardUI` reference and updates corresponding GUI element
- Pattern: `stream.subscribe(|value| ui.update_window_n(value))`
- Total subscriptions: 9 (3 sensors + 1 combined + 5 operators)

## File Structure

### `src/source/raw_streams.rs` (Phase 1)
```rust
pub struct RawStreams {
    pub sensor1: UnboundedReceiver<SensorValue>,
    pub sensor2: UnboundedReceiver<SensorValue>,
    pub sensor3: UnboundedReceiver<SensorValue>,
}

impl RawStreams {
    pub fn new() -> Self {
        // Create 3 sensors with 1-5 Hz frequency
        // Natural timing: 200-1000ms intervals
        // Sensor 1: values 1-9
        // Sensor 2: values 10-90
        // Sensor 3: values 100-900
        // Returns raw values without timestamps
    }
}
```

### `src/processing/timestamped.rs` (Phase 2)
```rust
pub struct SharedTimestampedStreams {
    pub sensor1: FluxionShared<Timestamped<SensorValue>>,
    pub sensor2: FluxionShared<Timestamped<SensorValue>>,
    pub sensor3: FluxionShared<Timestamped<SensorValue>>,
}

impl SharedTimestampedStreams {
    pub fn new(raw: RawStreams) -> Self {
        // Add timestamps using WasmTimer, then share each stream
        // Each shared stream has 2 subscribers: 1 for GUI, 1 for combine_latest
        let sensor1 = raw.sensor1.into_fluxion_stream()
            .map_ordered(|v| Timestamped::new(v))  // WasmTimer wrapper
            .share();
        let sensor2 = raw.sensor2.into_fluxion_stream()
            .map_ordered(|v| Timestamped::new(v))
            .share();
        let sensor3 = raw.sensor3.into_fluxion_stream()
            .map_ordered(|v| Timestamped::new(v))
            .share();

        Self { sensor1, sensor2, sensor3 }
    }
}
```

### `src/processing/merged.rs` (Phase 2)
```rust
pub struct SharedMergedStream {
    merged: FluxionShared<Timestamped<SummedValue>>,
}

impl SharedMergedStream {
    pub fn new(timestamped: SharedTimestampedStreams) -> Self {
        let merged = FluxionStream::new(timestamped.sensor1.subscribe().unwrap())
            .combine_latest(vec![
                FluxionStream::new(timestamped.sensor2.subscribe().unwrap()),
                FluxionStream::new(timestamped.sensor3.subscribe().unwrap())
            ])
            .filter_ordered(|combined| {
                // Filter: only even sums
                let sum = combined.0 + combined.1 + combined.2;
                sum % 2 == 0
            })
            .map_ordered(|(s1, s2, s3)| {
                // Sum all sensor values: 112-998 (even only)
                s1 + s2 + s3
            })
            .share();  // Share for 6 subscribers: 1 GUI + 5 operators
        Self { merged }
    }

    pub fn subscribe_for_gui(&self)
        -> Result<FluxionStream<Timestamped<SummedValue>>, SubjectError>
    {
        Ok(FluxionStream::new(self.merged.subscribe()?))
    }
}
```

### `src/processing/time_bound.rs` (Phase 2)
```rust
pub struct TimeBoundStreams {
    debounce: FluxionStream<Timestamped<SummedValue>>,
    throttle: FluxionStream<Timestamped<SummedValue>>,
    buffer: FluxionStream<Vec<Timestamped<SummedValue>>>,
    sample: FluxionStream<Timestamped<SummedValue>>,
    window: FluxionStream<Window<Timestamped<SummedValue>>>,
}

impl TimeBoundStreams {
    pub fn new(merged: &SharedMergedStream, duration_ms: u64) -> Self {
        let debounce = FluxionStream::new(merged.merged.subscribe().unwrap())
            .debounce(Duration::from_millis(duration_ms));
        let throttle = FluxionStream::new(merged.merged.subscribe().unwrap())
            .throttle(Duration::from_millis(duration_ms));
        let buffer = FluxionStream::new(merged.merged.subscribe().unwrap())
            .buffer(Duration::from_millis(duration_ms));
        let sample = FluxionStream::new(merged.merged.subscribe().unwrap())
            .sample(Duration::from_millis(duration_ms));
        let window = FluxionStream::new(merged.merged.subscribe().unwrap())
            .window(Duration::from_millis(duration_ms));

        Self { debounce, throttle, buffer, sample, window }
    }
}
```

### `src/gui/dashboard_ui.rs` (Phase 3)
```rust
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{Document, HtmlElement, HtmlButtonElement};

pub struct DashboardUI {
    // 9 windows + 2 buttons = 11 GUI elements
    sensor1_window: HtmlElement,
    sensor2_window: HtmlElement,
    sensor3_window: HtmlElement,
    combined_window: HtmlElement,
    debounce_window: HtmlElement,
    throttle_window: HtmlElement,
    buffer_window: HtmlElement,
    sample_window: HtmlElement,
    window_window: HtmlElement,
    start_button: HtmlButtonElement,
    stop_button: HtmlButtonElement,
}

impl DashboardUI {
    pub fn new(document: &Document) -> Result<Rc<RefCell<Self>>, JsValue> {
        // Initialize all 11 GUI elements
        // Returns Rc<RefCell<T>> for sharing among 9 subscribes
    }

    // 11 hooking points
    pub fn update_sensor1(&mut self, value: u32) { /* ... */ }
    pub fn update_sensor2(&mut self, value: u32) { /* ... */ }
    pub fn update_sensor3(&mut self, value: u32) { /* ... */ }
    pub fn update_combined(&mut self, value: u32) { /* ... */ }
    pub fn update_debounce(&mut self, value: u32) { /* ... */ }
    pub fn update_throttle(&mut self, value: u32) { /* ... */ }
    pub fn update_buffer(&mut self, values: &[u32]) { /* ... */ }
    pub fn update_sample(&mut self, value: u32) { /* ... */ }
    pub fn update_window(&mut self, count: usize) { /* ... */ }
    pub fn enable_start(&mut self) { /* ... */ }
    pub fn enable_stop(&mut self) { /* ... */ }
}
```

### `src/lib.rs` - Entry Point
```rust
use tokio_util::sync::CancellationToken;

#[wasm_bindgen]
pub async fn start_dashboard() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // 1. Create cancellation token for lifecycle management
    let cancellation_token = CancellationToken::new();

    // 2. Create GUI with 11 hooking points (9 windows + 2 buttons)
    let ui = DashboardUI::new(&document)?;

    // 3. Create source layer (frequencies not configured yet)
    let source = SourceLayer::new();

    // 4. Create processing layer (takes source + cancellation token)
    let processing = ProcessingLayer::new(source, cancellation_token.clone());

    // 5. Create application (orchestrates wiring)
    let app = Application::new(ui, processing, cancellation_token);

    // 6. Wire up start/stop button events
    // Start button: triggers wiring → feeds all 9 windows asynchronously
    // Stop button: triggers cancellation token
    app.setup_controls().await?;

    Ok(())
}
```

### `src/app.rs` - Application Orchestrator
```rust
pub struct Application {
    ui: Rc<RefCell<DashboardUI>>,
    processing: ProcessingLayer,
    cancellation_token: CancellationToken,
}

impl Application {
    pub fn new(
        ui: Rc<RefCell<DashboardUI>>,
        processing: ProcessingLayer,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self { ui, processing, cancellation_token }
    }

    pub async fn setup_controls(&self) -> Result<(), JsValue> {
        // Wire start button → triggers stream wiring
        let ui_clone = self.ui.clone();
        let processing_clone = self.processing.clone();
        self.setup_start_button(ui_clone, processing_clone)?;

        // Wire stop button → triggers cancellation
        let token_clone = self.cancellation_token.clone();
        self.setup_stop_button(token_clone)?;

        Ok(())
    }

    fn setup_start_button(
        &self,
        ui: Rc<RefCell<DashboardUI>>,
        processing: ProcessingLayer,
    ) -> Result<(), JsValue> {
        // Clicking start button triggers async stream wiring
        // This wires all 9 subscriptions to GUI windows
        // Streams start feeding data asynchronously
    }

    fn setup_stop_button(&self, token: CancellationToken) -> Result<(), JsValue> {
        // Clicking stop button triggers cancellation
        // All streams stop, resources cleaned up
    }

    async fn wire_streams(
        ui: Rc<RefCell<DashboardUI>>,
        processing: ProcessingLayer,
    ) {
        // Wire 3 sensor windows (SharedTimestampedStreams)
        // Wire 1 combined window (SharedMergedStream)
        // Wire 5 time-bound operator windows (TimeBoundStreams)
        // Total: 9 subscriptions feeding GUI asynchronously
    }
}
```

### Module Structure
```
src/
├── source/
│   ├── mod.rs
│   └── raw_streams.rs         # Phase 1: RawStreams
├── processing/
│   ├── mod.rs
│   ├── timestamped.rs         # Phase 2: SharedTimestampedStreams
│   ├── merged.rs              # Phase 2: SharedMergedStream
│   └── time_bound.rs          # Phase 2: TimeBoundStreams
├── gui/
│   ├── mod.rs
│   └── dashboard_ui.rs        # Phase 3: DashboardUI (Rc<RefCell<T>>)
├── lib.rs                     # Entry point: creates token, GUI, layers, app
└── app.rs                     # Application: orchestrates wiring & controls
```

### Module Structure
```
src/
├── source/
│   ├── mod.rs
│   └── raw_streams.rs         # Phase 1: RawStreams
├── processing/
│   ├── mod.rs
│   ├── timestamped.rs         # Phase 2: SharedTimestampedStreams
│   ├── merged.rs              # Phase 2: SharedMergedStream
│   └── time_bound.rs          # Phase 2: TimeBoundStreams
├── gui/
│   ├── mod.rs
│   └── dashboard_ui.rs        # Phase 3: DashboardUI
├── lib.rs
└── app.rs                     # Main wiring
## Data Flow

```
SOURCE LAYER (Raw values, no timestamps)
┌─────────────────┐
│ Sensor 1 (1-9)  │──> Timestamp + Share ──┬──> GUI Window 1
└─────────────────┘                         └──> combine_latest
                                                      │
┌─────────────────┐                                   │
│ Sensor 2 (10-90)│──> Timestamp + Share ──┬──> GUI Window 2
└─────────────────┘                         └──>      │
                                                      │
┌─────────────────┐                                   │
│ Sensor 3(100-900│──> Timestamp + Share ──┬──> GUI Window 3
└─────────────────┘                         └──>      │
                                                      ▼
                    PROCESSING: CombineLatest + Filter(even) + Sum
                              (Produces even sums: 112-998)
                                          │
                                          ▼
                                 Share (6 subscribers)
                                          │
      ┌───────────────────────────────┼───────────────────────────────┐
      │                               │                               │
      ▼                               ▼                               ▼
 GUI Window 4                    Debounce ──> GUI Window 5     Sample ──> GUI Window 8
(Combined Stream)                Throttle ──> GUI Window 6     Window ──> GUI Window 9
                                  Buffer ──> GUI Window 7

Total: 4 share operators (3 timestamped + 1 merged)
Total: 9 GUI subscriptions (3 sensors + 1 combined + 5 operators)
```

## Key Design Decisions

### 1. **Source Frequency: 1-5 Hz**
- **Why:** Human perception sweet spot - slow enough to track, fast enough to see behavior
- **How:** Random intervals between 200-1000ms per sensor
- **Benefit:** Makes operator differences visually obvious

### 2. **Strategic Sharing Pattern**
- **Why:** Enable dual observation (GUI + processing) at each transformation stage
- **Benefit:** Users see raw sensors, combined result, AND operator outputs simultaneously
- **Pattern:** Share at timestamped level (3 operators) + merged level (1 operator) = 4 total shares
- **Subscriptions:** 9 total (3 + 1 + 5) - each directly wired to GUI via `.subscribe()`

### 3. **Five Object Model**
- **RawStreams:** Single responsibility - data generation with configurable frequency
- **SharedTimestampedStreams:** Single responsibility - timestamp instrumentation + dual subscription
- **SharedMergedStream:** Single responsibility - stream combination, filtering (even), summing + multi-subscription
- **TimeBoundStreams:** Single responsibility - apply 5 time-bound operators
- **DashboardUI:** Single responsibility - GUI element management with 9 update hooks
- **Benefit:** Clean separation, easy to test, obvious data flow, CancellationToken lifecycle

### 4. **Layered Presentation**
- **Source Layer:** What data exists
- **Processing Layer:** How data flows and multiplies
- **Presentation Layer:** What users see (future phase)
- **Benefit:** Clear mental model, easy to explain

## Benefits

### Educational Value
- **Visual comparison:** Side-by-side operator behavior
- **Multicast demonstration:** One source, five consumers
- **Real-time:** See operators react to live data

### Code Quality
- **Testable:** Each layer independently mockable
- **Maintainable:** Clear responsibilities
- **Extensible:** Easy to add more operators or sources

### Performance
- **Efficient:** Source runs once, results broadcast
- **Browser-friendly:** 1-5 Hz won't overwhelm rendering
- **Scalable:** Can add more subscribers without rerunning source

## Implementation Phases

### Phase 1: Core Architecture ✅
- Implement RawStreams, MergedStream, SharedStreams
- Wire up merge and share operators
- Verify data flows correctly

### Phase 2: Operator Integration (Current)
- Implement 5 time-bound operators
- Create subscription methods
- Test each operator chain independently

### Phase 3: Presentation (Next)
- Wire SharedStreams to DOM
- Create 5 visualization windows
- Add start/stop controls

### Phase 4: Polish
- Add operator configuration (duration controls)
- Visual styling and layout
- Performance monitoring

## Testing Strategy

```rust
#[test]
fn test_raw_streams() {
    let raw = RawStreams::new();
    // Verify 3 streams created
    // Verify timing ranges (1-5 Hz)
}

#[test]
fn test_merged_stream() {
    let raw = mock_raw_streams();
    let merged = MergedStream::new(raw);
    // Verify merge behavior
    // Verify ordering preserved
}

#[test]
fn test_shared_streams() {
    let shared = mock_shared_streams();
    let sub1 = shared.subscribe_debounce(100).unwrap();
    let sub2 = shared.subscribe_throttle(100).unwrap();
    // Verify independent subscriptions
    // Verify operator behavior
}
```
