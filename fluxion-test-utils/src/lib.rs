// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#![allow(clippy::multiple_crate_versions, clippy::doc_markdown)]
pub mod animal;
pub mod helpers;
pub mod person;
pub mod plant;
pub mod sequenced;
pub mod test_data;

// Re-export commonly used test utilities
pub use helpers::assert_no_element_emitted;
pub use sequenced::Sequenced;
pub use test_data::{DataVariant, TestData};
