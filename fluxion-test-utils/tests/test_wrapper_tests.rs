// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_test_utils::test_wrapper::TestWrapper;

#[test]
fn test_new_creates_wrapper_with_timestamp() {
    let wrapper = TestWrapper::new(42, 100);
    assert_eq!(wrapper.value(), &42);
    assert_eq!(wrapper.timestamp(), 100);
}

#[test]
fn test_new_with_string() {
    let wrapper = TestWrapper::new("hello".to_string(), 200);
    assert_eq!(wrapper.value(), &"hello".to_string());
    assert_eq!(wrapper.timestamp(), 200);
}

#[test]
fn test_value_returns_reference() {
    let wrapper = TestWrapper::new(vec![1, 2, 3], 50);
    assert_eq!(wrapper.value(), &vec![1, 2, 3]);
}

#[test]
fn test_timestamped_into_inner() {
    let wrapper = TestWrapper::new(42, 100);
    let inner = wrapper.clone().into_inner();
    assert_eq!(inner, 42);
}

#[test]
fn test_timestamped_timestamp() {
    let wrapper = TestWrapper::new("test", 250);
    assert_eq!(wrapper.timestamp(), 250);
}

#[test]
fn test_timestamped_with_timestamp() {
    let _original = TestWrapper::new(42, 100);
    let new_wrapper = TestWrapper::with_timestamp(42, 500);
    assert_eq!(new_wrapper.value(), &42);
    assert_eq!(new_wrapper.timestamp(), 500);
}

#[test]
fn test_clone() {
    let original = TestWrapper::new(42, 100);
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(cloned.value(), &42);
    assert_eq!(cloned.timestamp(), 100);
}

#[test]
fn test_debug_format() {
    let wrapper = TestWrapper::new(42, 100);
    let debug_str = format!("{:?}", wrapper);
    assert!(debug_str.contains("TestWrapper"));
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("100"));
}

#[test]
fn test_partial_eq_same_values() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(42, 100);
    assert_eq!(wrapper1, wrapper2);
}

#[test]
fn test_partial_eq_different_values() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(43, 100);
    assert_ne!(wrapper1, wrapper2);
}

#[test]
fn test_partial_eq_different_timestamps() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(42, 200);
    assert_ne!(wrapper1, wrapper2);
}

#[test]
fn test_ord_by_timestamp_then_value() {
    let wrapper1 = TestWrapper::new(10, 100);
    let wrapper2 = TestWrapper::new(20, 100);
    let wrapper3 = TestWrapper::new(10, 200);

    // Same timestamp, different values - ordered by value
    assert!(wrapper1 < wrapper2);

    // Different timestamps - ordered by timestamp
    assert!(wrapper1 < wrapper3);
    assert!(wrapper2 < wrapper3);
}

#[test]
fn test_partial_ord() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(42, 200);

    assert!(wrapper1 < wrapper2);
    assert!(wrapper2 > wrapper1);
    assert!(wrapper1 <= wrapper1);
    assert!(wrapper1 >= wrapper1);
}

#[test]
fn test_ord_with_same_wrapper() {
    let wrapper = TestWrapper::new(42, 100);
    assert_eq!(wrapper.cmp(&wrapper), std::cmp::Ordering::Equal);
}

#[test]
fn test_sorting_by_timestamp() {
    let mut wrappers = [
        TestWrapper::new(1, 300),
        TestWrapper::new(2, 100),
        TestWrapper::new(3, 200),
    ];

    wrappers.sort();

    assert_eq!(wrappers[0].timestamp(), 100);
    assert_eq!(wrappers[1].timestamp(), 200);
    assert_eq!(wrappers[2].timestamp(), 300);
}

#[test]
fn test_with_complex_type() {
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Person {
        name: String,
        age: u32,
    }

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let wrapper = TestWrapper::new(person.clone(), 500);
    assert_eq!(wrapper.value(), &person);
    assert_eq!(wrapper.timestamp(), 500);
}

#[test]
fn test_with_timestamp_preserves_value() {
    let _original = TestWrapper::new("test".to_string(), 100);
    let updated = TestWrapper::with_timestamp("test".to_string(), 200);

    assert_eq!(updated.value(), "test");
    assert_eq!(updated.timestamp(), 200);
}

#[test]
fn test_clone_independence() {
    let original = TestWrapper::new(vec![1, 2, 3], 100);
    let cloned = original.clone();

    // Create a modified version
    let modified = TestWrapper::with_timestamp(vec![4, 5, 6], 200);

    // Original and first clone should be unchanged
    assert_eq!(original.value(), &vec![1, 2, 3]);
    assert_eq!(original.timestamp(), 100);
    assert_eq!(cloned.value(), &vec![1, 2, 3]);
    assert_eq!(cloned.timestamp(), 100);
    assert_eq!(modified.value(), &vec![4, 5, 6]);
    assert_eq!(modified.timestamp(), 200);
}

#[test]
fn test_eq_trait() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(42, 100);
    let wrapper3 = TestWrapper::new(42, 200);

    assert!(wrapper1.eq(&wrapper2));
    assert!(!wrapper1.eq(&wrapper3));
}

#[test]
fn test_multiple_timestamps() {
    let wrapper1 = TestWrapper::new(42, 0);
    let wrapper2 = TestWrapper::new(42, u64::MAX);

    assert_eq!(wrapper1.timestamp(), 0);
    assert_eq!(wrapper2.timestamp(), u64::MAX);
    assert!(wrapper1 < wrapper2);
}

#[test]
fn test_value_reference_lifetime() {
    let wrapper = TestWrapper::new(42, 100);
    let value_ref = wrapper.value();
    assert_eq!(*value_ref, 42);
    // value_ref should still be valid here
    assert_eq!(*value_ref, 42);
}

#[test]
fn test_with_option_type() {
    let wrapper = TestWrapper::new(Some(42), 100);
    assert_eq!(wrapper.value(), &Some(42));
    assert_eq!(wrapper.timestamp(), 100);

    let wrapper_none: TestWrapper<Option<i32>> = TestWrapper::new(None, 200);
    assert_eq!(wrapper_none.value(), &None);
    assert_eq!(wrapper_none.timestamp(), 200);
}

#[test]
fn test_with_result_type() {
    let wrapper_ok: TestWrapper<Result<i32, String>> = TestWrapper::new(Ok(42), 100);
    assert_eq!(wrapper_ok.value(), &Ok(42));

    let wrapper_err: TestWrapper<Result<i32, String>> =
        TestWrapper::new(Err("error".to_string()), 200);
    assert_eq!(wrapper_err.value(), &Err("error".to_string()));
}

#[test]
fn test_ordering_transitivity() {
    let wrapper1 = TestWrapper::new(10, 100);
    let wrapper2 = TestWrapper::new(20, 200);
    let wrapper3 = TestWrapper::new(30, 300);

    assert!(wrapper1 < wrapper2);
    assert!(wrapper2 < wrapper3);
    assert!(wrapper1 < wrapper3); // Transitivity
}

#[test]
fn test_equality_reflexive() {
    let wrapper = TestWrapper::new(42, 100);
    assert_eq!(wrapper, wrapper);
}

#[test]
fn test_equality_symmetric() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(42, 100);

    assert_eq!(wrapper1, wrapper2);
    assert_eq!(wrapper2, wrapper1);
}

#[test]
fn test_equality_transitive() {
    let wrapper1 = TestWrapper::new(42, 100);
    let wrapper2 = TestWrapper::new(42, 100);
    let wrapper3 = TestWrapper::new(42, 100);

    assert_eq!(wrapper1, wrapper2);
    assert_eq!(wrapper2, wrapper3);
    assert_eq!(wrapper1, wrapper3);
}

#[test]
fn test_zero_timestamp() {
    let wrapper = TestWrapper::new("zero", 0);
    assert_eq!(wrapper.timestamp(), 0);
}

#[test]
fn test_max_timestamp() {
    let wrapper = TestWrapper::new("max", u64::MAX);
    assert_eq!(wrapper.timestamp(), u64::MAX);
}

#[test]
fn test_into_inner_consumes_wrapper() {
    let wrapper = TestWrapper::new(42, 100);
    let inner = wrapper.into_inner();
    // wrapper is consumed, can't use it anymore
    assert_eq!(inner, 42);
}

#[test]
fn test_with_unit_type() {
    let wrapper = TestWrapper::new((), 100);
    assert_eq!(wrapper.value(), &());
    assert_eq!(wrapper.timestamp(), 100);
}

#[test]
fn test_with_tuple_type() {
    let wrapper = TestWrapper::new((1, "hello", true), 100);
    assert_eq!(wrapper.value(), &(1, "hello", true));
}

#[test]
fn test_nested_wrappers() {
    let inner_wrapper = TestWrapper::new(42, 50);
    let outer_wrapper = TestWrapper::new(inner_wrapper.clone(), 100);

    assert_eq!(outer_wrapper.value(), &inner_wrapper);
    assert_eq!(outer_wrapper.timestamp(), 100);
    assert_eq!(outer_wrapper.value().timestamp(), 50);
}
