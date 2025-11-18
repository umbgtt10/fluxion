// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, Ordered, OrderedWrapper, StreamItem};
use std::cmp::Ordering;

#[test]
fn test_stream_item_value_creation() {
    let item: StreamItem<i32> = StreamItem::Value(42);
    assert!(item.is_value());
    assert!(!item.is_error());
}

#[test]
fn test_stream_item_error_creation() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test error"));
    assert!(!item.is_value());
    assert!(item.is_error());
}

#[test]
fn test_stream_item_ok_extracts_value() {
    let item = StreamItem::Value(42);
    assert_eq!(item.ok(), Some(42));
}

#[test]
fn test_stream_item_ok_discards_error() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    assert_eq!(item.ok(), None);
}

#[test]
fn test_stream_item_err_extracts_error() {
    let error = FluxionError::stream_error("test error");
    let item: StreamItem<i32> = StreamItem::Error(error.clone());

    let extracted = item.err();
    assert!(extracted.is_some());
}

#[test]
fn test_stream_item_err_discards_value() {
    let item = StreamItem::Value(42);
    assert!(item.err().is_none());
}

#[test]
fn test_stream_item_map_transforms_value() {
    let item = StreamItem::Value(5);
    let mapped = item.map(|x| x * 2);
    assert_eq!(mapped.ok(), Some(10));
}

#[test]
fn test_stream_item_map_propagates_error() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let mapped = item.map(|x| x * 2);
    assert!(mapped.is_error());
}

#[test]
fn test_stream_item_map_type_transformation() {
    let item = StreamItem::Value(42);
    let mapped = item.map(|x| x.to_string());
    assert_eq!(mapped.ok(), Some("42".to_string()));
}

#[test]
fn test_stream_item_and_then_chains_success() {
    let item = StreamItem::Value(5);
    let result = item.and_then(|x| StreamItem::Value(x * 2));
    assert_eq!(result.ok(), Some(10));
}

#[test]
fn test_stream_item_and_then_propagates_initial_error() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("initial"));
    let result = item.and_then(|x| StreamItem::Value(x * 2));
    assert!(result.is_error());
}

#[test]
fn test_stream_item_and_then_propagates_chained_error() {
    let item = StreamItem::Value(5);
    let result = item.and_then(|_| StreamItem::<i32>::Error(FluxionError::stream_error("chained")));
    assert!(result.is_error());
}

#[test]
fn test_stream_item_unwrap_returns_value() {
    let item = StreamItem::Value(42);
    assert_eq!(item.unwrap(), 42);
}

#[test]
#[should_panic(expected = "called `StreamItem::unwrap()` on an `Error` value")]
fn test_stream_item_unwrap_panics_on_error() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let _ = item.unwrap();
}

#[test]
fn test_stream_item_expect_returns_value() {
    let item = StreamItem::Value(42);
    assert_eq!(item.expect("should be a value"), 42);
}

#[test]
#[should_panic(expected = "Expected a value")]
fn test_stream_item_expect_panics_with_message() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let _ = item.expect("Expected a value");
}

#[test]
fn test_stream_item_from_result_ok() {
    let result: Result<i32, FluxionError> = Ok(42);
    let item: StreamItem<i32> = result.into();
    assert_eq!(item.ok(), Some(42));
}

#[test]
fn test_stream_item_from_result_err() {
    let result: Result<i32, FluxionError> = Err(FluxionError::stream_error("test"));
    let item: StreamItem<i32> = result.into();
    assert!(item.is_error());
}

#[test]
fn test_stream_item_into_result_value() {
    let item = StreamItem::Value(42);
    let result: Result<i32, FluxionError> = item.into();
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_stream_item_into_result_error() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let result: Result<i32, FluxionError> = item.into();
    assert!(result.is_err());
}

#[test]
fn test_stream_item_eq_same_values() {
    let item1 = StreamItem::Value(42);
    let item2 = StreamItem::Value(42);
    assert_eq!(item1, item2);
}

#[test]
fn test_stream_item_eq_different_values() {
    let item1 = StreamItem::Value(42);
    let item2 = StreamItem::Value(43);
    assert_ne!(item1, item2);
}

#[test]
fn test_stream_item_eq_errors_never_equal() {
    let item1: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    assert_ne!(item1, item2);
}

#[test]
fn test_stream_item_eq_value_not_equal_error() {
    let item1 = StreamItem::Value(42);
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    assert_ne!(item1, item2);
}

#[test]
fn test_stream_item_partial_ord_values() {
    let item1 = StreamItem::Value(10);
    let item2 = StreamItem::Value(20);
    assert!(item1 < item2);
    assert!(item2 > item1);
}

#[test]
fn test_stream_item_partial_ord_errors_equal() {
    let item1: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error2"));
    assert_eq!(item1.partial_cmp(&item2), Some(Ordering::Equal));
}

#[test]
fn test_stream_item_partial_ord_error_less_than_value() {
    let error: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let value = StreamItem::Value(42);
    assert!(error < value);
    assert!(value > error);
}

#[test]
fn test_stream_item_ord_values() {
    let item1 = StreamItem::Value(10);
    let item2 = StreamItem::Value(20);
    assert_eq!(item1.cmp(&item2), Ordering::Less);
    assert_eq!(item2.cmp(&item1), Ordering::Greater);
    assert_eq!(item1.cmp(&item1), Ordering::Equal);
}

#[test]
fn test_stream_item_ord_errors_equal() {
    let item1: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error2"));
    assert_eq!(item1.cmp(&item2), Ordering::Equal);
}

#[test]
fn test_stream_item_ord_error_less_than_value() {
    let error: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let value = StreamItem::Value(42);
    assert_eq!(error.cmp(&value), Ordering::Less);
    assert_eq!(value.cmp(&error), Ordering::Greater);
}

#[test]
fn test_stream_item_clone() {
    let item = StreamItem::Value(42);
    let cloned = item.clone();
    assert_eq!(cloned.ok(), Some(42));
}

#[test]
fn test_stream_item_clone_error() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let cloned = item.clone();
    assert!(cloned.is_error());
}

#[test]
fn test_stream_item_ordered_value() {
    let ordered = OrderedWrapper::with_order(42, 100);
    let item = StreamItem::Value(ordered);

    assert_eq!(item.order(), 100);
    assert_eq!(item.get(), &42);
}

#[test]
fn test_stream_item_ordered_error_has_zero_order() {
    let item: StreamItem<OrderedWrapper<i32>> =
        StreamItem::Error(FluxionError::stream_error("test"));

    assert_eq!(item.order(), 0);
}

#[test]
#[should_panic(expected = "called `get()` on StreamItem::Error")]
fn test_stream_item_ordered_get_panics_on_error() {
    let item: StreamItem<OrderedWrapper<i32>> =
        StreamItem::Error(FluxionError::stream_error("test"));

    let _ = item.get();
}

#[test]
fn test_stream_item_ordered_with_order() {
    let item: StreamItem<OrderedWrapper<i32>> = StreamItem::with_order(42, 200);

    assert_eq!(item.order(), 200);
    assert_eq!(item.get(), &42);
}

// Note: CompareByInner tests are skipped as OrderedWrapper doesn't implement CompareByInner
// These tests would require a custom type that implements both Ordered and CompareByInner
#[test]
fn test_stream_item_chained_map_operations() {
    let item = StreamItem::Value(5);
    let result = item.map(|x| x * 2).map(|x| x + 3).map(|x| x.to_string());

    assert_eq!(result.ok(), Some("13".to_string()));
}

#[test]
fn test_stream_item_chained_and_then_operations() {
    let item = StreamItem::Value(5);
    let result = item
        .and_then(|x| StreamItem::Value(x * 2))
        .and_then(|x| StreamItem::Value(x + 3))
        .and_then(|x| StreamItem::Value(x.to_string()));

    assert_eq!(result.ok(), Some("13".to_string()));
}

#[test]
fn test_stream_item_mixed_map_and_then() {
    let item = StreamItem::Value(5);
    let result = item
        .map(|x| x * 2)
        .and_then(|x| StreamItem::Value(x + 3))
        .map(|x| x.to_string());

    assert_eq!(result.ok(), Some("13".to_string()));
}

#[test]
fn test_stream_item_error_short_circuits_chain() {
    let item = StreamItem::Value(5);
    let result = item
        .map(|x| x * 2)
        .and_then(|_| StreamItem::Error(FluxionError::stream_error("error in chain")))
        .map(|x: i32| x + 100); // This should not execute

    assert!(result.is_error());
}

#[test]
fn test_stream_item_with_custom_type() {
    #[derive(Debug, Clone, PartialEq)]
    struct CustomData {
        value: String,
        count: usize,
    }

    let data = CustomData {
        value: "test".to_string(),
        count: 42,
    };

    let item = StreamItem::Value(data.clone());
    assert_eq!(item.ok(), Some(data));
}

#[test]
fn test_stream_item_map_with_closure_capturing() {
    let multiplier = 10;
    let item = StreamItem::Value(5);
    let result = item.map(|x| x * multiplier);

    assert_eq!(result.ok(), Some(50));
}

#[test]
fn test_stream_item_result_roundtrip() {
    let original = StreamItem::Value(42);
    let as_result: Result<i32, FluxionError> = original.into();
    let back_to_item: StreamItem<i32> = as_result.into();

    assert_eq!(back_to_item.ok(), Some(42));
}

#[test]
fn test_stream_item_error_result_roundtrip() {
    let original: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let as_result: Result<i32, FluxionError> = original.into();
    let back_to_item: StreamItem<i32> = as_result.into();

    assert!(back_to_item.is_error());
}

#[test]
fn test_stream_item_ordering_with_sorting() {
    let mut items = [
        StreamItem::Value(30),
        StreamItem::Value(10),
        StreamItem::Error(FluxionError::stream_error("error")),
        StreamItem::Value(20),
    ];

    items.sort();

    // Errors should come first (Less than values), then values in order
    assert!(items[0].is_error());
    assert_eq!(items[1].clone().ok(), Some(10));
    assert_eq!(items[2].clone().ok(), Some(20));
    assert_eq!(items[3].clone().ok(), Some(30));
}

#[test]
fn test_stream_item_debug_format() {
    let value_item = StreamItem::Value(42);
    let debug_str = format!("{:?}", value_item);
    assert!(debug_str.contains("Value"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_stream_item_error_debug_format() {
    let error_item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test error"));
    let debug_str = format!("{:?}", error_item);
    assert!(debug_str.contains("Error"));
}

#[test]
fn test_stream_item_complex_nested_operations() {
    let process = |x: i32| -> StreamItem<String> {
        if x > 0 {
            StreamItem::Value(format!("positive: {}", x))
        } else {
            StreamItem::Error(FluxionError::stream_error("negative number"))
        }
    };

    let item1 = StreamItem::Value(5);
    let result1 = item1.and_then(process);
    assert_eq!(result1.ok(), Some("positive: 5".to_string()));

    let item2 = StreamItem::Value(-5);
    let result2 = item2.and_then(process);
    assert!(result2.is_error());
}

#[test]
fn test_stream_item_option_like_behavior() {
    let item = StreamItem::Value(42);

    // Can pattern match like Option
    match item {
        StreamItem::Value(v) => assert_eq!(v, 42),
        StreamItem::Error(_) => panic!("Should be a value"),
    }
}

#[test]
fn test_stream_item_error_pattern_matching() {
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    match item {
        StreamItem::Value(_) => panic!("Should be an error"),
        StreamItem::Error(e) => {
            assert!(format!("{:?}", e).contains("test"));
        }
    }
}
