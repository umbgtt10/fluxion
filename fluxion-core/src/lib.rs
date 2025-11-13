#![allow(clippy::multiple_crate_versions)]
pub mod ordered;

// Re-export commonly used types
pub use ordered::{Ordered, OrderedWrapper};

/// Trait for comparing ordered types by their inner values.
/// This is used by stream operators to establish a stable ordering.
pub trait CompareByInner {
    fn cmp_inner(&self, other: &Self) -> std::cmp::Ordering;
}
