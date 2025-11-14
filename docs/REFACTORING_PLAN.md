# Fluxion Refactoring Plan: Stabilizing Error Handling

## 1. Executive Summary

**Goal:** Bring the Fluxion codebase to a stable, consistent, and robust state by completing the error handling refactor.

This plan replaces the previous `ERROR_HANDLING_PLAN.md`. The immediate priority is to fix the inconsistent error handling that currently exists across the workspace. The current implementation mixes `Result`-based APIs with functions that silently log or swallow errors, making the library unreliable.

This plan focuses on three core activities:
1.  **Complete API-level error propagation.**
2.  **Overhaul tests to validate error paths.**
3.  **Update documentation to reflect the correct behavior.**

---

## 2. Current State Analysis

A full codebase review identified the following key issues:

*   **Incomplete `fluxion-exec` Refactor:** The `subscribe_async` and `subscribe_latest_async` functions are the public face for stream consumption, but they do not propagate errors back to the caller. They have the correct trait bounds for fallible operations but return `()` instead of the planned `Result<()>`. This is the most critical gap.
*   **Silent Failures in `fluxion-stream`:** Operators like `combine_latest` use a `safe_lock` utility to avoid panicking on poisoned mutexes. However, upon failure, they log the error and drop the item, which constitutes a silent failure from the caller's perspective. Errors must be propagated through the stream as `Err` items.
*   **Inconsistent Test Coverage:** While `fluxion-test-utils` has been partially updated (e.g., `push` returns `Result`), most tests still rely on `_unchecked` variants or `unwrap()`. There is a lack of tests that specifically validate error-path behavior.

---

## 3. The Battle Plan

This plan is broken into sequential phases. Each phase must be completed before the next begins to ensure a methodical and stable refactoring process.

### Phase 1: Stabilize `fluxion-exec` (The Public API)

**Objective:** Ensure that the primary stream consumers, `subscribe_async` and `subscribe_latest_async`, correctly report errors back to the caller.

*   **Task 1.1: Modify Signatures to Return `Result<()>`**
    *   Change the return type of `subscribe_async` and `subscribe_latest_async` from `()` to `fluxion_error::Result<()>`.

*   **Task 1.2: Implement Error Aggregation**
    *   In `subscribe_async`, use an `Arc<Mutex<Vec<E>>>` or a similar concurrent structure to collect errors from spawned tasks.
    *   If no `on_error_callback` is provided, store errors in this collection.
    *   At the end of the function, if the collection is not empty, return a `FluxionError::MultipleErrors` (or a `SubscriptionError`) containing the collected errors. Otherwise, return `Ok(())`.
    *   Apply a similar strategy for `subscribe_latest_async`.

*   **Task 1.3: Update `fluxion-exec` Tests**
    *   Modify the tests in `tests/subscribe_async_tests.rs` and `tests/subscribe_latest_async_tests.rs`.
    *   Calls to these functions must now handle the `Result`. Use `.unwrap()` in tests where success is expected.
    *   Add new tests to verify that errors are correctly returned when no `on_error_callback` is provided.

### Phase 2: Enable Error Propagation in `fluxion-stream`

**Objective:** Transform stream operators from silently dropping items on internal errors to propagating them through the stream.

*   **Task 2.1: Change Stream Item Types**
    *   Identify all operators in `fluxion-stream` that can fail (e.g., due to `safe_lock`).
    *   Change their output from `impl Stream<Item = T>` to `impl Stream<Item = Result<T, FluxionError>>`.

*   **Task 2.2: Propagate Errors**
    *   In operators like `combine_latest`, where `safe_lock` is used, if the lock fails, `yield` an `Err(FluxionError::LockError { ... })` instead of logging and returning `None`.
    *   Audit other operators for potential failure points and apply the same pattern.

*   **Task 2.3: Update `fluxion-stream` Tests**
    *   Update all tests for the modified operators to handle a stream of `Result`s.
    *   Add specific tests to trigger a lock error (e.g., via intentional poisoning) and assert that an `Err` is correctly yielded by the stream.

### Phase 3: Documentation and Finalization

**Objective:** Ensure the library is usable and maintainable by providing clear, accurate documentation.

*   **Task 3.1: Create Error Handling Guide**
    *   Create a new document at `docs/error-handling.md`.
    *   Explain the library's error philosophy, the `FluxionError` type, and how to handle `Result`-returning functions and streams.
    *   Provide clear examples for error handling at both the subscriber and operator levels.

*   **Task 3.2: Update All Public API Documentation**
    *   Add detailed `# Errors` sections to the doc comments for every public function that can fail, including all stream operators and subscribers.
    *   Update the crate-level documentation for `fluxion-exec` and `fluxion-stream` to reflect the new error handling model.

*   **Task 3.3: Full Workspace Test and Review**
    *   Run `cargo test --workspace --all-features` and ensure all tests pass.
    *   Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` and fix any new lints.
    *   Manually review the changes to ensure consistency and correctness.

---
I will now begin work on **Phase 1**.
