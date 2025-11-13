// Copyright 2025 Umberto Gotti
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_test_utils::Sequenced;

#[test]
fn test_sequenced_ordering() {
    // Arrange
    let first = Sequenced::new("first");
    let second = Sequenced::new("second");
    let third = Sequenced::new("third");

    // Assert
    assert!(first < second);
    assert!(second < third);
    assert!(first < third);
}

#[test]
fn test_sequenced_deref() {
    // Arrange
    let seq = Sequenced::new("hello");

    // Assert
    assert_eq!(seq.len(), 5);
}

#[test]
fn test_sequenced_new_assigns_sequence() {
    // Arrange & Act
    let item1 = Sequenced::new(42);
    let item2 = Sequenced::new(100);

    // Assert
    assert_ne!(item1.sequence(), item2.sequence());
    assert!(item1.sequence() < item2.sequence());
}

#[test]
fn test_sequenced_into_inner() {
    // Arrange
    let original_value = String::from("test value");
    let sequenced = Sequenced::new(original_value.clone());

    // Act
    let extracted = sequenced.into_inner();

    // Assert
    assert_eq!(extracted, original_value);
}

#[test]
fn test_sequenced_get() {
    // Arrange
    let value = vec![1, 2, 3, 4, 5];
    let sequenced = Sequenced::new(value.clone());

    // Act
    let reference = sequenced.get();

    // Assert
    assert_eq!(reference, &value);
}

#[test]
fn test_sequenced_get_mut() {
    // Arrange
    let mut sequenced = Sequenced::new(vec![1, 2, 3]);

    // Act
    let mutable_ref = sequenced.get_mut();
    mutable_ref.push(4);

    // Assert
    assert_eq!(sequenced.value, vec![1, 2, 3, 4]);
}

#[test]
fn test_sequenced_equality_same_value_same_sequence() {
    // Arrange
    let item1 = Sequenced::new(42);
    let item2 = item1.clone();

    // Assert
    assert_eq!(item1, item2);
}

#[test]
fn test_sequenced_equality_same_value_different_sequence() {
    // Arrange
    let item1 = Sequenced::new(42);
    let item2 = Sequenced::new(42);

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_sequenced_inequality() {
    // Arrange
    let item1 = Sequenced::new(100);
    let item2 = Sequenced::new(200);

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_sequenced_partial_ord() {
    // Arrange
    let first = Sequenced::new("a");
    let second = Sequenced::new("z");

    // Act
    let comparison = first.partial_cmp(&second);

    // Assert
    assert_eq!(comparison, Some(std::cmp::Ordering::Less));
}

#[test]
fn test_sequenced_ord_consistent_with_sequence() {
    // Arrange
    let early = Sequenced::new(999);
    let late = Sequenced::new(1);

    // Assert
    assert!(early < late);
    assert_eq!(early.cmp(&late), std::cmp::Ordering::Less);
}

#[test]
fn test_sequenced_display() {
    // Arrange
    let sequenced = Sequenced::new("hello world");

    // Act
    let displayed = format!("{sequenced}");

    // Assert
    assert_eq!(displayed, "hello world");
}

#[test]
fn test_sequenced_display_with_number() {
    // Arrange
    let sequenced = Sequenced::new(12345);

    // Act
    let displayed = format!("{sequenced}");

    // Assert
    assert_eq!(displayed, "12345");
}

#[test]
fn test_sequenced_deref_mut() {
    // Arrange
    let mut sequenced = Sequenced::new(String::from("hello"));

    // Act
    sequenced.push_str(" world");

    // Assert
    assert_eq!(sequenced.value, "hello world");
}

#[test]
fn test_sequenced_clone_independence() {
    // Arrange
    let original = Sequenced::new(vec![1, 2, 3]);
    let mut cloned = original.clone();

    // Act
    cloned.value.push(4);

    // Assert
    assert_eq!(original.value, vec![1, 2, 3]);
    assert_eq!(cloned.value, vec![1, 2, 3, 4]);
    assert_eq!(original.sequence(), cloned.sequence());
}

#[test]
fn test_sequenced_sequence_monotonic() {
    // Arrange & Act
    let items: Vec<Sequenced<i32>> = (0..100).map(Sequenced::new).collect();

    // Assert
    for i in 1..items.len() {
        assert!(
            items[i - 1].sequence() < items[i].sequence(),
            "Sequence at {} should be less than sequence at {}",
            i - 1,
            i
        );
    }
}

#[test]
fn test_sequenced_debug() {
    // Arrange
    let sequenced = Sequenced::new(42);

    // Act
    let debug_string = format!("{sequenced:?}");

    // Assert
    assert!(debug_string.contains("42"));
    assert!(debug_string.contains("value"));
    assert!(debug_string.contains("sequence"));
}

#[test]
fn test_sequenced_multiple_types() {
    // Arrange & Act
    let string_ts = Sequenced::new(String::from("text"));
    let int_ts = Sequenced::new(123);
    let vec_ts = Sequenced::new(vec![1, 2, 3]);
    let tuple_ts = Sequenced::new((1, "a"));

    // Assert
    assert!(string_ts.sequence() < int_ts.sequence());
    assert!(int_ts.sequence() < vec_ts.sequence());
    assert!(vec_ts.sequence() < tuple_ts.sequence());
}

#[test]
fn test_sequenced_sorting_by_sequence() {
    // Arrange
    let item1 = Sequenced::new("third");
    let item2 = Sequenced::new("first");
    let item3 = Sequenced::new("second");

    let mut items = [item1.clone(), item2.clone(), item3.clone()];

    // Capture the original sequence order
    let original_order: Vec<_> = items.iter().map(Sequenced::sequence).collect();

    // Act
    items.reverse();
    items.sort();

    // Assert
    let sorted_order: Vec<_> = items.iter().map(Sequenced::sequence).collect();
    assert_eq!(sorted_order, original_order);
}

#[test]
fn test_sequenced_value_field_public() {
    // Arrange
    let sequenced = Sequenced::new(String::from("public"));

    // Act & Assert
    assert_eq!(sequenced.value, "public");
}
