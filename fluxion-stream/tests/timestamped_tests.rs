use fluxion_stream::Timestamped;

#[test]
fn test_timestamped_ordering() {
    // Arrange
    let first = Timestamped::new("first");
    let second = Timestamped::new("second");
    let third = Timestamped::new("third");

    // Assert
    assert!(first < second);
    assert!(second < third);
    assert!(first < third);
}

#[test]
fn test_timestamped_deref() {
    // Arrange
    let seq = Timestamped::new("hello");

    // Assert - Derefs to &str
    assert_eq!(seq.len(), 5);
}

#[test]
fn test_timestamped_new_assigns_sequence() {
    // Arrange & Act
    let item1 = Timestamped::new(42);
    let item2 = Timestamped::new(100);

    // Assert - Each new item gets a unique sequence
    assert_ne!(item1.sequence(), item2.sequence());
    assert!(item1.sequence() < item2.sequence());
}

#[test]
fn test_timestamped_into_inner() {
    // Arrange
    let original_value = String::from("test value");
    let timestamped = Timestamped::new(original_value.clone());

    // Act
    let extracted = timestamped.into_inner();

    // Assert
    assert_eq!(extracted, original_value);
}

#[test]
fn test_timestamped_get() {
    // Arrange
    let value = vec![1, 2, 3, 4, 5];
    let timestamped = Timestamped::new(value.clone());

    // Act
    let reference = timestamped.get();

    // Assert
    assert_eq!(reference, &value);
}

#[test]
fn test_timestamped_get_mut() {
    // Arrange
    let mut timestamped = Timestamped::new(vec![1, 2, 3]);

    // Act
    let mutable_ref = timestamped.get_mut();
    mutable_ref.push(4);

    // Assert
    assert_eq!(timestamped.value, vec![1, 2, 3, 4]);
}

#[test]
fn test_timestamped_equality_same_value_same_sequence() {
    // Arrange
    let item1 = Timestamped::new(42);
    let item2 = item1.clone();

    // Assert
    assert_eq!(item1, item2);
}

#[test]
fn test_timestamped_equality_same_value_different_sequence() {
    // Arrange
    let item1 = Timestamped::new(42);
    let item2 = Timestamped::new(42);

    // Assert - Different sequences means not equal
    assert_ne!(item1, item2);
}

#[test]
fn test_timestamped_inequality() {
    // Arrange
    let item1 = Timestamped::new(100);
    let item2 = Timestamped::new(200);

    // Assert
    assert_ne!(item1, item2);
}

#[test]
fn test_timestamped_partial_ord() {
    // Arrange
    let first = Timestamped::new("a");
    let second = Timestamped::new("z");

    // Act
    let comparison = first.partial_cmp(&second);

    // Assert - Ordering is based on sequence, not value
    assert_eq!(comparison, Some(std::cmp::Ordering::Less));
}

#[test]
fn test_timestamped_ord_consistent_with_sequence() {
    // Arrange
    let early = Timestamped::new(999);
    let late = Timestamped::new(1);

    // Assert - Even though 999 > 1, early was created first
    assert!(early < late);
    assert_eq!(early.cmp(&late), std::cmp::Ordering::Less);
}

#[test]
fn test_timestamped_display() {
    // Arrange
    let timestamped = Timestamped::new("hello world");

    // Act
    let displayed = format!("{}", timestamped);

    // Assert
    assert_eq!(displayed, "hello world");
}

#[test]
fn test_timestamped_display_with_number() {
    // Arrange
    let timestamped = Timestamped::new(12345);

    // Act
    let displayed = format!("{}", timestamped);

    // Assert
    assert_eq!(displayed, "12345");
}

#[test]
fn test_timestamped_deref_mut() {
    // Arrange
    let mut timestamped = Timestamped::new(String::from("hello"));

    // Act - Use deref_mut to modify
    timestamped.push_str(" world");

    // Assert
    assert_eq!(timestamped.value, "hello world");
}

#[test]
fn test_timestamped_clone_independence() {
    // Arrange
    let original = Timestamped::new(vec![1, 2, 3]);
    let mut cloned = original.clone();

    // Act
    cloned.value.push(4);

    // Assert - Original is unchanged
    assert_eq!(original.value, vec![1, 2, 3]);
    assert_eq!(cloned.value, vec![1, 2, 3, 4]);
    assert_eq!(original.sequence(), cloned.sequence());
}

#[test]
fn test_timestamped_sequence_monotonic() {
    // Arrange & Act
    let items: Vec<Timestamped<i32>> = (0..100).map(Timestamped::new).collect();

    // Assert - Sequences are strictly monotonically increasing
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
fn test_timestamped_debug() {
    // Arrange
    let timestamped = Timestamped::new(42);

    // Act
    let debug_string = format!("{:?}", timestamped);

    // Assert - Should contain both value and sequence
    assert!(debug_string.contains("42"));
    assert!(debug_string.contains("value"));
    assert!(debug_string.contains("sequence"));
}

#[test]
fn test_timestamped_multiple_types() {
    // Arrange & Act
    let string_ts = Timestamped::new(String::from("text"));
    let int_ts = Timestamped::new(123);
    let vec_ts = Timestamped::new(vec![1, 2, 3]);
    let tuple_ts = Timestamped::new((1, "a"));

    // Assert - All have valid sequences
    assert!(string_ts.sequence() < int_ts.sequence());
    assert!(int_ts.sequence() < vec_ts.sequence());
    assert!(vec_ts.sequence() < tuple_ts.sequence());
}

#[test]
fn test_timestamped_sorting_by_sequence() {
    // Arrange
    let item1 = Timestamped::new("third");
    let item2 = Timestamped::new("first");
    let item3 = Timestamped::new("second");

    let mut items = [item1.clone(), item2.clone(), item3.clone()];

    // Capture the original sequence order
    let original_order: Vec<_> = items.iter().map(|item| item.sequence()).collect();

    // Act - Reverse the vec (but sequences stay the same)
    items.reverse();
    items.sort();

    // Assert - After sorting, items are back in sequence order
    let sorted_order: Vec<_> = items.iter().map(|item| item.sequence()).collect();
    assert_eq!(sorted_order, original_order);
}

#[test]
fn test_timestamped_value_field_public() {
    // Arrange
    let timestamped = Timestamped::new(String::from("public"));

    // Act & Assert - Can access value field directly
    assert_eq!(timestamped.value, "public");
}
