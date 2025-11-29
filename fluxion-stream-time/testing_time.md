# Testing Time-Dependent Operators

Testing asynchronous code that relies on the passage of time (e.g., `debounce`, `throttle`, `delay`) presents unique challenges. This document outlines the strategies available, the specific constraints imposed by the Tokio runtime, and a recommended architecture for robust, runtime-agnostic testing.

---

## Table of Contents

1. [The Fundamental Challenge](#the-fundamental-challenge)
2. [Testing Approaches](#testing-approaches)
   - [Approach 1: Real Time](#approach-1-real-time-not-recommended)
   - [Approach 2: Simulated Time (Global Clock)](#approach-2-simulated-time-global-clock)
   - [Approach 3: Abstracted Time (Dependency Injection)](#approach-3-abstracted-time-dependency-injection)
3. [Tokio-Specific Considerations](#tokio-specific-considerations)
   - [`start_paused = true` vs. `pause()`](#start_paused--true-vs-pause)
   - [The `yield_now()` Requirement](#the-yield_now-requirement)
   - [Limitations of Simulated Time](#limitations-of-simulated-time)
4. [Multi-Runtime Roadmap Implications](#multi-runtime-roadmap-implications)
5. [The Recommended Architecture](#the-recommended-architecture)
   - [Step 1: Define a Time Trait](#step-1-define-a-time-trait)
   - [Step 2: Implement for Each Runtime](#step-2-implement-for-each-runtime)
   - [Step 3: Inject into Operators](#step-3-inject-into-operators)
   - [Step 4: Default for Ergonomics](#step-4-default-for-ergonomics)
   - [Step 5: MockTimer for Tests](#step-5-mocktimer-for-tests)
6. [Summary of Trade-offs](#summary-of-trade-offs)
7. [Feature-Gated Runtime Implementations](#feature-gated-runtime-implementations)
8. [Parameterized Tests Across Runtimes](#parameterized-tests-across-runtimes)
9. [Assessment: Is the Current Approach Sufficient?](#assessment-is-the-current-approach-sufficient)

---

## The Fundamental Challenge

In production, time flows continuously. A `debounce(500ms)` operator waits half a second after the last event before emitting. Testing this behavior with *real* time is problematic:

1.  **Performance**: A test suite with hundreds of time-dependent tests could take minutes or hours.
2.  **Flakiness**: System load, GC pauses, or scheduler jitter can cause real sleeps to over- or under-shoot, leading to non-deterministic failures.

The goal of any testing strategy is to **decouple the logic of time from the wall clock**, allowing tests to be fast, deterministic, and reproducible.

---

## Testing Approaches

### Approach 1: Real Time (Not Recommended)

Use actual `std::thread::sleep` or `tokio::time::sleep` with real durations.

| Aspect      | Details                                                                 |
|-------------|-------------------------------------------------------------------------|
| **Pros**    | Zero infrastructure; tests mirror production exactly.                   |
| **Cons**    | Extremely slow; inherently flaky; unsuitable for CI.                    |
| **Verdict** | **Avoid entirely.** Only acceptable for manual smoke tests.             |

---

### Approach 2: Simulated Time (Global Clock)

Use the runtime's built-in time-mocking facilities (e.g., `tokio::time::pause()` and `tokio::time::advance()`).

**Mechanism:**
- The runtime intercepts all calls to its timer APIs (`sleep`, `timeout`, `interval`).
- Time is "frozen" at a specific instant.
- Calling `advance(duration)` instantly fires all timers scheduled to expire within that window, without wall-clock delay.

| Aspect         | Details                                                                                                |
|----------------|--------------------------------------------------------------------------------------------------------|
| **Pros**       | No changes to production code; tests are fast; linear, readable test flow.                             |
| **Cons**       | Tightly coupled to a specific runtime; susceptible to concurrency races with `tokio::spawn`.           |
| **Verdict**    | **Good for single-runtime projects.** Requires careful synchronization (`yield_now()`).                |

---

### Approach 3: Abstracted Time (Dependency Injection)

Define an abstract `Timer` trait and inject the implementation into operators.

**Mechanism:**
- Production code calls `timer.sleep(duration)` instead of `tokio::time::sleep(duration)`.
- In production, `timer` is a `TokioTimer` (or `AsyncStdTimer`, `SmolTimer`, etc.).
- In tests, `timer` is a `MockTimer` that allows the test to manually control when each sleep completes.

| Aspect         | Details                                                                                                |
|----------------|--------------------------------------------------------------------------------------------------------|
| **Pros**       | Fully runtime-agnostic; 100% deterministic tests; no concurrency races; zero-cost with monomorphization.|
| **Cons**       | More verbose operator signatures; initial setup cost.                                                  |
| **Verdict**    | **Best for multi-runtime libraries.** The investment pays off in long-term maintainability.            |

---

## Tokio-Specific Considerations

### `start_paused = true` vs. `pause()`

Tokio offers two ways to freeze time:

1.  **`#[tokio::test(start_paused = true)]`**: Time is paused *before* the async runtime starts the test body. This is the most "aggressive" freeze.
2.  **Explicit `tokio::time::pause()`**: Time is paused at a specific point within the test, after some setup code has run.

**The Problem with `start_paused = true`:**

When time is paused from the very beginning, and an operator uses `tokio::spawn` to run background logic, there is a **race condition**:

```
Test Body                     Spawned Task
----------                     ------------
1. Creates operator
2. Calls advance(500ms)        (may not have run yet!)
                               3. Registers its Sleep timer
                               4. (Timer deadline is now in the "past")
```

In this scenario, the spawned task's timer never fires because the test advanced past its deadline before the task even registered it.

**Why `pause()` is often more stable:**

By calling `pause()` *after* the initial setup (channel creation, operator construction), you give the Tokio scheduler more opportunity to initialize background tasks before freezing the clock.

---

### The `yield_now()` Requirement

Even with `pause()`, you may need to use `tokio::task::yield_now().await` to ensure background tasks have progressed to their `await` points before advancing time. This "forces a context switch," allowing the executor to run pending tasks.

```rust
tx.send(value)?;
tokio::task::yield_now().await; // <-- Ensures operator's internal task has processed the send
advance(Duration::from_millis(500)).await;
```

This is a fragile pattern, as it relies on implementation details of the operator. However, it is often necessary when using Tokio's simulated time with operators that spawn internal tasks.

---

### Limitations of Simulated Time

1.  **Runtime Lock-in**: `tokio::time::pause()` only works with Tokio. It cannot be used with `async-std`, `smol`, or WASM runtimes.
2.  **Global State**: The paused clock is a process-global state, which can cause interference if tests run in parallel within the same process (though `#[tokio::test]` spawns isolated runtimes by default).
3.  **Opacity**: Tests become dependent on internal implementation details (e.g., whether the operator uses `spawn` or polls inline).

---

## Multi-Runtime Roadmap Implications

If the roadmap includes supporting `async-std`, `smol`, or `wasm-bindgen-futures`:

-   `tokio::time::pause()` is **not portable**. There is no equivalent in `async-std`.
-   Each runtime has its own timer wheel implementation.
-   The only portable solution is **Abstracted Time (Approach 3)**, where the timer is a generic parameter or injected dependency.

---

## The Recommended Architecture

To achieve the stated goals of **zero performance impact**, **minimal code intrusion**, and **full test control**, the following architecture is recommended:

### Step 1: Define a Time Trait

Create a trait that abstracts timer operations.

```rust
use std::future::Future;
use std::time::{Duration, Instant};

pub trait AsyncTimer: Send + Sync + 'static {
    type Sleep: Future<Output = ()> + Send;

    /// Creates a future that completes after `duration`.
    fn sleep(&self, duration: Duration) -> Self::Sleep;

    /// Returns the current instant according to this timer.
    fn now(&self) -> Instant;
}
```

### Step 2: Implement for Each Runtime

Provide zero-cost implementations for each supported runtime. Because these are unit structs with no fields, they compile away completely.

```rust
// --- Tokio ---
pub struct TokioTimer;

impl AsyncTimer for TokioTimer {
    type Sleep = tokio::time::Sleep;

    fn sleep(&self, duration: Duration) -> Self::Sleep {
        tokio::time::sleep(duration)
    }

    fn now(&self) -> Instant {
        tokio::time::Instant::now().into_std()
    }
}

// --- async-std (future) ---
// pub struct AsyncStdTimer;
// impl AsyncTimer for AsyncStdTimer { ... }
```

### Step 3: Inject into Operators

Use generics to accept any timer implementation. This avoids `Box<dyn ...>` overhead.

```rust
pub fn debounce_with_timer<S, T, Timer>(
    stream: S,
    duration: Duration,
    timer: Timer,
) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
    Timer: AsyncTimer,
{
    DebounceStream {
        stream,
        duration,
        timer,
        // ...
    }
}
```

### Step 4: Default for Ergonomics

Provide a convenience function that uses the default runtime's timer, so users don't have to pass it explicitly.

```rust
/// Debounces a stream using the default Tokio timer.
pub fn debounce<S, T>(
    stream: S,
    duration: Duration,
) -> impl Stream<Item = StreamItem<T>>
where
    S: Stream<Item = StreamItem<T>>,
    T: Send,
{
    debounce_with_timer(stream, duration, TokioTimer)
}
```

### Step 5: MockTimer for Tests

Create a `MockTimer` that allows manual control over time.

```rust
use std::sync::{Arc, Mutex};
use std::task::Waker;

pub struct MockTimer {
    inner: Arc<Mutex<MockTimerState>>,
}

struct MockTimerState {
    current_time: Instant,
    pending_sleeps: Vec<(Instant, Waker)>, // deadline, waker
}

impl MockTimer {
    pub fn new() -> Self { ... }

    /// Advances the mock clock, waking all sleeps whose deadline has passed.
    pub fn advance(&self, duration: Duration) {
        let mut state = self.inner.lock().unwrap();
        state.current_time += duration;
        state.pending_sleeps.retain(|(deadline, waker)| {
            if *deadline <= state.current_time {
                waker.wake_by_ref();
                false // Remove from pending
            } else {
                true // Keep in pending
            }
        });
    }
}

impl AsyncTimer for MockTimer {
    type Sleep = MockSleep;
    fn sleep(&self, duration: Duration) -> Self::Sleep { ... }
    fn now(&self) -> Instant { ... }
}
```

With this setup, tests look like:

```rust
#[tokio::test]
async fn test_debounce() {
    let mock_timer = MockTimer::new();
    let (tx, stream) = test_channel();
    let mut debounced = debounce_with_timer(stream, Duration::from_millis(500), mock_timer.clone());

    tx.send(value)?;
    mock_timer.advance(Duration::from_millis(500));

    assert_eq!(debounced.next().await, Some(value));
}
```

**No `yield_now()`, no `pause()`, no race conditions.** The `MockTimer` is explicit, synchronous state.

---

## Summary of Trade-offs

| Approach                 | Performance Impact | Code Readability | Test Determinism | Multi-Runtime |
|--------------------------|--------------------|------------------|------------------|---------------|
| Real Time                | N/A (unusable)     | High             | None             | Yes           |
| Simulated (Tokio pause)  | None               | High             | Medium (races)   | **No**        |
| Abstracted (DI + Mock)   | None (monomorphic) | Medium           | **Full**         | **Yes**       |

---

## Feature-Gated Runtime Implementations

When supporting multiple runtimes, each timer implementation should be gated behind a Cargo feature to avoid pulling in unnecessary dependencies.

### Cargo.toml Configuration

```toml
[features]
default = ["tokio-runtime"]
tokio-runtime = ["tokio"]
async-std-runtime = ["async-std"]
smol-runtime = ["smol"]

[dependencies]
tokio = { version = "1", features = ["time"], optional = true }
async-std = { version = "1", optional = true }
smol = { version = "2", optional = true }
```

### Conditional Compilation

```rust
// The trait is always available
pub trait AsyncTimer: Send + Sync + 'static {
    type Sleep: Future<Output = ()> + Send;
    fn sleep(&self, duration: Duration) -> Self::Sleep;
}

// Each implementation is conditionally compiled
#[cfg(feature = "tokio-runtime")]
pub struct TokioTimer;

#[cfg(feature = "tokio-runtime")]
impl AsyncTimer for TokioTimer { ... }

#[cfg(feature = "async-std-runtime")]
pub struct AsyncStdTimer;

#[cfg(feature = "async-std-runtime")]
impl AsyncTimer for AsyncStdTimer { ... }
```

**Benefits:**
1.  **No bloat**: Users only compile the runtime they use.
2.  **No conflicts**: Avoids pulling in multiple async runtimes simultaneously.
3.  **Clear API**: Default convenience functions use whichever runtime feature is enabled.

This is the standard pattern used by ecosystem crates like `reqwest`, `sqlx`, and `tonic`.

---

## Parameterized Tests Across Runtimes

When the same test logic needs to run against multiple runtimes, several strategies are available:

### Option 1: Macros (Most Common)

Define test logic once, then invoke it for each runtime via a macro:

```rust
macro_rules! runtime_tests {
    ($runtime_attr:meta, $timer:expr) => {
        #[$runtime_attr]
        async fn test_debounce_emits_after_quiet_period() {
            let timer = $timer;
            // ... test body using `timer` ...
        }

        #[$runtime_attr]
        async fn test_debounce_resets_on_new_value() {
            let timer = $timer;
            // ... test body ...
        }
    };
}

#[cfg(feature = "tokio-runtime")]
mod tokio_tests {
    use super::*;
    runtime_tests!(tokio::test, TokioTimer);
}

#[cfg(feature = "async-std-runtime")]
mod async_std_tests {
    use super::*;
    runtime_tests!(async_std::test, AsyncStdTimer);
}
```

| Aspect      | Details                                                        |
|-------------|----------------------------------------------------------------|
| **Pros**    | No extra dependencies; works with any test harness.            |
| **Cons**    | Macro syntax can be verbose; test names include module prefix. |

---

### Option 2: `rstest` with Features

Use [`rstest`](https://crates.io/crates/rstest) for parameterized tests:

```rust
use rstest::rstest;

#[rstest]
#[cfg_attr(feature = "tokio-runtime", case::tokio(TokioTimer))]
#[cfg_attr(feature = "async-std-runtime", case::async_std(AsyncStdTimer))]
#[tokio::test] // or use a runtime-agnostic executor
async fn test_debounce<T: AsyncTimer>(#[case] timer: T) {
    // ... test body ...
}
```

| Aspect      | Details                                                                    |
|-------------|----------------------------------------------------------------------------|
| **Pros**    | Clean, declarative syntax.                                                 |
| **Cons**    | Requires choosing one `#[..::test]` attribute (or using a wrapper crate).  |

---

### Option 3: Generic Test Functions + Per-Runtime Modules

Write test logic as a generic async function, then call it from runtime-specific test modules:

```rust
// tests/common.rs
pub async fn debounce_test_logic<T: AsyncTimer>(timer: T) {
    // ... all assertions here ...
}

// tests/tokio_tests.rs
#[cfg(feature = "tokio-runtime")]
mod tokio {
    #[tokio::test]
    async fn test_debounce() {
        super::super::common::debounce_test_logic(TokioTimer).await;
    }
}

// tests/async_std_tests.rs
#[cfg(feature = "async-std-runtime")]
mod async_std {
    #[async_std::test]
    async fn test_debounce() {
        super::super::common::debounce_test_logic(AsyncStdTimer).await;
    }
}
```

| Aspect      | Details                                           |
|-------------|---------------------------------------------------|
| **Pros**    | No macros; simple function calls; easy to debug.  |
| **Cons**    | Boilerplate for each test invocation.             |

---

### Recommendation for Multi-Runtime Testing

For a multi-runtime library, **Option 1 (Macros)** or **Option 3 (Generic functions)** are the most pragmatic:

-   They require no extra dependencies.
-   They work with each runtime's native `#[..::test]` attribute.
-   They scale well as more operators are added.

The macro approach is more DRY, while the generic function approach is more explicit and easier to debug.

---

## Assessment: Is the Current Approach Sufficient?

### Current State

Fluxion currently uses **Simulated Time (Tokio pause)** for time-based operator tests. We have stabilized the tests by:

1.  Using explicit `tokio::time::pause()` instead of `start_paused = true` where needed.
2.  Adding `tokio::task::yield_now()` calls to synchronize with spawned tasks.

### Is It Good Enough?

**Yes, for the current phase.**

| Criterion                     | Assessment                                                                 |
|-------------------------------|----------------------------------------------------------------------------|
| **Tokio-Only Support**        | ✅ The current approach is appropriate for a Tokio-only library.            |
| **Test Stability**            | ✅ Tests pass reliably with `pause()` and `yield_now()`.                    |
| **Production Performance**    | ✅ No test infrastructure affects production code.                          |
| **Code Readability**          | ✅ Production code has no timer generics; tests are straightforward.        |

### When to Refactor

The refactoring to **Abstracted Time** should be prioritized when:

1.  **Multi-runtime support** (`async-std`, `smol`, WASM) is added to the roadmap.
2.  The number of time-based operators grows, making `yield_now()` boilerplate unacceptable.
3.  Flaky tests become a recurring CI issue.

Until then, the current approach represents a reasonable trade-off between engineering investment and immediate needs.
