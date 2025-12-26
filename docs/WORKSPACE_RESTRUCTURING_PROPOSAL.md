# Fluxion Workspace Restructuring Proposal

**Date:** December 26, 2025  
**Status:** Design Proposal  
**Purpose:** Address runtime complexity and prepare for no-std/Embassy expansion

---

## Executive Summary

This document proposes a **hybrid runtime-isolated architecture** that maintains a single workspace while separating runtime-specific concerns into dedicated crates. This approach:

- ✅ Eliminates conditional compilation sprawl (50+ `#[cfg]` attributes)
- ✅ Improves developer experience for runtime-specific applications (WASM, Embassy)
- ✅ Reduces test duplication (113 → ~30 test files)
- ✅ Maintains single workspace for coordination and versioning
- ✅ Prepares for no-std/Embassy embedded example

---

## Current State Analysis

### Workspace Structure

```
fluxion/
├── Cargo.toml                    # Workspace root
├── fluxion/                      # Facade crate (re-exports)
├── fluxion-core/                 # Core traits, runtime-agnostic
├── fluxion-stream/               # Operators, runtime-agnostic
├── fluxion-ordered-merge/        # Merge algorithms
├── fluxion-exec/                 # Execution layer (CFG-heavy)
├── fluxion-stream-time/          # Time operators (CFG-heavy)
├── fluxion-test-utils/           # Test utilities
└── examples/
    ├── stream-aggregation/       # Tokio example
    ├── legacy-integration/       # Tokio example
    └── wasm-dashboard/           # WASM example
```

### Feature Flag Matrix (Current)

| Crate | Features | Target Specificity |
|-------|----------|-------------------|
| `fluxion-core` | `std`, `alloc`, `runtime-tokio`, `runtime-async-std`, `runtime-smol`, `tracing` | Low |
| `fluxion-stream` | `std`, `alloc`, `runtime-tokio`, `runtime-async-std`, `runtime-smol`, `tracing` | Low |
| `fluxion-exec` | `std`, `alloc`, `runtime-tokio`, `runtime-async-std`, `runtime-smol` | **Medium** |
| `fluxion-stream-time` | `std`, `alloc`, `runtime-tokio`, `runtime-async-std`, `runtime-smol`, `runtime-wasm`, `runtime-embassy` | **HIGH** |

### Conditional Compilation Footprint

**Scope of Problem:**
- **50+ `#[cfg]` attributes** across crates (target_arch, features)
- **8 target-specific dependency sections** in Cargo.toml files
- **113 duplicated test files** across tokio/async-std/smol/wasm/embassy
- **Dual trait implementations** (e.g., `?Send` vs `Send+Sync`)
- **CI complexity**: 3 OS × 5 runtimes × 2 targets (native/wasm)

**Most Complex Files:**
1. `fluxion-exec/src/subscribe.rs` - Dual trait with cfg(wasm32)
2. `fluxion-stream-time/src/*` - All time operators have 5 runtime branches
3. `fluxion-stream-time/tests/*` - 113 test files, 80% duplication
4. `fluxion-core/src/fluxion_task.rs` - 4 spawn implementations

---

## Proposed Architecture

### New Workspace Structure

```
fluxion/
├── Cargo.toml                           # Workspace root
│
├── Core Layer (Runtime-Agnostic, no-std compatible)
│   ├── fluxion-core/                    # Traits, types, no runtime deps
│   ├── fluxion-stream/                  # Operators (map, filter, scan, etc.)
│   └── fluxion-ordered-merge/           # Merge algorithms
│
├── Runtime Abstraction Layer (NEW)
│   └── fluxion-runtime/                 # NEW: Runtime trait abstraction
│       ├── src/
│       │   ├── lib.rs                   # Runtime trait definition
│       │   ├── timer.rs                 # Timer trait
│       │   └── executor.rs              # Spawn trait
│       └── Cargo.toml                   # No runtime deps, pure traits
│
├── Shared Implementation Layer (NEW)
│   ├── fluxion-exec-core/               # NEW: Shared subscribe/spawn impl
│   │   ├── src/
│   │   │   ├── subscribe_impl.rs        # Generic subscribe logic
│   │   │   └── spawn_impl.rs            # Generic spawn logic
│   │   └── Cargo.toml                   # Depends on fluxion-runtime
│   │
│   └── fluxion-time-core/               # NEW: Shared time operator impl
│       ├── src/
│       │   ├── delay_impl.rs            # Generic delay logic
│       │   ├── throttle_impl.rs         # Generic throttle logic
│       │   └── debounce_impl.rs         # Generic debounce logic
│       └── Cargo.toml                   # Depends on fluxion-runtime
│
├── Runtime-Specific Crates (Thin Facades)
│   ├── fluxion-tokio/                   # NEW: Tokio runtime
│   │   ├── src/
│   │   │   ├── lib.rs                   # Re-exports + tokio integration
│   │   │   ├── executor.rs              # tokio::spawn wrapper
│   │   │   ├── timer.rs                 # TokioTimer impl
│   │   │   └── subscribe.rs             # SubscribeExt with Send bounds
│   │   └── Cargo.toml                   # deps: tokio, fluxion-exec-core
│   │
│   ├── fluxion-wasm/                    # NEW: WASM runtime (PRIORITY)
│   │   ├── src/
│   │   │   ├── lib.rs                   # Re-exports for WASM
│   │   │   ├── executor.rs              # wasm_bindgen_futures::spawn_local
│   │   │   ├── timer.rs                 # WasmTimer (gloo-timers)
│   │   │   └── subscribe.rs             # SubscribeExt with ?Send
│   │   ├── tests/
│   │   │   └── integration_tests.rs     # WASM-specific tests
│   │   └── Cargo.toml                   # deps: wasm-bindgen, gloo-timers
│   │
│   └── fluxion-embassy/                 # NEW: Embassy runtime (PRIORITY)
│       ├── src/
│       │   ├── lib.rs                   # Re-exports for embassy
│       │   ├── executor.rs              # embassy_executor::Spawner
│       │   ├── timer.rs                 # embassy_time::Timer
│       │   └── subscribe.rs             # SubscribeExt (embassy compat)
│       ├── tests/
│       │   └── integration_tests.rs     # Embassy-specific tests
│       └── Cargo.toml                   # deps: embassy-executor, embassy-time
│
├── Legacy Runtime Crates (Optional, for compatibility)
│   ├── fluxion-async-std/               # async-std runtime (deprecated)
│   └── fluxion-smol/                    # smol runtime
│
├── Facade & Utils
│   ├── fluxion/                         # Main facade (feature-gated re-exports)
│   └── fluxion-test-utils/              # Shared test utilities
│
└── Examples (Runtime-Specific)
    ├── examples/tokio-stream-aggregation/    # Renamed from stream-aggregation
    ├── examples/tokio-legacy-integration/    # Renamed from legacy-integration
    ├── examples/wasm-dashboard/              # Uses fluxion-wasm
    └── examples/embassy-sensor-hub/          # NEW: Embassy no-std example
        ├── Cargo.toml
        ├── src/
        │   ├── main.rs                       # Embassy executor setup
        │   ├── sensor_reader.rs              # Read from hardware sensors
        │   ├── stream_processor.rs           # Fluxion stream processing
        │   └── actuator_control.rs           # Control outputs
        ├── memory.x                          # Linker script (target-specific)
        └── README.md                         # Hardware setup, flashing
```

### Feature Flag Reduction

**Before (per crate):**
```toml
[features]
default = ["std", "runtime-tokio"]
std = ["..."]
alloc = ["..."]
runtime-tokio = ["..."]
runtime-async-std = ["..."]
runtime-smol = ["..."]
runtime-wasm = ["..."]
runtime-embassy = ["..."]
```

**After (runtime-specific crates have NO features):**
```toml
# fluxion-wasm/Cargo.toml
[dependencies]
fluxion-core = { workspace = true, default-features = false, features = ["alloc"] }
fluxion-exec-core = { workspace = true }
fluxion-time-core = { workspace = true }
wasm-bindgen-futures = "0.4"
gloo-timers = { version = "0.3", features = ["futures"] }

# No [features] section needed!
```

---

## Migration Strategy

### Phase 1: Extract Shared Implementations (Week 1-2)

**Goal:** Zero duplication of subscribe/time logic

1. **Create `fluxion-runtime` crate**
   ```rust
   // fluxion-runtime/src/lib.rs
   pub trait Runtime {
       type Timer: Timer;
       type Executor: Executor;
   }
   
   pub trait Timer {
       type Instant: Ord + Clone;
       fn now(&self) -> Self::Instant;
       fn sleep(&self, duration: Duration) -> impl Future<Output = ()>;
   }
   
   pub trait Executor {
       fn spawn<F>(&self, future: F) where F: Future<Output = ()>;
   }
   ```

2. **Create `fluxion-exec-core` crate**
   - Move `subscribe_impl()` from `fluxion-exec/src/subscribe.rs`
   - Make it generic over `Runtime` trait
   - No cfg attributes needed

3. **Create `fluxion-time-core` crate**
   - Extract delay/throttle/debounce/sample/timeout logic
   - Generic over `Runtime::Timer` trait
   - Eliminates 50+ cfg branches

**Testing:** All existing tests should pass with no behavior changes.

### Phase 2: Create `fluxion-wasm` Crate (Week 3)

**Goal:** Improve wasm-dashboard development experience

1. **Create `fluxion-wasm` crate**
   - Implement `Runtime` trait for WASM
   - Provide `SubscribeExt` with `#[async_trait(?Send)]`
   - Re-export all WASM-compatible operators

2. **Update `wasm-dashboard` dependencies**
   ```toml
   # Before:
   fluxion-core = { path = "../../fluxion-core", default-features = false, features = ["alloc"] }
   fluxion-exec = { path = "../../fluxion-exec", default-features = false }
   fluxion-stream = { path = "../../fluxion-stream", default-features = false, features = ["alloc"] }
   fluxion-stream-time = { path = "../../fluxion-stream-time", default-features = false, features = ["runtime-wasm"] }
   
   # After:
   fluxion-wasm = { path = "../../fluxion-wasm" }  # Done!
   ```

3. **Update CI** - Remove `--exclude wasm-dashboard` hack

**Validation:** wasm-dashboard builds and runs identically.

### Phase 3: Create `fluxion-embassy` Crate (Week 4-5)

**Goal:** Enable no-std/Embassy development

1. **Create `fluxion-embassy` crate**
   ```rust
   // fluxion-embassy/src/lib.rs
   #![no_std]
   
   pub use fluxion_core::*;
   pub use fluxion_stream::*;
   pub use fluxion_exec_core::SubscribeExt;
   
   pub mod time {
       pub use fluxion_time_core::*;
   }
   
   pub mod executor {
       // Embassy-specific spawn
   }
   ```

2. **Create `examples/embassy-sensor-hub`**
   ```
   examples/embassy-sensor-hub/
   ├── Cargo.toml                    # Profile for size optimization
   ├── .cargo/config.toml            # Target configuration
   ├── memory.x                      # Linker script (STM32F4)
   ├── build.rs                      # Build script
   ├── src/
   │   ├── main.rs                   # Embassy executor + tasks
   │   ├── sensor_streams.rs         # ADC/I2C sensor streams
   │   ├── processing.rs             # Stream operators (filter, map, etc.)
   │   └── actuator_sink.rs          # PWM/GPIO control
   └── README.md                     # Hardware: STM32F4 Discovery board
   ```

   **Target:** STM32F4 Discovery Board (widely available, $15, USB programming)

3. **Hardware Integration Example**
   ```rust
   // examples/embassy-sensor-hub/src/main.rs
   #![no_std]
   #![no_main]
   
   use embassy_executor::Spawner;
   use embassy_time::Duration;
   use fluxion_embassy::prelude::*;
   
   #[embassy_executor::main]
   async fn main(spawner: Spawner) {
       // Initialize hardware
       let p = embassy_stm32::init(Default::default());
       
       // Create sensor stream from ADC
       let temp_stream = adc_to_stream(p.ADC1, p.PA0);
       
       // Process with Fluxion operators
       let processed = temp_stream
           .map(|raw| raw * 0.001)  // Convert to Celsius
           .filter(|&temp| temp > 20.0)  // Only above 20°C
           .throttle(Duration::from_secs(1))  // Max 1 per second
           .subscribe(
               |temp, _token| async move {
                   // Control fan based on temperature
                   set_fan_speed((temp - 20.0) * 10.0).await;
                   Ok(())
               },
               |err| defmt::error!("Sensor error: {}", err),
               None,
           )
           .await;
   }
   ```

**Validation:**
- Compiles for `thumbv7em-none-eabihf` target
- Flash to STM32F4 board
- Verify stream processing works on embedded hardware

### Phase 4: Consolidate Test Infrastructure (Week 6)

**Goal:** Eliminate 80% of test duplication

1. **Create `fluxion-test-utils/src/runtime_harness.rs`**
   ```rust
   #[async_trait]
   pub trait RuntimeTestHarness {
       type Timer: Timer;
       type Timestamped<T>: Timestamped<Inner = T>;
       
       fn timer() -> Self::Timer;
       async fn sleep(duration: Duration);
       fn test_channel<T>() -> (Sender<T>, Receiver<T>);
   }
   
   // Shared test implementations
   pub mod shared_tests {
       pub async fn test_delay_basic<R: RuntimeTestHarness>() { ... }
       pub async fn test_throttle_basic<R: RuntimeTestHarness>() { ... }
       // ... 30 generic test implementations
   }
   ```

2. **Simplify runtime-specific tests**
   ```rust
   // fluxion-tokio/tests/delay_tests.rs
   use fluxion_test_utils::runtime_harness::{RuntimeTestHarness, shared_tests};
   
   struct TokioHarness;
   impl RuntimeTestHarness for TokioHarness { ... }
   
   #[tokio::test]
   async fn test_delay_basic() {
       shared_tests::test_delay_basic::<TokioHarness>().await;
   }
   ```

3. **Result:** 113 test files → ~30 shared + ~5 per runtime

### Phase 5: Update Documentation & CI (Week 7)

1. **Update main README.md**
   - Add "Choosing a Runtime" section
   - Update examples for runtime-specific crates

2. **Update CI workflows**
   - Separate jobs for each runtime crate
   - Parallel execution (faster CI)
   - Remove feature flag matrix explosion

3. **Generate runtime-specific docs**
   ```yaml
   # .github/workflows/docs.yml
   - name: Build docs (tokio)
     run: cargo doc -p fluxion-tokio
   - name: Build docs (wasm)
     run: cargo doc -p fluxion-wasm --target wasm32-unknown-unknown
   - name: Build docs (embassy)
     run: cargo doc -p fluxion-embassy --target thumbv7em-none-eabihf
   ```

---

## Embassy Example: Sensor Hub Specification

### Hardware Requirements

**Development Board:** STM32F4 Discovery (STM32F407VG)
- **Cost:** ~$15 USD
- **Programming:** USB (ST-Link built-in)
- **Availability:** Widely available, beginner-friendly

**Sensors (Optional, can simulate):**
1. **Built-in:** Accelerometer (LIS3DSH), LEDs, buttons
2. **External:** DHT22 temperature/humidity sensor (~$3)
3. **External:** Potentiometer for analog input (~$1)

### Application Architecture

```
Sensor Hub Architecture:

Hardware Inputs → Streams → Operators → Outputs
─────────────────────────────────────────────────
ADC (Potentiometer) ──┐
                      ├─→ merge → filter → map → throttle → PWM (LED)
I2C (Accelerometer) ──┘           ↓
                               debounce → UART (Debug)
```

### Feature Demonstrations

| Operator | Sensor Input | Processing | Output | Purpose |
|----------|--------------|------------|--------|---------|
| `map` | ADC voltage | Convert to percentage | Calculate LED brightness | Data transformation |
| `filter` | Accelerometer | Remove noise < threshold | Only significant motion | Signal filtering |
| `throttle` | Button press | Debounce (200ms) | LED toggle | Rate limiting |
| `merge` | Multiple sensors | Combine streams | Unified processing | Stream combination |
| `debounce` | Accelerometer shake | Wait 500ms stability | Log event | Event coalescing |
| `sample` | Temperature | Sample every 1s | Power-efficient reading | Periodic sampling |

### Memory & Performance Targets

- **Flash:** < 64 KB (STM32F4 has 1 MB)
- **RAM:** < 32 KB (STM32F4 has 192 KB)
- **Latency:** < 10ms from sensor read to actuator control
- **no-std:** Zero allocator, all stack-based

### Code Size Expectations

```rust
// Estimated binary sizes:
// Basic example (LED blink):        ~8 KB
// With one stream operator:         ~12 KB
// Full sensor hub (5 operators):    ~45 KB
// With defmt logging:               +10 KB

// This leaves plenty of room for application logic
```

---

## Benefits Analysis

### Development Experience Improvements

| Aspect | Current | With Proposal | Impact |
|--------|---------|---------------|--------|
| **WASM setup** | 4 crate deps, features | `fluxion-wasm = "0.6"` | ⭐⭐⭐⭐⭐ |
| **Embassy setup** | Not possible | `fluxion-embassy = "0.6"` | ⭐⭐⭐⭐⭐ |
| **IDE experience** | Confusing cfg-gated code | Clean, runtime-specific | ⭐⭐⭐⭐ |
| **Documentation** | Generic with caveats | Runtime-specific | ⭐⭐⭐⭐⭐ |
| **Error messages** | "doesn't implement Send" | "use Rc instead of Arc" | ⭐⭐⭐⭐ |
| **CI speed** | Serial feature matrix | Parallel runtime jobs | ⭐⭐⭐ |
| **Test maintenance** | Fix bug in 4 places | Fix once | ⭐⭐⭐⭐⭐ |

### Codebase Metrics

| Metric | Current | Proposed | Change |
|--------|---------|----------|--------|
| Crates | 10 | 17 (+7) | +70% |
| `#[cfg]` attributes | ~50 | ~5 | **-90%** |
| Test files | 113 | ~45 | **-60%** |
| Test code duplication | 80% | <15% | **-65%** |
| Runtime setup complexity | High | Low | **Much better** |
| Maintenance burden | High (fix 4x) | Low (fix 1x) | **Much better** |

---

## Risks & Mitigation

### Risk 1: Increased Crate Count
**Concern:** 10 → 17 crates may feel overwhelming

**Mitigation:**
- Clear naming: `fluxion-wasm`, `fluxion-embassy` are self-explanatory
- Main `fluxion` crate remains as facade for tokio users
- Documentation clearly explains which crate to use
- Examples ship with correct runtime crate

### Risk 2: Breaking Change for Existing Users
**Concern:** Users need to migrate dependencies

**Mitigation:**
- **Option A:** Keep `fluxion-exec` + `fluxion-stream-time` as-is, add new crates
  - Pros: Zero breaking change
  - Cons: Maintain two code paths temporarily
- **Option B:** Add deprecation warnings in v0.7, remove in v0.8
  ```rust
  #[deprecated(
      since = "0.7.0",
      note = "Use `fluxion-tokio` for tokio, `fluxion-wasm` for WASM, etc."
  )]
  pub use fluxion_exec_core::*;
  ```

**Recommendation:** Option A for v0.7, Option B for v0.8

### Risk 3: Implementation Complexity
**Concern:** Runtime abstraction trait may be hard to implement correctly

**Mitigation:**
- Start with proven patterns (Timer, Executor traits)
- Reference implementations: tokio, embassy (both use similar traits)
- Extensive testing with test harness
- Document trait requirements clearly

### Risk 4: Documentation Fragmentation
**Concern:** Users confused about which docs to read

**Mitigation:**
- Landing page (README.md) with clear "Choose Your Runtime" section
- Each runtime crate has identical API surface (same operators)
- Docs.rs shows all crates with clear descriptions
- Examples directory renamed for clarity (`tokio-*/`, `wasm-*/`, `embassy-*`)

---

## Success Criteria

### Technical Goals
- ✅ wasm-dashboard uses single `fluxion-wasm` dependency
- ✅ embassy-sensor-hub compiles for `thumbv7em-none-eabihf`
- ✅ embassy-sensor-hub runs on STM32F4 hardware
- ✅ < 5 `#[cfg]` attributes remaining in core crates
- ✅ Test suite runs 40% faster (parallelization)
- ✅ All existing tests pass (behavioral equivalence)

### User Experience Goals
- ✅ New WASM project: 1 dependency, not 4
- ✅ New Embassy project: Clear path from zero to hardware
- ✅ IDE autocomplete shows only relevant APIs
- ✅ Error messages specific to runtime constraints
- ✅ Documentation focused on runtime-specific use cases

### Maintenance Goals
- ✅ Bug fix applies once, not 4 times
- ✅ New runtime can be added without touching existing code
- ✅ CI jobs run in parallel (faster feedback)
- ✅ Complexity isolated to runtime-specific crates

---

## Timeline

### 7-Week Implementation Plan

| Week | Focus | Deliverables | Risk Level |
|------|-------|--------------|------------|
| 1-2 | Shared implementation extraction | `fluxion-runtime`, `fluxion-exec-core`, `fluxion-time-core` | Low |
| 3 | WASM crate | `fluxion-wasm`, updated wasm-dashboard | Medium |
| 4-5 | Embassy crate + example | `fluxion-embassy`, `embassy-sensor-hub` | **High** |
| 6 | Test consolidation | Shared test harness, reduced duplication | Medium |
| 7 | Documentation + CI | Updated docs, parallel CI, migration guide | Low |

**Critical Path:** Embassy example (Week 4-5) - requires hardware validation

---

## Alternatives Considered

### Alternative 1: Full Multiworkspace
**Description:** Separate workspaces for tokio/wasm/embassy

**Pros:** Maximum isolation

**Cons:**
- Code duplication (core operators)
- Coordination nightmare (version bumps)
- Publishing confusion (fluxion-tokio vs fluxion-wasm crates.io)

**Decision:** Rejected - too much overhead

### Alternative 2: Feature Flags Only
**Description:** Keep current structure, just clean up features

**Pros:** No structural changes

**Cons:**
- Doesn't solve wasm-dashboard DX issues
- Doesn't solve test duplication
- Embassy still awkward to use

**Decision:** Rejected - doesn't address root causes

### Alternative 3: Single Runtime Trait, No Separate Crates
**Description:** Add Runtime trait, but keep everything in existing crates

**Pros:** Fewer crates

**Cons:**
- wasm-dashboard still needs 4 dependencies
- Embassy users see irrelevant tokio docs
- Doesn't solve IDE experience

**Decision:** Rejected - insufficient improvement

---

## Recommendation

**Proceed with Hybrid Architecture (Proposed)**

**Rationale:**
1. **Addresses all pain points:** DX, test duplication, Embassy support
2. **Measured risk:** Phased rollout, backward compatibility
3. **Ecosystem alignment:** Similar to tokio's spawn vs spawn_local approach
4. **Proven pattern:** Runtime trait abstraction is well-understood
5. **Future-proof:** Easily add more runtimes (e.g., async-io, custom embedded)

**Priority Order:**
1. **Phase 2 (WASM)** - Immediate DX win for wasm-dashboard
2. **Phase 3 (Embassy)** - Unlock embedded use case
3. **Phase 1 (Shared impl)** - Enables 2 & 3
4. **Phase 4 (Tests)** - Quality of life improvement
5. **Phase 5 (Docs/CI)** - Polish

**Decision Point:** After Phase 2 completion, evaluate success criteria before proceeding to Phase 3.

---

## Appendix A: Dependency Graph

```
Dependency Flow:

fluxion-core (traits, types)
    ↑
    ├─ fluxion-stream (operators)
    ├─ fluxion-ordered-merge (algorithms)
    └─ fluxion-runtime (NEW: trait abstraction)
           ↑
           ├─ fluxion-exec-core (NEW: shared impl)
           └─ fluxion-time-core (NEW: shared impl)
                  ↑
                  ├─ fluxion-tokio (tokio integration)
                  ├─ fluxion-wasm (WASM integration)
                  ├─ fluxion-embassy (Embassy integration)
                  └─ fluxion-smol (smol integration)
                         ↑
                         └─ fluxion (facade, re-exports)
```

---

## Appendix B: Example Cargo.toml Changes

### Before (wasm-dashboard)
```toml
[dependencies]
fluxion-core = { path = "../../fluxion-core", default-features = false, features = ["alloc"] }
fluxion-exec = { path = "../../fluxion-exec", default-features = false }
fluxion-stream = { path = "../../fluxion-stream", default-features = false, features = ["alloc"] }
fluxion-stream-time = { path = "../../fluxion-stream-time", default-features = false, features = ["runtime-wasm"] }
async-channel = { workspace = true }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
# ... 15 more WASM deps
```

### After (wasm-dashboard)
```toml
[dependencies]
fluxion-wasm = { path = "../../fluxion-wasm" }  # Everything included!
web-sys = { version = "0.3", features = ["..."] }
# ... app-specific deps only
```

### Before (tokio example)
```toml
[dependencies]
fluxion = { path = "../../fluxion" }
tokio = { version = "1.40", features = ["full"] }
```

### After (tokio example)
```toml
[dependencies]
fluxion-tokio = { path = "../../fluxion-tokio" }  # Can also keep fluxion facade
tokio = { version = "1.40", features = ["rt", "macros"] }  # Minimal features needed
```

### New (embassy example)
```toml
[dependencies]
fluxion-embassy = { path = "../../fluxion-embassy" }
embassy-executor = { version = "0.9", features = ["nightly"] }
embassy-time = "0.5"
embassy-stm32 = { version = "0.1", features = ["stm32f407vg"] }
defmt = "0.3"
defmt-rtt = "0.4"
panic-probe = { version = "0.3", features = ["print-defmt"] }

[profile.release]
opt-level = "z"  # Optimize for size
lto = true
codegen-units = 1
```

---

## Appendix C: no-std Compatibility Matrix

| Crate | std | alloc | no-std | Target Support |
|-------|-----|-------|--------|----------------|
| `fluxion-core` | ✅ | ✅ | ✅ | All |
| `fluxion-stream` | ✅ | ✅ | ✅ | All |
| `fluxion-ordered-merge` | ✅ | ✅ | ✅ | All |
| `fluxion-runtime` | ✅ | ✅ | ✅ | All (trait only) |
| `fluxion-exec-core` | ✅ | ✅ | ✅ | All |
| `fluxion-time-core` | ✅ | ✅ | ✅ | All |
| `fluxion-tokio` | ✅ | ❌ | ❌ | std only |
| `fluxion-wasm` | ✅ | ❌ | ❌ | wasm32 + std |
| `fluxion-embassy` | ❌ | ✅ | ✅ | **Embedded no-std** |
| `fluxion-smol` | ✅ | ❌ | ❌ | std only |

**Key Insight:** 6 crates are no-std compatible, enabling 24/27 operators on embedded.

---

**End of Document**
