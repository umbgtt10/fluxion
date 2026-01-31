// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, StreamItem};
use std::cmp::Ordering;

#[test]
fn test_stream_item_value_creation() {
    // Arrange & Act
    let item: StreamItem<i32> = StreamItem::Value(42);

    // Assert
    assert!(item.is_value());
    assert!(!item.is_error());
}

#[test]
fn test_stream_item_error_creation() {
    // Arrange & Act
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test error"));

    // Assert
    assert!(!item.is_value());
    assert!(item.is_error());
}

#[test]
fn test_stream_item_ok_extracts_value() {
    // Arrange
    let item = StreamItem::Value(42);

    // Act & Assert
    assert_eq!(item.ok(), Some(42));
}

#[test]
fn test_stream_item_ok_discards_error() {
    // Arrange
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Act & Assert
    assert_eq!(item.ok(), None);
}

#[test]
fn test_stream_item_err_extracts_error() {
    // Arrange
    let error = FluxionError::stream_error("test error");
    let item: StreamItem<i32> = StreamItem::Error(error.clone());

    // Act
    let extracted = item.err();

    // Assert
    assert!(extracted.is_some());
}

#[test]
fn test_stream_item_err_discards_value() {
    // Arrange
    let item = StreamItem::Value(42);

    // Act & Assert
    assert!(item.err().is_none());
}

#[test]
fn test_stream_item_map_transforms_value() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let mapped = item.map(|x| x * 2);

    // Assert
    assert_eq!(mapped.ok(), Some(10));
}

#[test]
fn test_stream_item_map_propagates_error() {
    // Arrange
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Act
    let mapped = item.map(|x| x * 2);

    // Assert
    assert!(mapped.is_error());
}

#[test]
fn test_stream_item_map_type_transformation() {
    // Arrange
    let item = StreamItem::Value(42);

    // Act
    let mapped = item.map(|x| x.to_string());

    // Assert
    assert_eq!(mapped.ok(), Some("42".to_string()));
}

#[test]
fn test_stream_item_and_then_chains_success() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let result = item.and_then(|x| StreamItem::Value(x * 2));

    // Assert
    assert_eq!(result.ok(), Some(10));
}

#[test]
fn test_stream_item_and_then_propagates_initial_error() {
    // Arrange
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("initial"));

    // Act
    let result = item.and_then(|x| StreamItem::Value(x * 2));

    // Assert
    assert!(result.is_error());
}

#[test]
fn test_stream_item_and_then_propagates_chained_error() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let result = item.and_then(|_| StreamItem::<i32>::Error(FluxionError::stream_error("chained")));

    // Assert
    assert!(result.is_error());
}

#[test]
fn test_stream_item_unwrap_returns_value() {
    // Arrange
    let item = StreamItem::Value(42);

    // Act & Assert
    assert_eq!(item.unwrap(), 42);
}

#[test]
#[should_panic(expected = "called `StreamItem::unwrap()` on an `Error` value")]
fn test_stream_item_unwrap_panics_on_error() {
    // Arrange & Act
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Assert
    let _ = item.unwrap();
}

#[test]
fn test_stream_item_expect_returns_value() {
    // Arrange
    let item = StreamItem::Value(42);

    // Act & Assert
    assert_eq!(item.expect("should be a value"), 42);
}

#[test]
#[should_panic(expected = "Expected a value")]
fn test_stream_item_expect_panics_with_message() {
    // Arrange & Act
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Assert
    let _ = item.expect("Expected a value");
}

#[test]
fn test_stream_item_from_result_ok() {
    // Arrange
    let result: Result<i32, FluxionError> = Ok(42);

    // Act
    let item: StreamItem<i32> = result.into();

    // Assert
    assert_eq!(item.ok(), Some(42));
}

#[test]
fn test_stream_item_from_result_err() {
    // Arrange
    let result: Result<i32, FluxionError> = Err(FluxionError::stream_error("test"));

    // Act
    let item: StreamItem<i32> = result.into();

    // Assert
    assert!(item.is_error());
}

#[test]
fn test_stream_item_into_result_value() {
    // Arrange
    let item = StreamItem::Value(42);

    // Act
    let result: Result<i32, FluxionError> = item.into();

    // Assert
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_stream_item_into_result_error() {
    // Arrange
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Act
    let result: Result<i32, FluxionError> = item.into();

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_stream_item_eq_same_values() {
    // Arrange & Act
    let item1 = StreamItem::Value(42);
    let item2 = StreamItem::Value(42);

    // Assert
    assert_eq!(item1, item2);
}

#[test]
fn test_stream_item_eq_different_values() {
    // Arrange & Act
    let item1 = StreamItem::Value(42);
    let item2 = StreamItem::Value(43);

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_stream_item_eq_errors_never_equal() {
    // Arrange & Act
    let item1: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_stream_item_eq_value_not_equal_error() {
    // Arrange & Act
    let item1 = StreamItem::Value(42);
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_stream_item_partial_ord_values() {
    // Arrange & Act
    let item1 = StreamItem::Value(10);
    let item2 = StreamItem::Value(20);

    // Assert
    assert!(item1 < item2);
    assert!(item2 > item1);
}

#[test]
fn test_stream_item_partial_ord_errors_equal() {
    // Arrange & Act
    let item1: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error2"));

    // Assert
    assert_eq!(item1.partial_cmp(&item2), Some(Ordering::Equal));
}

#[test]
fn test_stream_item_partial_ord_error_less_than_value() {
    // Arrange & Act
    let error: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let value = StreamItem::Value(42);

    // Assert
    assert!(error < value);
    assert!(value > error);
}

#[test]
fn test_stream_item_ord_values() {
    // Arrange & Act
    let item1 = StreamItem::Value(10);
    let item2 = StreamItem::Value(20);

    // Assert
    assert_eq!(item1.cmp(&item2), Ordering::Less);
    assert_eq!(item2.cmp(&item1), Ordering::Greater);
    assert_eq!(item1.cmp(&item1), Ordering::Equal);
}

#[test]
fn test_stream_item_ord_errors_equal() {
    // Arrange & Act
    let item1: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error1"));
    let item2: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("error2"));

    // Assert
    assert_eq!(item1.cmp(&item2), Ordering::Equal);
}

#[test]
fn test_stream_item_ord_error_less_than_value() {
    // Arrange & Act
    let error: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let value = StreamItem::Value(42);

    // Assert
    assert_eq!(error.cmp(&value), Ordering::Less);
    assert_eq!(value.cmp(&error), Ordering::Greater);
}

#[test]
fn test_stream_item_clone() {
    // Arrange & Act
    let item = StreamItem::Value(42);
    let cloned = item.clone();

    // Assert
    assert_eq!(cloned.ok(), Some(42));
}

#[test]
fn test_stream_item_clone_error() {
    // Arrange & Act
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));
    let cloned = item.clone();

    // Assert
    assert!(cloned.is_error());
}

#[test]
fn test_stream_item_chained_map_operations() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let result = item.map(|x| x * 2).map(|x| x + 3).map(|x| x.to_string());

    // Assert
    assert_eq!(result.ok(), Some("13".to_string()));
}

#[test]
fn test_stream_item_chained_and_then_operations() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let result = item
        .and_then(|x| StreamItem::Value(x * 2))
        .and_then(|x| StreamItem::Value(x + 3))
        .and_then(|x| StreamItem::Value(x.to_string()));

    // Assert
    assert_eq!(result.ok(), Some("13".to_string()));
}

#[test]
fn test_stream_item_mixed_map_and_then() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let result = item
        .map(|x| x * 2)
        .and_then(|x| StreamItem::Value(x + 3))
        .map(|x| x.to_string());

    // Assert
    assert_eq!(result.ok(), Some("13".to_string()));
}

#[test]
fn test_stream_item_error_short_circuits_chain() {
    // Arrange
    let item = StreamItem::Value(5);

    // Act
    let result = item
        .map(|x| x * 2)
        .and_then(|_| StreamItem::Error(FluxionError::stream_error("error in chain")))
        .map(|x: i32| x + 100);

    // Assert
    assert!(result.is_error());
}

#[test]
fn test_stream_item_with_custom_type() {
    // Arrange
    #[derive(Debug, Clone, PartialEq)]
    struct CustomData {
        value: String,
        count: usize,
    }

    let data = CustomData {
        value: "test".to_string(),
        count: 42,
    };

    // Act
    let item = StreamItem::Value(data.clone());

    // Assert
    assert_eq!(item.ok(), Some(data));
}

#[test]
fn test_stream_item_map_with_closure_capturing() {
    // Arrange
    let multiplier = 10;
    let item = StreamItem::Value(5);

    // Act
    let result = item.map(|x| x * multiplier);

    // Assert
    assert_eq!(result.ok(), Some(50));
}

#[test]
fn test_stream_item_result_roundtrip() {
    // Arrange
    let original = StreamItem::Value(42);

    // Act
    let as_result: Result<i32, FluxionError> = original.into();
    let back_to_item: StreamItem<i32> = as_result.into();

    // Assert
    assert_eq!(back_to_item.ok(), Some(42));
}

#[test]
fn test_stream_item_error_result_roundtrip() {
    // Arrange
    let original: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Act
    let as_result: Result<i32, FluxionError> = original.into();
    let back_to_item: StreamItem<i32> = as_result.into();

    // Assert
    assert!(back_to_item.is_error());
}

#[test]
fn test_stream_item_ordering_with_sorting() {
    // Arrange
    let mut items = [
        StreamItem::Value(30),
        StreamItem::Value(10),
        StreamItem::Error(FluxionError::stream_error("error")),
        StreamItem::Value(20),
    ];

    // Act
    items.sort();

    // Assert
    assert!(items[0].is_error());
    assert_eq!(items[1].clone().ok(), Some(10));
    assert_eq!(items[2].clone().ok(), Some(20));
    assert_eq!(items[3].clone().ok(), Some(30));
}

#[test]
fn test_stream_item_debug_format() {
    // Arrange & Act
    let value_item = StreamItem::Value(42);
    let debug_str = format!("{:?}", value_item);

    // Assert
    assert!(debug_str.contains("Value"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_stream_item_error_debug_format() {
    // Arrange & Act
    let error_item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test error"));
    let debug_str = format!("{:?}", error_item);

    // Assert
    assert!(debug_str.contains("Error"));
}

#[test]
fn test_stream_item_complex_nested_operations() {
    // Arrange
    let process = |x: i32| -> StreamItem<String> {
        if x > 0 {
            StreamItem::Value(format!("positive: {}", x))
        } else {
            StreamItem::Error(FluxionError::stream_error("negative number"))
        }
    };

    // Act
    let result1 = StreamItem::Value(5).and_then(process);
    let result2 = StreamItem::Value(-5).and_then(process);

    // Assert
    assert_eq!(result1.ok(), Some("positive: 5".to_string()));
    assert!(result2.is_error());
}

#[test]
fn test_stream_item_option_like_behavior() {
    // Arrange & Act
    let item = StreamItem::Value(42);

    // Assert
    match item {
        StreamItem::Value(v) => assert_eq!(v, 42),
        StreamItem::Error(_) => panic!("Should be a value"),
    }
}

#[test]
fn test_stream_item_error_pattern_matching() {
    // Arrange & Act
    let item: StreamItem<i32> = StreamItem::Error(FluxionError::stream_error("test"));

    // Assert
    match item {
        StreamItem::Value(_) => panic!("Should be an error"),
        StreamItem::Error(e) => {
            assert!(format!("{:?}", e).contains("test"));
        }
    }
}
