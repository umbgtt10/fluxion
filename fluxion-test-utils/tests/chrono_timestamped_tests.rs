// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_test_utils::ChronoTimestamped;
use std::cmp::Ordering;

#[test]
fn test_chrono_timestamped_ordering() {
    // Arrange
    let first = ChronoTimestamped::new("first");
    let second = ChronoTimestamped::new("second");
    let third = ChronoTimestamped::new("third");

    // Assert
    assert!(first < second);
    assert!(second < third);
    assert!(first < third);
}

#[test]
fn test_chrono_timestamped_deref() {
    // Arrange
    let seq = ChronoTimestamped::new("hello");

    // Assert
    assert_eq!(seq.len(), 5);
}

#[test]
fn test_chrono_timestamped_new_assigns_sequence() {
    // Arrange & Act
    let item1 = ChronoTimestamped::new(42);
    let item2 = ChronoTimestamped::new(100);

    // Assert
    assert_ne!(item1.timestamp(), item2.timestamp());
    assert!(item1.timestamp() < item2.timestamp());
}

#[test]
fn test_chrono_timestamped_into_inner() {
    // Arrange
    let original_value = String::from("test value");
    let timestamped = ChronoTimestamped::new(original_value.clone());

    // Act
    let extracted = timestamped.into_inner();

    // Assert
    assert_eq!(extracted, original_value);
}

#[test]
fn test_chrono_timestamped_get() {
    // Arrange
    let value = vec![1, 2, 3, 4, 5];
    let timestamped = ChronoTimestamped::new(value.clone());

    // Act
    let reference = timestamped.inner();

    // Assert
    assert_eq!(reference, &value);
}

#[test]
fn test_chrono_timestamped_get_mut() {
    // Arrange
    let mut timestamped = ChronoTimestamped::new(vec![1, 2, 3]);

    // Act
    let mutable_ref = timestamped.get_mut();
    mutable_ref.push(4);

    // Assert
    assert_eq!(timestamped.value, vec![1, 2, 3, 4]);
}

#[test]
fn test_chrono_timestamped_equality_same_value_same_sequence() {
    // Arrange
    let item1 = ChronoTimestamped::new(42);
    let item2 = item1.clone();

    // Assert
    assert_eq!(item1, item2);
}

#[test]
fn test_chrono_timestamped_equality_same_value_different_sequence() {
    // Arrange
    let item1 = ChronoTimestamped::new(42);
    let item2 = ChronoTimestamped::new(42);

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_chrono_timestamped_inequality() {
    // Arrange
    let item1 = ChronoTimestamped::new(100);
    let item2 = ChronoTimestamped::new(200);

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_chrono_timestamped_partial_ord() {
    // Arrange
    let first = ChronoTimestamped::new("a");
    let second = ChronoTimestamped::new("z");

    // Act
    let comparison = first.partial_cmp(&second);

    // Assert
    assert_eq!(comparison, Some(Ordering::Less));
}

#[test]
fn test_chrono_timestamped_ord_consistent_with_timestamp() {
    // Arrange
    let early = ChronoTimestamped::new(999);
    let late = ChronoTimestamped::new(1);

    // Assert
    assert!(early < late);
    assert_eq!(early.cmp(&late), Ordering::Less);
}

#[test]
fn test_chrono_timestamped_display() {
    // Arrange
    let timestamped = ChronoTimestamped::new("hello world");

    // Act
    let displayed = format!("{timestamped}");

    // Assert
    assert_eq!(displayed, "hello world");
}

#[test]
fn test_chrono_timestamped_display_with_number() {
    // Arrange
    let timestamped = ChronoTimestamped::new(12345);

    // Act
    let displayed = format!("{timestamped}");

    // Assert
    assert_eq!(displayed, "12345");
}

#[test]
fn test_chrono_timestamped_deref_mut() {
    // Arrange
    let mut timestamped = ChronoTimestamped::new(String::from("hello"));

    // Act
    timestamped.push_str(" world");

    // Assert
    assert_eq!(timestamped.value, "hello world");
}

#[test]
fn test_chrono_timestamped_clone_independence() {
    // Arrange
    let original = ChronoTimestamped::new(vec![1, 2, 3]);
    let mut cloned = original.clone();

    // Act
    cloned.value.push(4);

    // Assert
    assert_eq!(original.value, vec![1, 2, 3]);
    assert_eq!(cloned.value, vec![1, 2, 3, 4]);
    assert_eq!(original.timestamp(), cloned.timestamp());
}

#[test]
fn test_chrono_timestamped_sequence_monotonic() {
    // Arrange & Act - Add small delays to ensure timestamp differences
    let items: Vec<ChronoTimestamped<i32>> = (0..10)
        .map(|i| {
            let item = ChronoTimestamped::new(i);
            // Small sleep to ensure timestamp differences
            std::thread::sleep(std::time::Duration::from_micros(100));
            item
        })
        .collect();

    // Assert - timestamps should increase or stay same (due to precision limits)
    for i in 1..items.len() {
        assert!(
            items[i - 1].timestamp() <= items[i].timestamp(),
            "Timestamp at {} should be <= timestamp at {}",
            i - 1,
            i
        );
    }
}

#[test]
fn test_chrono_timestamped_debug() {
    // Arrange
    let timestamped = ChronoTimestamped::new(42);

    // Act
    let debug_string = format!("{timestamped:?}");

    // Assert
    assert!(debug_string.contains("42"));
    assert!(debug_string.contains("value"));
    assert!(debug_string.contains("timestamp"));
}

#[test]
fn test_chrono_timestamped_multiple_types() {
    // Arrange & Act
    let string_ts = ChronoTimestamped::new(String::from("text"));
    let int_ts = ChronoTimestamped::new(123);
    let vec_ts = ChronoTimestamped::new(vec![1, 2, 3]);
    let tuple_ts = ChronoTimestamped::new((1, "a"));

    // Assert
    assert!(string_ts.timestamp() < int_ts.timestamp());
    assert!(int_ts.timestamp() < vec_ts.timestamp());
    assert!(vec_ts.timestamp() < tuple_ts.timestamp());
}

#[test]
fn test_chrono_timestamped_sorting_by_sequence() {
    // Arrange
    let item1 = ChronoTimestamped::new("third");
    let item2 = ChronoTimestamped::new("first");
    let item3 = ChronoTimestamped::new("second");

    let mut items = [item1.clone(), item2.clone(), item3.clone()];

    // Capture the original sequence order
    let original_order: Vec<_> = items.iter().map(Timestamped::timestamp).collect();

    // Act
    items.reverse();
    items.sort();

    // Assert
    let sorted_order: Vec<_> = items.iter().map(Timestamped::timestamp).collect();
    assert_eq!(sorted_order, original_order);
}

#[test]
fn test_chrono_timestamped_value_field_public() {
    // Arrange
    let timestamped = ChronoTimestamped::new(String::from("public"));

    // Act & Assert
    assert_eq!(timestamped.value, "public");
}
