# Changelog

All notable changes to the Fluxion project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-11-16

### Added
- **Documentation**: Added `docs/FLUXION_OPERATOR_SUMMARY.md` - comprehensive operator reference guide
- **Documentation**: Added `docs/FLUXION_OPERATORS_ROADMAP.md` - planned operators and priorities
- **Documentation**: Added `docs/REFACTORING_PLAN.md` - error handling implementation plan
- **Examples**: Added chaining examples to README showing real-world operator composition
- **Tests**: Created `tests/combine_latest_tests.rs` with functional and composition examples
- **Integration**: Integrated `stream-aggregation` example into workspace members
- **API Docs**: Added comprehensive documentation to all FluxionStream extension methods
- **Code of Conduct**: Added Kingdom of Heaven inspired community guidelines

### Changed
- **README**: Restructured with improved organization and table of contents
- **README**: Enhanced examples section with links to test files for verification
- **README**: Added "Chaining Multiple Operators" section with working code examples
- **CI**: Updated `build.ps1` to include stream-aggregation example in build/test pipeline
- **Formatting**: Fixed clippy warnings and formatting inconsistencies across codebase
- **Imports**: Cleaned up unused imports and resolved import warnings
- **Documentation**: Updated all references from `TestChannel` to `tokio::sync::mpsc`

### Fixed
- Resolved beginner-level API usage mistakes in examples
- Fixed code formatting to match rustfmt standards
- Corrected import statements throughout test files

### Known Issues
- **Dependencies**: Harmless duplicate `windows-sys` versions (0.60.2 and 0.61.2) from tokio ecosystem - does not affect functionality

## [0.1.0] - 2025-11-16

### Added
- First release

---

[Unreleased]: https://github.com/umbgtt10/fluxion/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.1
[0.1.0]: https://github.com/umbgtt10/fluxion/releases/tag/v0.1.0
