// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::types::{CombinedState, WithPrevious};

// Simple test wrapper for Timestamped trait tests
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TestItem<T> {
    value: T,
    timestamp: u64,
}

impl<T> TestItem<T> {
    fn new(value: T, timestamp: u64) -> Self {
        Self { value, timestamp }
    }
}

impl<T: Clone + Send + Sync + 'static> HasTimestamp for TestItem<T> {
    type Timestamp = u64;

    fn timestamp(&self) -> Self::Timestamp {
        self.timestamp
    }
}

impl<T: Clone + Send + Sync + 'static> Timestamped for TestItem<T> {
    type Inner = T;

    fn with_timestamp(value: Self::Inner, timestamp: Self::Timestamp) -> Self {
        Self { value, timestamp }
    }

    fn with_fresh_timestamp(value: Self::Inner) -> Self {
        Self {
            value,
            timestamp: 0, // Simple implementation for testing
        }
    }

    fn into_inner(self) -> Self::Inner {
        self.value
    }
}

// WithPrevious tests

#[test]
fn test_with_previous_new() {
    let wp = WithPrevious::new(Some(10), 20);
    assert_eq!(wp.previous, Some(10));
    assert_eq!(wp.current, 20);
}

#[test]
fn test_with_previous_new_no_previous() {
    let wp: WithPrevious<i32> = WithPrevious::new(None, 42);
    assert_eq!(wp.previous, None);
    assert_eq!(wp.current, 42);
}

#[test]
fn test_with_previous_has_previous_true() {
    let wp = WithPrevious::new(Some("old"), "new");
    assert!(wp.has_previous());
}

#[test]
fn test_with_previous_has_previous_false() {
    let wp: WithPrevious<String> = WithPrevious::new(None, "first".to_string());
    assert!(!wp.has_previous());
}

#[test]
fn test_with_previous_as_pair_some() {
    let wp = WithPrevious::new(Some(100), 200);
    let pair = wp.as_pair();
    assert!(pair.is_some());
    let (prev, curr) = pair.unwrap();
    assert_eq!(*prev, 100);
    assert_eq!(*curr, 200);
}

#[test]
fn test_with_previous_as_pair_none() {
    let wp: WithPrevious<i32> = WithPrevious::new(None, 42);
    let pair = wp.as_pair();
    assert!(pair.is_none());
}

#[test]
fn test_with_previous_clone() {
    let wp = WithPrevious::new(Some(vec![1, 2]), vec![3, 4]);
    let cloned = wp.clone();
    assert_eq!(wp, cloned);
}

#[test]
fn test_with_previous_debug() {
    let wp = WithPrevious::new(Some(5), 10);
    let debug_str = format!("{:?}", wp);
    assert!(debug_str.contains("previous"));
    assert!(debug_str.contains("current"));
}

#[test]
fn test_with_previous_partial_eq() {
    let wp1 = WithPrevious::new(Some(1), 2);
    let wp2 = WithPrevious::new(Some(1), 2);
    let wp3 = WithPrevious::new(Some(1), 3);

    assert_eq!(wp1, wp2);
    assert_ne!(wp1, wp3);
}

#[test]
fn test_with_previous_ord() {
    let wp1 = WithPrevious::new(Some(1), 2);
    let wp2 = WithPrevious::new(Some(1), 3);
    let wp3 = WithPrevious::new(Some(2), 1);

    assert!(wp1 < wp2);
    assert!(wp2 < wp3);
}

// WithPrevious Timestamped implementation tests

#[test]
fn test_with_previous_timestamped_timestamp() {
    let current = TestItem::new(100, 42u64);
    let wp = WithPrevious::new(None, current);

    assert_eq!(wp.timestamp(), 42u64);
}

#[test]
fn test_with_previous_timestamped_with_timestamp() {
    let wp = WithPrevious::<TestItem<i32>>::with_timestamp(999, 123u64);

    assert_eq!(wp.previous, None);
    assert_eq!(wp.current.timestamp(), 123u64);
    assert_eq!(wp.current.into_inner(), 999);
}

#[test]
fn test_with_previous_timestamped_with_fresh_timestamp() {
    let wp = WithPrevious::<TestItem<String>>::with_fresh_timestamp("test".to_string());

    assert_eq!(wp.previous, None);
    assert_eq!(wp.current.into_inner(), "test");
}

#[test]
fn test_with_previous_timestamped_into_inner() {
    let current = TestItem::new(42, 100u64);
    let previous = TestItem::new(10, 50u64);
    let wp = WithPrevious::new(Some(previous), current);

    let inner = wp.into_inner();
    assert_eq!(inner, 42);
}

// CombinedState tests

#[test]
fn test_combined_state_new() {
    let state = CombinedState::new(vec![(1, 100u64), (2, 100u64), (3, 100u64)], 100u64);
    assert_eq!(state.values(), vec![1, 2, 3]);
    assert_eq!(state.timestamp(), 100u64);
}

#[test]
fn test_combined_state_new_empty() {
    let state: CombinedState<i32, u64> = CombinedState::new(vec![], 0);
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_combined_state_values() {
    let state = CombinedState::new(
        vec![
            ("a".to_string(), 42u64),
            ("b".to_string(), 42u64),
            ("c".to_string(), 42u64),
        ],
        42u64,
    );
    assert_eq!(
        state.values(),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}

#[test]
fn test_combined_state_len() {
    let state = CombinedState::new(vec![(10, 1u64), (20, 1u64), (30, 1u64), (40, 1u64)], 1u64);
    assert_eq!(state.len(), 4);
}

#[test]
fn test_combined_state_len_empty() {
    let state: CombinedState<i32, u64> = CombinedState::new(vec![], 0);
    assert_eq!(state.len(), 0);
}

#[test]
fn test_combined_state_is_empty_false() {
    let state = CombinedState::new(vec![(1, 0u64)], 0u64);
    assert!(!state.is_empty());
}

#[test]
fn test_combined_state_is_empty_true() {
    let state: CombinedState<String, u64> = CombinedState::new(vec![], 0);
    assert!(state.is_empty());
}

#[test]
fn test_combined_state_clone() {
    let state = CombinedState::new(vec![(vec![1, 2], 99u64), (vec![3, 4], 99u64)], 99u64);
    let cloned = state.clone();
    assert_eq!(state, cloned);
}

#[test]
fn test_combined_state_debug() {
    let state = CombinedState::new(vec![(1, 50u64), (2, 50u64), (3, 50u64)], 50u64);
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("state"));
    assert!(debug_str.contains("timestamp"));
}

#[test]
fn test_combined_state_partial_eq() {
    let state1 = CombinedState::new(vec![(1, 10u64), (2, 10u64)], 10u64);
    let state2 = CombinedState::new(vec![(1, 10u64), (2, 10u64)], 10u64);
    let state3 = CombinedState::new(vec![(1, 20u64), (2, 20u64)], 20u64);

    assert_eq!(state1, state2);
    assert_ne!(state1, state3);
}

#[test]
fn test_combined_state_ord() {
    let state1 = CombinedState::new(vec![(1, 10u64)], 10u64);
    let state2 = CombinedState::new(vec![(1, 20u64)], 20u64);
    let state3 = CombinedState::new(vec![(2, 10u64)], 10u64);

    assert!(state1 < state2);
    assert!(state1 < state3);
}

// CombinedState Timestamped implementation tests

#[test]
fn test_combined_state_timestamped_timestamp() {
    let state = CombinedState::new(vec![(10, 999u64), (20, 999u64), (30, 999u64)], 999u64);
    assert_eq!(state.timestamp(), 999u64);
}

#[test]
fn test_combined_state_timestamped_with_timestamp() {
    let original = CombinedState::new(vec![(1, 100u64), (2, 100u64), (3, 100u64)], 100u64);
    let new_state = CombinedState::with_timestamp(original, 200u64);

    assert_eq!(new_state.values(), vec![1, 2, 3]);
    assert_eq!(new_state.timestamp(), 200u64);
}

#[test]
fn test_combined_state_timestamped_with_fresh_timestamp() {
    let original = CombinedState::new(vec![("a", 100u64), ("b", 100u64)], 100u64);
    let new_state = CombinedState::with_fresh_timestamp(original.clone());

    // with_fresh_timestamp recycles the timestamp for CombinedState
    assert_eq!(new_state.values(), original.values());
    assert_eq!(new_state.timestamp(), 100u64);
}

#[test]
fn test_combined_state_timestamped_into_inner() {
    let state = CombinedState::new(vec![(42, 555u64), (84, 555u64)], 555u64);
    let cloned = state.clone();
    let inner = state.into_inner();

    assert_eq!(inner, cloned);
}

// Integration tests combining both types

#[test]
fn test_with_previous_containing_combined_state() {
    let state1 = CombinedState::new(vec![(1, 100u64), (2, 100u64)], 100u64);
    let state2 = CombinedState::new(vec![(3, 200u64), (4, 200u64)], 200u64);

    let wp = WithPrevious::new(Some(state1.clone()), state2.clone());

    assert!(wp.has_previous());
    let (prev, curr) = wp.as_pair().unwrap();
    assert_eq!(prev.values(), vec![1, 2]);
    assert_eq!(curr.values(), vec![3, 4]);
}

#[test]
fn test_with_previous_combined_state_timestamp() {
    let state = CombinedState::new(vec![(10, 333u64)], 333u64);
    let wp = WithPrevious::new(None, state);

    // Timestamp comes from current CombinedState
    assert_eq!(wp.timestamp(), 333u64);
}

#[test]
fn test_combined_state_mutation_through_with_previous() {
    let initial = CombinedState::new(vec![(1, 10u64), (2, 10u64), (3, 10u64)], 10u64);
    let updated = CombinedState::new(vec![(4, 20u64), (5, 20u64), (6, 20u64)], 20u64);

    let wp = WithPrevious::new(Some(initial), updated.clone());

    let inner = wp.into_inner();
    assert_eq!(inner, updated);
}

// Edge case tests

#[test]
fn test_combined_state_single_element() {
    let state = CombinedState::new(vec![(42, 1u64)], 1u64);
    assert_eq!(state.len(), 1);
    assert!(!state.is_empty());
    assert_eq!(state.values()[0], 42);
}

#[test]
fn test_with_previous_same_values() {
    let wp = WithPrevious::new(Some(5), 5);
    assert!(wp.has_previous());
    let (prev, curr) = wp.as_pair().unwrap();
    assert_eq!(prev, curr);
}

#[test]
fn test_combined_state_large_vector() {
    let large_vec: Vec<i32> = (0..1000).collect();
    let pairs: Vec<(i32, u64)> = large_vec.iter().map(|&v| (v, 0u64)).collect();
    let state = CombinedState::new(pairs, 0u64);
    assert_eq!(state.len(), 1000);
    assert_eq!(state.values(), large_vec);
}

#[test]
fn test_with_previous_string_types() {
    let wp = WithPrevious::new(Some("previous".to_string()), "current".to_string());

    assert_eq!(wp.previous.as_ref().unwrap(), "previous");
    assert_eq!(&wp.current, "current");
}

#[test]
fn test_combined_state_with_complex_types() {
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Complex {
        id: u32,
        name: String,
    }

    let items = vec![
        Complex {
            id: 1,
            name: "first".to_string(),
        },
        Complex {
            id: 2,
            name: "second".to_string(),
        },
    ];

    let pairs: Vec<(Complex, u64)> = items.iter().map(|v| (v.clone(), 100u64)).collect();
    let state = CombinedState::new(pairs, 100u64);
    assert_eq!(state.values(), items);
}

#[test]
fn test_timestamped_trait_bounds_compile() {
    // This test verifies that the trait implementations satisfy the required bounds
    fn requires_timestamped<T: Timestamped>(_value: T) {}

    let wp = WithPrevious::new(None, TestItem::new(42, 100u64));
    requires_timestamped(wp);

    let state = CombinedState::new(vec![(1, 50u64), (2, 50u64), (3, 50u64)], 50u64);
    requires_timestamped(state);
}
