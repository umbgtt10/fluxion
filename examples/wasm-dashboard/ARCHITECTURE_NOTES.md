# Architecture Notes

## Current Design Considerations

### 1. Task Cancellation Token Usage

In `dashboard_updater.rs`, `FluxionTask::spawn` receives a `task_cancel` token but uses the shared `cancel_token` instead:

```rust
FluxionTask::spawn(move |_task_cancel| async move {
    stream.subscribe(..., Some(cancel_token)).await
})
```

**Current behavior:** All tasks share a single cancellation token for coordinated shutdown.

**Future option:** Use the task-specific token for individual task cancellation:
```rust
FluxionTask::spawn(move |task_cancel| async move {
    stream.subscribe(..., Some(task_cancel)).await
})
```

### 2. Subscribe Error Handling

Stream subscription errors are silently discarded:

```rust
let _ = stream.subscribe(...).await;
```

**Current behavior:** UI update handlers return `Infallible`, so no handler errors occur. Stream-level completion/errors are not logged.

**Future option:** Add diagnostic logging for stream completion or errors if debugging is needed.

### 3. Task Storage Timing

Tasks are spawned and stored within `run()`, making them inaccessible from outside until `run()` blocks.

**Current behavior:** Run-to-completion model where the updater runs until cancelled, then drops all tasks.

**Future option:** If external task management is needed, return tasks from wire methods or store them before blocking in `run()`.
