# CI/Testing Automation Options

**Status**: Planning / Design Phase
**Goal**: Eliminate discrepancy between local testing scripts and GitHub Actions CI
**Decision Date**: TBD

---

## The Problem

Fluxion currently has **two sources of truth** for testing/CI logic:

1. **GitHub Actions** (`.github/workflows/ci.yml`, `bench.yml`)
   - Runs on push/PR
   - Inline YAML commands

2. **Local PowerShell Scripts** (`.ci/*.ps1`)
   - Run by developers locally
   - Similar but not identical logic

**Issues**:
- ❌ Duplication of testing logic
- ❌ Different commands (nextest vs cargo test)
- ❌ Easy to update one and forget the other
- ❌ "Works locally, fails in CI" scenarios
- ❌ Maintenance burden (2x the work)

---

## Design Goals

1. **Single Source of Truth**: One place defines all test/CI logic
2. **100% Consistency**: Local and CI run identical commands
3. **Cross-Platform**: Works on Linux, macOS, Windows
4. **Easy to Use**: Simple commands for developers
5. **Maintainable**: Easy to add new tasks

---

## Option 1: cargo-xtask ⭐ (Recommended for Complex Projects)

### What It Is
A Rust-based automation tool using a dedicated `xtask` crate in your workspace. This is the **Rust ecosystem standard** used by `cargo`, `rust-analyzer`, and many large projects.

### Implementation

**Structure**:
```
fluxion/
├── xtask/              # New crate
│   ├── Cargo.toml
│   └── src/main.rs     # All CI logic here
├── .github/workflows/
│   └── ci.yml          # Calls: cargo xtask ci
└── .ci/                # DELETE after migration
```

**Example Code** (`xtask/src/main.rs`):
```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run Tokio runtime tests
    TestTokio,
    /// Run Embassy runtime tests (nightly)
    TestEmbassy,
    /// Run all tests
    TestAll,
    /// Full CI validation
    Ci,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::TestTokio => test_tokio(),
        Commands::TestEmbassy => test_embassy(),
        Commands::TestAll => test_all(),
        Commands::Ci => ci_full(),
    }
}

fn test_tokio() -> Result<()> {
    run_cmd("cargo", &["nextest", "run", "--features", "runtime-tokio"])?;
    run_cmd("cargo", &["test", "--doc", "--features", "runtime-tokio"])?;
    Ok(())
}

fn test_embassy() -> Result<()> {
    run_cmd("cargo", &[
        "+nightly", "nextest", "run",
        "--package", "fluxion-stream-time",
        "--features", "runtime-embassy",
        "--no-default-features",
    ])
}

fn ci_full() -> Result<()> {
    run_cmd("cargo", &["fmt", "--check"])?;
    run_cmd("cargo", &["clippy", "--", "-D", "warnings"])?;
    test_all()?;
    Ok(())
}

fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program).args(args).status()?;
    if !status.success() {
        anyhow::bail!("Command failed");
    }
    Ok(())
}
```

**Usage**:
```bash
# Local developer:
cargo xtask test-tokio
cargo xtask test-all
cargo xtask ci

# GitHub Actions (.github/workflows/ci.yml):
- run: cargo xtask ci
```

### Pros
- ✅ 100% consistency (identical binary locally and in CI)
- ✅ Type-safe, compile-time validation
- ✅ No external dependencies (just Cargo)
- ✅ Cross-platform (Rust handles OS differences)
- ✅ IDE support (autocomplete, refactoring)
- ✅ Self-documenting (`cargo xtask --help`)
- ✅ Industry standard for Rust projects

### Cons
- ⚠️ Requires creating new crate
- ⚠️ More initial setup than shell scripts
- ⚠️ Rust compilation time (minor)

### Best For
- ✅ Complex projects (Fluxion qualifies)
- ✅ Teams wanting type safety
- ✅ Long-term maintainability

---

## Option 2: Just ⭐ (Recommended for Simplicity)

### What It Is
Modern command runner (like Make but better). Simple, fast, cross-platform.

**Installation**: `cargo install just`

### Implementation

**Structure**:
```
fluxion/
├── justfile            # Single source of truth
├── .github/workflows/
│   └── ci.yml          # Calls: just ci
└── .ci/                # DELETE
```

**Example** (`justfile`):
```justfile
# justfile

# Run Tokio runtime tests
test-tokio:
    cargo nextest run --workspace --features runtime-tokio
    cargo test --doc --workspace --features runtime-tokio

# Run Embassy runtime tests (nightly)
test-embassy:
    cargo +nightly nextest run \
        --package fluxion-core \
        --package fluxion-stream \
        --package fluxion-stream-time \
        --features runtime-embassy \
        --no-default-features

# Run smol runtime tests
test-smol:
    cargo nextest run --workspace --features runtime-smol --no-default-features
    cargo test --doc --workspace --features runtime-smol

# Run all tests
test-all: test-tokio test-smol test-embassy

# Full CI validation
ci: test-all
    cargo fmt --all --check
    cargo clippy --all-targets --all-features -- -D warnings
    @echo "✅ CI validation complete"
```

**Usage**:
```bash
# Local developer:
just test-tokio
just test-all
just ci

# GitHub Actions:
- run: just ci
```

### Pros
- ✅ Simple, readable syntax
- ✅ Cross-platform (Linux/Mac/Windows)
- ✅ Fast setup (5 minutes)
- ✅ Task dependencies (`test-all: test-tokio test-smol`)
- ✅ Growing adoption in Rust ecosystem
- ✅ Single source of truth

### Cons
- ⚠️ Requires installing `just` (but trivial: `cargo install just`)
- ⚠️ Not pure Rust (but close enough)
- ⚠️ Shell-based (less type safety than xtask)

### Best For
- ✅ Quick wins
- ✅ Teams wanting minimal setup
- ✅ Simple to moderate complexity

---

## Option 3: Makefile (GNU Make)

### Implementation

**Example** (`Makefile`):
```makefile
.PHONY: test-tokio test-embassy test-all ci

test-tokio:
	cargo nextest run --features runtime-tokio
	cargo test --doc --features runtime-tokio

test-embassy:
	cargo +nightly nextest run --features runtime-embassy

test-all: test-tokio test-embassy

ci: test-all
	cargo fmt --check
	cargo clippy -- -D warnings
```

**Usage**: `make test-tokio` or `make ci`

### Pros
- ✅ Universally available (usually pre-installed)
- ✅ Familiar to most developers
- ✅ Zero external dependencies

### Cons
- ❌ Windows support problematic (requires MinGW/WSL)
- ⚠️ Tab vs space issues (error-prone)
- ⚠️ Syntax gets complex quickly
- ⚠️ Not ideal for cross-platform Rust projects

### Best For
- ⚠️ Linux-only projects
- ⚠️ Teams with Make expertise
- ❌ **Not recommended for Fluxion** (Windows support needed)

---

## Option 4: Cargo Aliases

### Implementation

**Example** (`.cargo/config.toml`):
```toml
[alias]
test-tokio = "nextest run --features runtime-tokio"
test-embassy = "+nightly nextest run --features runtime-embassy"
```

**Usage**: `cargo test-tokio`

### Pros
- ✅ No external tools needed
- ✅ Native Cargo integration
- ✅ Simple for basic tasks

### Cons
- ❌ Can't chain commands easily
- ❌ No logic/conditionals
- ❌ Can't run format + clippy + tests together
- ❌ **Too limited for Fluxion's needs**

### Best For
- ⚠️ Very simple projects
- ❌ **Not recommended for Fluxion**

---

## Option 5: cargo-make

### What It Is
Cargo plugin for task automation using TOML configuration.

**Installation**: `cargo install cargo-make`

### Implementation

**Example** (`Makefile.toml`):
```toml
[tasks.test-tokio]
command = "cargo"
args = ["nextest", "run", "--features", "runtime-tokio"]

[tasks.test-embassy]
command = "cargo"
args = ["+nightly", "nextest", "run", "--features", "runtime-embassy"]

[tasks.ci]
dependencies = ["test-tokio", "test-embassy"]
```

**Usage**: `cargo make test-tokio`

### Pros
- ✅ Designed specifically for Rust
- ✅ Cross-platform
- ✅ Powerful features

### Cons
- ⚠️ Requires installing cargo-make
- ⚠️ TOML syntax can be verbose
- ⚠️ Less popular than Just or xtask
- ⚠️ Overlaps with xtask (choose one or the other)

### Best For
- ⚠️ Teams preferring TOML over Rust code
- ⚠️ Projects needing task automation but not xtask

---

## Comparison Matrix

| Solution | Setup | Complexity | Power | Type Safety | Adoption | Cross-Platform |
|----------|-------|------------|-------|-------------|----------|----------------|
| **xtask** | New crate | Medium | High | ✅ | Industry standard | ✅ |
| **Just** | Install tool | Low | Medium | ❌ | Growing | ✅ |
| **Make** | Built-in | Low-Med | Medium | ❌ | Universal | ⚠️ Windows pain |
| **Cargo aliases** | Zero | Very Low | Low | ❌ | Built-in | ✅ |
| **cargo-make** | Install tool | Medium | High | ❌ | Niche | ✅ |

---

## Recommendation for Fluxion

Given Fluxion's requirements:
- ✅ Multiple runtimes (Tokio, smol, Embassy, WASM, async-std)
- ✅ Complex test matrices
- ✅ Feature gating validation
- ✅ Benchmark runs
- ✅ Cross-platform support needed (Windows/Linux/macOS)

### Primary Recommendation: **xtask** or **Just**

**Choose xtask if:**
- You value type safety and compile-time validation
- You want zero external dependencies
- You prefer Rust code over shell-like syntax
- Long-term maintainability is top priority

**Choose Just if:**
- You want fast setup (< 30 minutes)
- Simple, readable syntax is preferred
- You're okay with one external tool (`cargo install just`)
- Quick wins are more important than perfect purity

**Both eliminate the PowerShell/YAML inconsistency completely.**

### NOT Recommended:
- ❌ **Make** - Windows support is problematic
- ❌ **Cargo aliases** - Too limited for complex needs
- ⚠️ **cargo-make** - Less popular, overlaps with xtask

---

## Migration Path

### If Choosing xtask:

1. Create `xtask` crate
2. Add to workspace in root `Cargo.toml`
3. Port `.ci/*.ps1` logic to Rust functions
4. Test locally: `cargo xtask test-all`
5. Update `ci.yml` to call `cargo xtask ci`
6. Verify CI passes
7. Delete `.ci/*.ps1` scripts

**Estimated effort**: 4-6 hours

### If Choosing Just:

1. Install Just: `cargo install just`
2. Create `justfile` in root
3. Port `.ci/*.ps1` logic to Just recipes
4. Test locally: `just test-all`
5. Update `ci.yml` to call `just ci`
6. Verify CI passes
7. Delete `.ci/*.ps1` scripts

**Estimated effort**: 1-2 hours

---

## Implementation Examples

### Example: GitHub Actions with xtask

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: dtolnay/rust-toolchain@nightly

      - name: Run CI validation
        run: cargo xtask ci
```

### Example: GitHub Actions with Just

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: dtolnay/rust-toolchain@nightly

      - name: Install Just
        run: cargo install just

      - name: Run CI validation
        run: just ci
```

---

## Decision Criteria

| Criterion | xtask | Just | Winner |
|-----------|-------|------|--------|
| **Setup Time** | 4-6 hours | 1-2 hours | Just |
| **Type Safety** | ✅ | ❌ | xtask |
| **Maintainability** | ✅ | ✅ | Tie |
| **No External Deps** | ✅ | ❌ (needs `just`) | xtask |
| **Simplicity** | Medium | High | Just |
| **Rust Ecosystem Fit** | ✅ Standard | ✅ Growing | Tie |
| **Cross-Platform** | ✅ | ✅ | Tie |

**Verdict**: Both are excellent choices. Pick based on team preference.

---

## Further Reading

- [cargo-xtask pattern](https://github.com/matklad/cargo-xtask)
- [Just documentation](https://github.com/casey/just)
- [Examples in rust-analyzer](https://github.com/rust-lang/rust-analyzer/tree/master/xtask)

---

## Related Documents

- [ROADMAP.md](../ROADMAP.md) - Feature roadmap
- [FUTURE_ARCHITECTURE.md](FUTURE_ARCHITECTURE.md) - Runtime abstraction plans

---

*Document Status: Planning phase, decision pending*
*Last Updated: January 13, 2026*
