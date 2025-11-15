# fluxion

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

[![CI](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml/badge.svg)](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml)
[![Coverage](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml/badge.svg)](https://github.com/umbgtt10/fluxion/actions/workflows/ci.yml)

Fluxion is a 100% Rust implementation of composite Rx-style stream operators and helpers focused on correct temporal ordering and efficient async processing.

## Quick Start

Add fluxion to your `Cargo.toml`:

```toml
[dependencies]
fluxion = { path = "path/to/fluxion" }
tokio = { version = "1.48", features = ["full"] }
futures = "0.3"
```

Basic usage example:

```rust
use fluxion::FluxionStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Create a channel and stream
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
    let stream = FluxionStream::from_unbounded_receiver(rx);
    
    // Send some values
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    tx.send(3).unwrap();
    drop(tx); // Close the sender
    
    // Collect and print values
    let values: Vec<_> = stream.collect().await;
    println!("Received: {:?}", values); // Prints: Received: [1, 2, 3]
}
```

## Documentation

Fluxion provides comprehensive documentation with conceptual guides and working examples:

### 📚 Getting Started Guides

- **[fluxion-stream](fluxion-stream/)** - Stream composition and temporal ordering
  - Architecture and key concepts
  - Temporal ordering explained
  - Operator selection guide with comparison tables
  - Common patterns with runnable examples
  - Performance characteristics

- **[fluxion-exec](fluxion-exec/)** - Execution patterns and async subscriptions
  - Sequential vs. latest-value processing
  - Task spawning and cancellation
  - Comparisons with other async patterns
  - Error handling best practices

### 🔧 API Documentation

Generate full API docs locally:
```bash
cargo doc --no-deps --open
```

Or browse specific crate docs:
- `cargo doc --package fluxion-stream --open` - Stream operators
- `cargo doc --package fluxion-exec --open` - Execution utilities
- `cargo doc --package fluxion-test-utils --open` - Testing helpers

### 📖 Quick Reference

**Stream Operators:**
- `combine_latest` - Combine multiple streams with latest values
- `with_latest_from` - Sample secondary on primary emission
- `ordered_merge` - Merge preserving temporal order  
- `emit_when` - Gate emissions by filter condition
- `take_latest_when` - Sample on trigger
- `take_while_with` - Emit while condition holds
- `combine_with_previous` - Pair consecutive values

**Execution:**
- `subscribe_async` - Sequential item processing
- `subscribe_latest_async` - Latest-value with auto-cancellation

## Quick links
- Crates: `fluxion-stream`, `fluxion-exec`, `fluxion-test-utils`
- Full docs: `cargo doc --no-deps --open`

## Getting started

Prerequisites: Rust (recommended toolchain is pinned by `rust-toolchain.toml`).

- Build and run tests for the whole workspace:

```powershell
cargo build --all
cargo test --all-features --all-targets
```

- Run the local CI helper (PowerShell):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\.ci\ci.ps1
```

Workspace overview

- `fluxion-stream` — stream combinators and timestamping helpers (ordering guarantees)
- `fluxion-exec` — execution utilities and subscription helpers for async processing
- `fluxion-test-utils` — shared test helpers, fixtures and sample data used by tests

Development notes

- Clippy, formatting and documentation warnings are treated as errors in CI.
- Local scripts live in `.ci/` (use `coverage.ps1` to collect coverage locally with `cargo-llvm-cov`).

TODOs:
- [ ] Enable direct usage of TestChannel in FluxionStream methods:
    - Currently: `FluxionStream::from(person1.stream).ordered_merge(vec![FluxionStream::from(person2.stream)])`
    - Goal: `person1.ordered_merge(vec![person2])`
    - Blocker: Avoid circular dependencies between fluxion-stream and fluxion-test-utils
- [ ] Review all performance benches:
    - Ensure consistency
	- Double-check sample sizes and test duration
	- Ensure reasonable complexity of the benches
	- Consider adding additional cases
- [ ] Investigate a clean way to suppress unit-test harness noise when running `cargo bench`.
	- Consider options: `cargo -q bench`, running the bench binary directly, per-package bench (`-p`), or a `.ci/bench.ps1` wrapper that builds then executes bench executables.
- [ ] Rework/review ci.yml
	- Consider options: add code coverage
	- Improve speed
	- Consider removing unnecessary steps
	- Check whether new steps should be added
- [ ] Windows double-versioning issue:
    - Resolve `windows-sys` multiple-version issue on Windows
	- Symptom: both `windows-sys 0.60.x` and `0.61.x` appear in `Cargo.lock` (clippy flags `multiple_crate_versions`).
	- Options:
	   - bump workspace `tokio`/transitive crates so they align [DONE]
	   - pin `socket2`/`mio` to compatible versions [NOT DONE]
	   - or accept both and suppress the clippy lint in CI. [DONE]
	- Fix / workaround:
       - suppress Clippy at crate level with #![allow(clippy::multiple_crate_versions, clippy::doc_markdown)] [DONE]
       - or add a [patch.crates-io] override for the transitive crate (preferred over forcing windows-sys directly) [NOT DONE]
	- Note: attempting to force windows-sys = "0.61.2" may fail if an upstream crate requires ^0.60; use cargo tree -i to inspect dependency paths before attempting a pin/patch
	- Optimal solution: update tokio / related transitive crates to versions that align their windows-sys reqs, remove any suppressions/pins and re-run cargo update (may require coordinated upgrades).
	- Quick checks:
		```
		cargo tree -i windows-sys@0.60.2 --workspace
		cargo tree -i windows-sys@0.61.2 --workspace
		cargo tree -i windows-sys --workspace
		```
		Use `cargo update` after edits to `Cargo.toml` to refresh `Cargo.lock`.


Issues & contributions

Contributions are welcome. Please open issues and PRs. See [CONTRIBUTING.md](CONTRIBUTING.md) for details. Use the existing CI scripts to verify changes locally before pushing.

License

Apache-2.0
