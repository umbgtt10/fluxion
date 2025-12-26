# Fluxion Workspace Restructuring Proposal

**Date:** December 26, 2025
**Status:** Design Proposal
**Purpose:** Address runtime complexity and prepare for no-std/Embassy expansion

---

## Executive Summary

This document proposes a **hybrid runtime-isolated architecture** with the following structure:

**Main Workspace (Single):**
- All library crates in one workspace (fluxion-core, fluxion-wasm, fluxion-tokio, etc.)
- Shared versioning, unified dependency management
- Single `cargo test --workspace` for all libraries

**Example Workspaces (Independent):**
- Each example is its own separate workspace
- Target-specific `.cargo/config.toml` (wasm32, thumbv7em, etc.)
- Solves rust-analyzer target/feature issues

**This is NOT a full multiworkspace split** (which was rejected) - the library crates stay together.

**Benefits:**
- ✅ Eliminates conditional compilation sprawl (50+ `#[cfg]` attributes)
- ✅ Improves developer experience for runtime-specific applications (WASM, Embassy)
- ✅ Reduces test duplication (113 → ~30 test files)
- ✅ Maintains single workspace for library coordination and versioning
- ✅ Examples get perfect rust-analyzer support (correct target auto-selected)
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
└── Examples (Independent Workspaces - NOT workspace members)
    ├── examples/tokio-stream-aggregation/    # Renamed from stream-aggregation
    │   ├── Cargo.toml                        # [workspace] - INDEPENDENT
    │   └── src/main.rs
    ├── examples/tokio-legacy-integration/    # Renamed from legacy-integration
    │   ├── Cargo.toml                        # [workspace] - INDEPENDENT
    │   └── src/main.rs
    ├── examples/wasm-dashboard/              # Uses fluxion-wasm
    │   ├── Cargo.toml                        # [workspace] - INDEPENDENT
    │   ├── .cargo/config.toml                # target = "wasm32-unknown-unknown"
    │   └── src/lib.rs
    └── examples/embassy-sensor-hub/          # NEW: Embassy no-std example
        ├── Cargo.toml                        # [workspace] - INDEPENDENT
        ├── .cargo/config.toml                # target = "thumbv7em-none-eabihf"
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

## Migration Strategy: Incremental, Non-Disruptive Approach

**Key Principle:** Each phase is **independently deployable** and **backward compatible**.

### Incremental Implementation Benefits

**Zero Breaking Changes During Migration:**
- ✅ Existing users continue using current API (`fluxion`, `fluxion-stream-time`, etc.)
- ✅ Old crates remain functional throughout migration
- ✅ New crates are **additive** - no removals until v0.8
- ✅ Each phase can be released as minor version bump (0.6.14, 0.6.15, etc.)
- ✅ Rollback possible at any phase boundary

**Independent Validation:**
- ✅ Each phase has clear success criteria
- ✅ Can pause after Phase 2 (WASM crate) and assess
- ✅ Deprecation warnings in v0.7, removals in v0.8
- ✅ Users have 6+ months to migrate

**Parallel Old/New Structure (Transition Period):**
```
# Week 1-2: Add fluxion-runtime, fluxion-exec-core, fluxion-time-core
# - Old crates STILL WORK (use new internals)
# - No user-facing changes

# Week 3: Add fluxion-wasm
# - wasm-dashboard CAN migrate (optional)
# - Old path still works

# Week 4-5: Add fluxion-embassy
# - NEW functionality (no old path to break)

# Week 6: Add test harness
# - Internal improvement (no API changes)

# Week 7+: Add deprecation warnings
# - Give users 6 months notice before v0.8
```

### Phase 1: Extract Shared Implementations (Week 1-2)

**Goal:** Zero duplication of subscribe/time logic

**Backward Compatibility Strategy:**
- Old crates (`fluxion-exec`, `fluxion-stream-time`) **become thin wrappers** around new core
- Public API unchanged - just internal refactoring
- Users see no difference

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

**Goal:** Improve wasm-dashboard development experience + fix rust-analyzer

**Backward Compatibility Strategy:**
- `fluxion-wasm` is **NEW** - doesn't break anything
- `wasm-dashboard` migration is **OPTIONAL** in v0.7
- Old WASM path (`fluxion-stream-time` with `runtime-wasm` feature) still works

**Release:** v0.7.0 (minor version bump)
- Changelog: "Added: `fluxion-wasm` crate for improved WASM support"
- Deprecation notice: "Note: `fluxion-stream-time` WASM path will be removed in v0.8"

1. **Create `fluxion-wasm` crate**
   - Implement `Runtime` trait for WASM
   - Provide `SubscribeExt` with `#[async_trait(?Send)]`
   - Re-export all WASM-compatible operators

2. **Convert `wasm-dashboard` to independent workspace**
   ```toml
   # examples/wasm-dashboard/Cargo.toml
   [package]
   name = "wasm-dashboard"
   version = "0.1.0"
   edition = "2021"

   [workspace]  # <-- Makes this an independent workspace (not a member)

   [dependencies]
   fluxion-wasm = { path = "../../fluxion-wasm" }  # Done - single dependency!
   web-sys = { version = "0.3", features = ["..."] }
   # ... app-specific deps
   ```

   ```toml
   # examples/wasm-dashboard/.cargo/config.toml
   [build]
   target = "wasm32-unknown-unknown"  # rust-analyzer uses this automatically
   ```

3. **Remove from root workspace members**
   ```toml
   # Root Cargo.toml - REMOVE wasm-dashboard from members
   [workspace]
   members = [
       "fluxion-core",
       "fluxion-wasm",
       # ... other crates
       # NOT: "examples/wasm-dashboard"  <-- REMOVED
   ]
   ```

4. **Update CI** - Keep `--exclude wasm-dashboard` (still needed for workspace commands)

**Benefits:**
- ✅ rust-analyzer automatically uses wasm32 target when editing wasm-dashboard files
- ✅ No more false errors about missing `Send` bounds
- ✅ Single dependency instead of 4
- ✅ IDE shows only WASM-relevant documentation

**Validation:**
- wasm-dashboard builds and runs identically
- Open `src/lib.rs` in VS Code - no rust-analyzer errors

### Phase 3: Create `fluxion-embassy` Crate (Week 4-5)

**Goal:** Enable no-std/Embassy development

**Backward Compatibility Strategy:**
- `fluxion-embassy` is **BRAND NEW** functionality
- No existing Embassy users to break
- Unlocks entirely new market segment

**Release:** v0.7.1 (minor version bump)
- Changelog: "Added: `fluxion-embassy` crate for embedded/no_std support"
- Blog post: "Fluxion on Embedded: Reactive Streams on STM32"

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

**Backward Compatibility Strategy:**
- Test consolidation is **INTERNAL ONLY**
- No public API changes
- All tests continue to pass
- Can be done incrementally (one operator at a time)

**Release:** No version bump needed (internal change)
- Commit messages track progress
- Can pause/resume anytime

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

**Backward Compatibility Strategy:**
- Documentation updates are **NON-BREAKING**
- CI improvements are **INFRASTRUCTURE ONLY**
- Can be done in parallel with other phases
- Modular CI refactoring is **OPTIONAL** (can defer to Phase 6)

**Release:** v0.7.2 (documentation-only bump)
- Changelog: "Improved: Runtime-specific documentation and examples"
- No code changes

1. **Update main README.md**
   - Add "Choosing a Runtime" section
   - Update examples for runtime-specific crates

2. **Refactor GitHub Actions workflows (RECOMMENDED)**

   **Current structure** (monolithic, 324 lines):
   ```
   .github/workflows/
   ├── ci.yml          # 324 lines: lint, test, embedded, embassy
   └── bench.yml       # 215 lines: run benchmarks, compare, deploy
   ```

   **Proposed structure** (reusable, modular):
   ```
   .github/workflows/
   ├── ci.yml                      # Main orchestrator (calls reusable workflows)
   ├── bench.yml                   # Benchmark orchestrator
   │
   ├── reusable/
   │   ├── setup-rust.yml          # Install Rust toolchain (reused everywhere)
   │   ├── cache-setup.yml         # Cargo cache configuration
   │   │
   │   ├── lint-format.yml         # cargo fmt --check
   │   ├── lint-clippy.yml         # cargo clippy -D warnings
   │   ├── lint-deny.yml           # cargo deny (licenses)
   │   │
   │   ├── test-tokio.yml          # Tokio runtime tests
   │   ├── test-async-std.yml      # async-std tests
   │   ├── test-smol.yml           # smol tests
   │   ├── test-wasm.yml           # WASM tests (wasm-pack)
   │   ├── test-embassy.yml        # Embassy tests (nightly)
   │   ├── test-examples.yml       # Test independent example workspaces
   │   │
   │   ├── build-workspace.yml     # Build all library crates
   │   ├── build-embedded.yml      # Build for thumbv7em (no_std)
   │   ├── build-docs.yml          # Generate documentation
   │   │
   │   ├── bench-run.yml           # Run criterion benchmarks
   │   ├── bench-compare.yml       # Compare with baseline
   │   └── bench-deploy.yml        # Deploy to GitHub Pages
   │
   └── scheduled/
       ├── nightly-tests.yml       # Nightly comprehensive tests
       └── weekly-audit.yml        # Security audit + dependency updates
   ```

   **New main CI workflow** (`.github/workflows/ci.yml`):
   ```yaml
   name: CI

   on:
     push:
       branches: [main]
     pull_request:

   jobs:
     # Setup (cached, runs once)
     setup:
       uses: ./.github/workflows/reusable/setup-rust.yml
       with:
         toolchain: stable

     # Lint jobs (parallel)
     fmt:
       needs: setup
       uses: ./.github/workflows/reusable/lint-format.yml

     clippy:
       needs: setup
       uses: ./.github/workflows/reusable/lint-clippy.yml

     deny:
       needs: setup
       uses: ./.github/workflows/reusable/lint-deny.yml

     # Build jobs (parallel, after lint)
     build-workspace:
       needs: [fmt, clippy]
       uses: ./.github/workflows/reusable/build-workspace.yml
       with:
         profile: release

     build-embedded:
       needs: [fmt, clippy]
       uses: ./.github/workflows/reusable/build-embedded.yml
       with:
         target: thumbv7em-none-eabihf

     # Test jobs (parallel, after build)
     test-tokio:
       needs: build-workspace
       uses: ./.github/workflows/reusable/test-tokio.yml
       strategy:
         matrix:
           os: [ubuntu-latest, windows-latest, macos-latest]
       with:
         os: ${{ matrix.os }}

     test-wasm:
       needs: build-workspace
       uses: ./.github/workflows/reusable/test-wasm.yml

     test-embassy:
       needs: build-workspace
       uses: ./.github/workflows/reusable/test-embassy.yml

     test-examples:
       needs: build-workspace
       uses: ./.github/workflows/reusable/test-examples.yml

     # Documentation
     docs:
       needs: build-workspace
       uses: ./.github/workflows/reusable/build-docs.yml
   ```

   **Benefits:**
   - ✅ **Each workflow = single responsibility** (easier to debug)
   - ✅ **Parallel execution** (faster CI - 5 test jobs run simultaneously)
   - ✅ **Reusable across workflows** (nightly tests reuse same jobs)
   - ✅ **Matrix multiplication** (easy to test more OS/runtime combinations)
   - ✅ **Easy selective runs** - Manually trigger specific jobs
   - ✅ **Better caching** - Shared setup job caches once
   - ✅ **Easier maintenance** - Change one file, not 324 lines

   **Benchmark workflow** (`.github/workflows/bench.yml`):
   ```yaml
   name: Benchmarks

   on:
     workflow_dispatch:
     schedule:
       - cron: '0 4 * * *'

   jobs:
     run-benchmarks:
       uses: ./.github/workflows/reusable/bench-run.yml
       with:
         packages: "fluxion-core,fluxion-stream,fluxion-stream-time,fluxion-ordered-merge"

     compare-baseline:
       needs: run-benchmarks
       uses: ./.github/workflows/reusable/bench-compare.yml
       with:
         threshold: 10  # Fail on >10% regression

     deploy-pages:
       needs: compare-baseline
       if: github.ref == 'refs/heads/main'
       uses: ./.github/workflows/reusable/bench-deploy.yml
   ```

3. **Update CI workflows (.github/workflows/ci.yml) - Minimal changes**

   **Keep `--exclude wasm-dashboard` everywhere** (examples not in workspace)
   ```yaml
   # All workspace commands stay the same - examples are excluded automatically
   - name: Build workspace
     run: cargo build --workspace --exclude wasm-dashboard  # Still needed!

   # Test examples with explicit manifest paths
   - name: Test tokio examples
     run: |
       cargo test --manifest-path examples/tokio-stream-aggregation/Cargo.toml
       cargo test --manifest-path examples/tokio-legacy-integration/Cargo.toml

   - name: Test WASM example
     run: cargo test --manifest-path examples/wasm-dashboard/Cargo.toml --target wasm32-unknown-unknown

   - name: Test Embassy example
     run: cargo test --manifest-path examples/embassy-sensor-hub/Cargo.toml --target thumbv7em-none-eabihf
   ```

3. **Refactor PowerShell CI scripts (RECOMMENDED)**

   **Same modular approach as GitHub Actions** - organize by responsibility

   **Current structure** (flat, monolithic):
   ```
   .ci/
   ├── ci.ps1                      # Main orchestrator (126 lines)
   ├── build.ps1                   # Build + upgrade (224 lines)
   ├── tokio_tests.ps1
   ├── async_std_tests.ps1
   ├── smol_tests.ps1
   ├── embassy_tests.ps1
   ├── wasm_tests.ps1
   ├── no_std_check.ps1
   ├── test_feature_gating.ps1
   └── sync-readme-examples.ps1
   ```

   **Proposed structure** (organized, modular):
   ```
   .ci/
   ├── ci.ps1                      # Main orchestrator (calls subscripts)
   │
   ├── scripts/
   │   ├── common/
   │   │   ├── utils.ps1           # Shared: Write-Color, Invoke-StepAction
   │   │   └── install.ps1         # Shared: Install cargo tools
   │   │
   │   ├── check/
   │   │   ├── format.ps1          # cargo fmt --check
   │   │   ├── clippy.ps1          # cargo clippy with -D warnings
   │   │   ├── no_std.ps1          # no_std compilation check
   │   │   └── features.ps1        # Feature gating verification
   │   │
   │   ├── test/
   │   │   ├── tokio.ps1           # Tokio runtime tests
   │   │   ├── async_std.ps1       # async-std tests
   │   │   ├── smol.ps1            # smol tests
   │   │   ├── embassy.ps1         # Embassy tests
   │   │   ├── wasm.ps1            # WASM tests (wasm-pack)
   │   │   └── examples.ps1        # Test all example workspaces
   │   │
   │   ├── build/
   │   │   ├── workspace.ps1       # Build workspace crates
   │   │   ├── examples.ps1        # Build example workspaces
   │   │   └── docs.ps1            # Generate docs
   │   │
   │   └── tools/
   │       ├── upgrade.ps1         # cargo upgrade --workspace
   │       └── audit.ps1           # cargo audit
   │
   └── examples/                   # Example-specific scripts
       ├── wasm-dashboard/
       │   └── scripts/
       │       └── run-dashboard.ps1
       └── embassy-sensor-hub/
           └── scripts/
               └── flash.ps1       # Flash to hardware
   ```

   **New main orchestrator** (`.ci/ci.ps1`):
   ```powershell
   # Simplified main script
   param([switch]$SkipTests, [switch]$Fast)

   Set-StrictMode -Version Latest
   $ErrorActionPreference = 'Stop'

   # Import common utilities
   . "$PSScriptRoot\scripts\common\utils.ps1"
   . "$PSScriptRoot\scripts\common\install.ps1"

   Write-Header "Fluxion CI Pipeline"

   # Checks
   Invoke-Script "$PSScriptRoot\scripts\check\format.ps1"
   Invoke-Script "$PSScriptRoot\scripts\check\features.ps1"
   Invoke-Script "$PSScriptRoot\scripts\check\no_std.ps1"
   Invoke-Script "$PSScriptRoot\scripts\check\clippy.ps1"

   # Build
   Invoke-Script "$PSScriptRoot\scripts\build\workspace.ps1"

   if (-not $SkipTests) {
       # Tests (can run in parallel with Start-Job)
       Invoke-Script "$PSScriptRoot\scripts\test\tokio.ps1"
       Invoke-Script "$PSScriptRoot\scripts\test\wasm.ps1"

       if (-not $Fast) {
           Invoke-Script "$PSScriptRoot\scripts\test\async_std.ps1"
           Invoke-Script "$PSScriptRoot\scripts\test\smol.ps1"
           Invoke-Script "$PSScriptRoot\scripts\test\embassy.ps1"
       }

       Invoke-Script "$PSScriptRoot\scripts\test\examples.ps1"
   }

   # Documentation
   Invoke-Script "$PSScriptRoot\scripts\build\docs.ps1"

   Write-Success "✅ CI pipeline complete!"
   ```

   **Benefits:**
   - ✅ Each script has single responsibility
   - ✅ Easy to run individual checks: `.ci/scripts/test/wasm.ps1`
   - ✅ Shared utilities imported once (DRY principle)
   - ✅ CI flags: `./ci.ps1 -Fast` (skip slow runtimes)
   - ✅ Parallel execution possible with `Start-Job`
   - ✅ Easier to maintain and understand

4. **Update PowerShell scripts (minimal changes)**

   **No immediate changes needed** - all scripts already use `--exclude wasm-dashboard`

   Scripts already correct:
   - `.ci/ci.ps1` - Line 67-71: Already excludes wasm-dashboard
   - `.ci/build.ps1` - Line 167-168: Already excludes wasm-dashboard
   - `.ci/tokio_tests.ps1` - Line 50: Already excludes wasm-dashboard
   - `.ci/no_std_check.ps1` - Line 68, 76: Already excludes wasm-dashboard

   **Add new script** (`.ci/scripts/test/examples.ps1`):
   ```powershell
   # Test all independent example workspaces
   Invoke-StepAction "Test tokio examples" {
     cargo test --manifest-path examples/tokio-stream-aggregation/Cargo.toml
     cargo test --manifest-path examples/tokio-legacy-integration/Cargo.toml
   }

   Invoke-StepAction "Test WASM example" {
     cargo test --manifest-path examples/wasm-dashboard/Cargo.toml --target wasm32-unknown-unknown
   }

   Invoke-StepAction "Test Embassy example" {
     cargo test --manifest-path examples/embassy-sensor-hub/Cargo.toml --target thumbv7em-none-eabihf
   }
   ```

5. **Update example scripts (examples/wasm-dashboard/scripts/run-dashboard.ps1)**

   **No changes needed** - script already uses relative paths from `$PSScriptRoot`
   ```powershell
   # Already works correctly
   $DashboardDir = Join-Path $PSScriptRoot ".."
   if (-not (Test-Path (Join-Path $DashboardDir "Cargo.toml"))) {
     Write-Error "Could not find Cargo.toml"
   }
   ```

6. **Generate runtime-specific docs**
   ```yaml
   # .github/workflows/docs.yml
   - name: Build docs (tokio)
     run: cargo doc -p fluxion-tokio
   - name: Build docs (wasm)
     run: cargo doc -p fluxion-wasm --target wasm32-unknown-unknown
   - name: Build docs (embassy)
     run: cargo doc -p fluxion-embassy --target thumbv7em-none-eabihf
   ```

**Summary of CI/Script Changes:**
- ✅ **GitHub Actions CI - Immediate**: Add explicit example test steps with `--manifest-path`
- ✅ **GitHub Actions CI - Recommended**: Refactor into reusable workflows (`.github/workflows/reusable/`)
- ✅ **Benchmark workflow - Recommended**: Split into run/compare/deploy reusable workflows
- ✅ **PowerShell scripts - Immediate**: No changes needed (already exclude wasm-dashboard)
- ✅ **PowerShell scripts - Recommended**: Refactor into modular structure (`.ci/scripts/`)
- ✅ **Example scripts**: No changes needed (use relative paths)
- ✅ **Keep `--exclude wasm-dashboard`**: Still required since examples not in members list
- ✅ **Benefits**: Single-responsibility, parallel execution, reusability, easier maintenance
- ✅ **Timeline**: Immediate changes for Phase 5 (Week 7), modular refactoring optional (Phase 6)

**Refactoring Priority:**
1. **High**: GitHub Actions reusable workflows (enables parallel execution, faster CI)
2. **Medium**: PowerShell scripts (improves local dev experience)
3. **Low**: Benchmark workflow (works well currently, but modularity helps debugging)

---

## Incremental Adoption Path for Users

### Migration Timeline (No Breaking Changes)

**v0.6.x (Current) → v0.7.0 (Additive Changes Only)**
```toml
# OLD WAY (still works in v0.7)
[dependencies]
fluxion-stream-time = { version = "0.7", features = ["runtime-wasm"] }

# NEW WAY (available in v0.7, recommended)
[dependencies]
fluxion-wasm = "0.7"  # Cleaner, better DX
```

**v0.7.x (Transition Period - 6 months)**
- Both old and new APIs work
- Deprecation warnings guide users
- Examples show new patterns
- Migration guide published

**v0.8.0 (Clean Break)**
- Remove old runtime features from `fluxion-stream-time`
- Remove `fluxion-exec` (replaced by `fluxion-exec-core` + runtime crates)
- Clean, focused crates

### Per-Phase User Impact

| Phase | Version | User-Facing Changes | Migration Required? |
|-------|---------|---------------------|---------------------|
| 1 (Shared impl) | 0.6.14 | None (internal) | ❌ No |
| 2 (WASM crate) | 0.7.0 | New `fluxion-wasm` available | ❌ Optional |
| 3 (Embassy) | 0.7.1 | New `fluxion-embassy` available | ❌ Optional (new feature) |
| 4 (Test harness) | Internal | None (internal) | ❌ No |
| 5 (Docs/CI) | 0.7.2 | Better docs | ❌ No |
| 6 (Deprecation) | 0.7.3 | Warnings on old API | ⚠️ Suggested |
| 7 (Cleanup) | 0.8.0 | Old features removed | ✅ Yes (6 months notice) |

### Example: WASM User Migration (Optional in v0.7)

**Before (v0.6.x - v0.7.x, still works):**
```toml
[dependencies]
fluxion-core = { version = "0.7", default-features = false }
fluxion-exec = { version = "0.7", default-features = false }
fluxion-stream = { version = "0.7", default-features = false }
fluxion-stream-time = { version = "0.7", features = ["runtime-wasm"] }
```

**After (v0.7.0+, recommended):**
```toml
[dependencies]
fluxion-wasm = "0.7"  # Single dependency!
```

**When to migrate:**
- ⏰ **Recommended:** During v0.7.x (before v0.8)
- 🔔 **Warning:** v0.7.3+ shows deprecation warnings
- ⚠️ **Required:** Before upgrading to v0.8.0

### Rollback Strategy

**Each phase is independently reversible:**

**If Phase 2 (WASM crate) has issues:**
1. Mark `fluxion-wasm` as deprecated
2. Keep old `runtime-wasm` feature working
3. No users affected (new crate was optional)

**If Phase 3 (Embassy) has issues:**
1. Mark `fluxion-embassy` as experimental
2. No users affected (brand new feature)
3. Can iterate without breaking anyone

**If entire migration needs pause:**
- Old crates (`fluxion-stream-time` with features) work forever in v0.7.x
- Can delay v0.8.0 cleanup indefinitely
- Each released phase provides value independently

### Canary Testing Strategy

**Before each phase release:**

1. **Alpha release** (e.g., v0.7.0-alpha.1)
   - Published to crates.io
   - Announced on GitHub Discussions
   - Request feedback from power users

2. **Beta release** (e.g., v0.7.0-beta.1)
   - Documentation finalized
   - Migration guide published
   - CI testing across all platforms

3. **Release candidate** (e.g., v0.7.0-rc.1)
   - 1 week soak period
   - Monitor GitHub issues
   - Fix critical bugs only

4. **Stable release** (v0.7.0)
   - Changelog published
   - Blog post / announcement
   - Backward compatibility guaranteed

**Total time per phase:** 2-3 weeks (development) + 2 weeks (alpha/beta/rc)

### Parallel Development Model

**Enables concurrent work without blocking:**

```
Week 1-2:  Phase 1 (Shared impl)      [Main branch]
Week 3:    Phase 2 (WASM crate)        [Main branch]
Week 4-5:  Phase 3 (Embassy)           [feature/embassy branch]
           Phase 4 (Tests)             [feature/test-harness branch] ← parallel!
Week 6:    Phase 3 merge               [Main branch]
Week 7:    Phase 4 merge               [Main branch]
Week 8:    Phase 5 (Docs/CI)           [Main branch]
```

**Benefits:**
- Phase 3 and Phase 4 can be developed simultaneously
- Phase 5 (docs) can start during Phase 3-4
- Each merge is independently validated
- Failed phase doesn't block others
- ✅ **Example scripts**: No changes needed (use relative paths)
- ✅ **Keep `--exclude wasm-dashboard`**: Still required since examples not in members list
- ✅ **Benefits**: Single-responsibility, parallel execution, reusability, easier maintenance
- ✅ **Timeline**: Immediate changes for Phase 5 (Week 7), modular refactoring optional (Phase 6)

**Refactoring Priority:**
1. **High**: GitHub Actions reusable workflows (enables parallel execution, faster CI)
2. **Medium**: PowerShell scripts (improves local dev experience)
3. **Low**: Benchmark workflow (works well currently, but modularity helps debugging)

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
| **WASM rust-analyzer** | ❌ False errors, wrong target | ✅ Clean, correct target | ⭐⭐⭐⭐⭐ |
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
- Examples are independent workspaces for clarity (`tokio-*/`, `wasm-*/`, `embassy-*`)

### Risk 5: Examples Not Part of Main Workspace
**Concern:** Examples as independent workspaces might be harder to maintain

**Mitigation:**
- **Benefit:** Solves rust-analyzer target/feature issues completely
- **Benefit:** Each example can specify its own `.cargo/config.toml` (auto-target selection)
- **Benefit:** No more `--exclude` hacks in CI
- CI can still test examples with explicit paths: `cargo test --manifest-path examples/wasm-dashboard/Cargo.toml`
- Version updates: Simple find-replace across example Cargo.toml files
- Better isolation: Example dependencies don't pollute main workspace

---

## Success Criteria

### Technical Goals
- ✅ wasm-dashboard uses single `fluxion-wasm` dependency
- ✅ wasm-dashboard is independent workspace with `.cargo/config.toml`
- ✅ rust-analyzer shows no errors when editing WASM example files
- ✅ embassy-sensor-hub compiles for `thumbv7em-none-eabihf`
- ✅ embassy-sensor-hub runs on STM32F4 hardware
- ✅ < 5 `#[cfg]` attributes remaining in core crates
- ✅ Test suite runs 40% faster (parallelization)
- ✅ All existing tests pass (behavioral equivalence)

### User Experience Goals
- ✅ New WASM project: 1 dependency, not 4
- ✅ Open WASM example in IDE: Zero false rust-analyzer errors
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

## Value Proposition: Beyond Development Experience

### Technical Sophistication Showcase

This restructuring demonstrates **advanced Rust ecosystem expertise** in multiple dimensions:

**1. Multi-Runtime Mastery**
- **Problem space**: Async Rust has 5+ runtimes with incompatible APIs (tokio, async-std, smol, WASM, Embassy)
- **Solution**: Runtime trait abstraction with shared implementation
- **Demonstrates**: Deep understanding of trait design, conditional compilation, zero-cost abstractions
- **Industry relevance**: Very few libraries successfully support this breadth (tokio ecosystem, embassy)

**2. no_std/Embedded Expertise**
- **Problem space**: Embedded Rust is growing but lacks reactive stream libraries
- **Solution**: Full no_std support + Embassy example with hardware validation
- **Demonstrates**: Embedded constraints (no heap, no threads), bare-metal timing, hardware integration
- **Market gap**: 24/27 operators on embedded - **no competitor offers this**

**3. WASM Production Readiness**
- **Problem space**: WASM libraries often have unclear setup, poor IDE support
- **Solution**: Dedicated `fluxion-wasm` crate + independent workspace
- **Demonstrates**: Modern WASM best practices, build tooling (trunk), browser integration
- **Validation**: Production-quality dashboard example

**4. CI/CD & Testing Excellence**
- **Problem space**: Multi-runtime testing is complex (5 runtimes × 3 OS × 2 targets = 30 combinations)
- **Solution**: Modular reusable workflows, shared test harness, parallel execution
- **Demonstrates**: DevOps maturity, scalable testing strategy, quality assurance
- **Metric**: 90% reduction in conditional compilation, 60% reduction in test duplication

### Ecosystem Alignment & Industry Standards

**Follows patterns from established projects:**

| Pattern | Fluxion | Industry Example |
|---------|---------|------------------|
| Runtime-specific crates | `fluxion-tokio`, `fluxion-wasm` | `tokio-util`, `tokio-stream`, `async-std` |
| Shared core + facades | `fluxion-core` + runtime crates | `embassy-executor` + `embassy-stm32` |
| Independent examples | Each example = workspace | `tokio/examples`, `embassy/examples` |
| Reusable CI workflows | `.github/workflows/reusable/` | `rust-lang/rust` workflows |
| Test harness abstraction | Trait-based shared tests | `tokio/tests` macros |

**This positions Fluxion as:**
- ✅ **Enterprise-grade** - Clear separation of concerns, professional structure
- ✅ **Contributor-friendly** - Easy to understand, test, and extend
- ✅ **Production-ready** - Comprehensive testing, clear deployment patterns

### Marketability & Visibility

**This restructuring enables compelling narratives:**

**1. Blog Posts / Conference Talks**
- "Building a Multi-Runtime Reactive Streams Library in Rust"
- "From Tokio to Embassy: Supporting 5 Async Runtimes with Shared Code"
- "WASM + Rust Streams: A Production Dashboard Case Study"
- "Embedded Reactive Programming: Fluxion on STM32 Hardware"

**2. Documentation Excellence**
- Runtime-specific docs on docs.rs (no confusion about which APIs work where)
- Hardware example with bill of materials, wiring diagrams, flashing instructions
- Clear migration guides (RxRust → Fluxion)

**3. Benchmarking Transparency**
- Public GitHub Pages with interactive Criterion reports
- Regression detection in CI (10% threshold)
- Baseline tracking across releases

**4. Open Source Credibility Indicators**
- Well-organized crate structure (signals maturity)
- Comprehensive CI (signals quality commitment)
- Multiple working examples (signals production-readiness)
- Cross-platform support (signals serious engineering)

### Contribution & Adoption Acceleration

**Lowers barriers to entry:**

**For Contributors:**
- ✅ **Clear structure** - Know where to add new operators (which crate?)
- ✅ **Modular tests** - Add runtime tests without touching 113 files
- ✅ **Fast feedback** - Run only relevant tests (`.ci/scripts/test/tokio.ps1`)
- ✅ **Good examples** - See how operators work in real applications

**For Adopters:**
- ✅ **Easy evaluation** - Try runtime-specific example in 5 minutes
- ✅ **Clear migration path** - Runtime-specific docs show exact steps
- ✅ **Confidence in quality** - Comprehensive CI visible on GitHub
- ✅ **Long-term viability** - Well-maintained structure signals commitment

**For Employers/Evaluators:**
- ✅ **Technical depth** - Demonstrates multi-domain Rust expertise
- ✅ **Architectural maturity** - Shows systems thinking, not just coding
- ✅ **DevOps competence** - CI/CD, testing strategy, automation
- ✅ **Project management** - 7-week phased rollout plan, risk mitigation

### Competitive Differentiation

**Current Rust reactive streaming landscape:**

| Library | Tokio | async-std | smol | WASM | Embassy | no_std | Structure Quality |
|---------|-------|-----------|------|------|---------|--------|-------------------|
| **Fluxion (after)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⭐⭐⭐⭐⭐ |
| RxRust | ✅ | ❌ | ❌ | Partial | ❌ | ❌ | ⭐⭐⭐ |
| futures-rs | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ⭐⭐⭐⭐ |
| async-stream | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ⭐⭐⭐ |

**Unique selling points enabled by this structure:**
1. **Only reactive streams library with Embassy support** (embedded market)
2. **Best-in-class WASM DX** (independent workspace, perfect rust-analyzer)
3. **Most comprehensive testing** (5 runtimes × 3 OS, automated regression)
4. **Clear runtime-specific docs** (no "this might not work on your runtime" caveats)
5. **Hardware example** (physical STM32 demo - tangible!)

### ROI: Investment vs. Return

**Investment:**
- 7 weeks engineering time (phased rollout)
- ~7 new crates (manageable increase)
- CI/script refactoring (one-time cost)

**Return:**
- **Short-term**: Fixes wasm-dashboard rust-analyzer (immediate win)
- **Medium-term**: Enables Embassy example (new market segment)
- **Long-term**:
  - Easier maintenance (fix once, not 4×)
  - Faster CI (parallel execution)
  - More contributors (clear structure)
  - Higher adoption (runtime-specific marketing)
  - Conference talks / blog posts (visibility)
  - Portfolio piece (demonstrates architecture skills)

**Intangible value:**
- Positions Fluxion as **the** multi-runtime reactive streams library in Rust
- Creates barrier to entry for competitors (comprehensive structure)
- Demonstrates **technical leadership** in async Rust ecosystem

### Summary: The "Rolls Royce" Justification

**Yes, this migration adds significant value beyond DX:**

✅ **Technical credibility** - Demonstrates mastery of complex Rust patterns
✅ **Ecosystem leadership** - Sets standard for multi-runtime libraries
✅ **Market differentiation** - Only library with this breadth of support
✅ **Contribution magnet** - Clean structure attracts contributors
✅ **Production confidence** - Professional CI/testing signals quality
✅ **Career showcase** - Demonstrates systems architecture & project management
✅ **Long-term maintenance** - Pays dividends in reduced complexity

**This is not just refactoring - it's strategic positioning.**

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
