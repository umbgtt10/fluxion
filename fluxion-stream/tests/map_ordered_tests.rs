// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_dave, TestData,
};
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{assert_stream_ended, test_channel};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value};

#[tokio::test]
async fn test_map_ordered_basic_transformation() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        format!(
            "Previous: {:?}, Current: {}",
            stream_item.previous.map(|p| p.value.to_string()),
            &stream_item.current.value
        )
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
    );

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_to_struct() -> anyhow::Result<()> {
    // Arrange
    #[derive(Debug, PartialEq)]
    struct AgeComparison {
        previous_age: Option<u32>,
        current_age: u32,
        age_increased: bool,
    }

    impl AgeComparison {
        fn new(previous_age: Option<u32>, current_age: u32) -> Self {
            let age_increased = previous_age.is_some_and(|prev| current_age > prev);
            AgeComparison {
                previous_age,
                current_age,
                age_increased,
            }
        }
    }

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        let current_age = match &stream_item.current.value {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let previous_age = stream_item
            .previous
            .as_ref()
            .and_then(|prev| match &prev.value {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });
        AgeComparison::new(previous_age, current_age)
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        AgeComparison::new(None, 25)
    );

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        AgeComparison::new(Some(25), 30)
    );

    tx.send(Sequenced::new(person_alice()))?; // Age 25 again
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        AgeComparison::new(Some(30), 25)
    );

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        AgeComparison::new(Some(25), 35)
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_extract_age_difference() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| -> i32 {
            let current_age = match &stream_item.current.value {
                TestData::Person(p) => p.age as i32,
                _ => 0,
            };
            let previous_age = stream_item
                .previous
                .as_ref()
                .and_then(|prev| match &prev.value {
                    TestData::Person(p) => Some(p.age as i32),
                    _ => None,
                });
            current_age - previous_age.unwrap_or(current_age)
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(unwrap_value(Some(unwrap_stream(&mut stream, 500).await)), 0); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(unwrap_value(Some(unwrap_stream(&mut stream, 500).await)), 5); // 30 - 25 = 5

    tx.send(Sequenced::new(person_dave()))?; // Age 28
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        -2
    ); // 28 - 30 = -2

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert_eq!(unwrap_value(Some(unwrap_stream(&mut stream, 500).await)), 7); // 35 - 28 = 7

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_single_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| stream_item.current.value.to_string());

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Person[name=Alice, age=25]"
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| stream_item.current.value.to_string());

    // Act
    drop(tx); // Close the stream

    // Assert
    assert_stream_ended(&mut stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_preserves_ordering() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        // Extract name from current
        match &stream_item.current.value {
            TestData::Person(p) => p.name.clone(),
            _ => String::from("Unknown"),
        }
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(person_bob()))?;
    tx.send(Sequenced::new(person_charlie()))?;
    tx.send(Sequenced::new(person_dave()))?;

    // Assert - order should be preserved
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Alice"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Bob"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Charlie"
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        "Dave"
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_multiple_transformations() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        // First transformation: extract age
        match &stream_item.current.value {
            TestData::Person(p) => p.age,
            _ => 0,
        }
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        25
    );

    tx.send(Sequenced::new(person_bob()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        30
    );

    tx.send(Sequenced::new(person_charlie()))?;
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        35
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_with_complex_closure() -> anyhow::Result<()> {
    // Arrange
    #[derive(Debug, PartialEq)]
    struct PersonSummary {
        name: String,
        age_category: &'static str,
        changed_from_previous: bool,
    }

    impl PersonSummary {
        fn new(name: String, age_category: &'static str, changed_from_previous: bool) -> Self {
            PersonSummary {
                name,
                age_category,
                changed_from_previous,
            }
        }
    }

    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        let current = match &stream_item.current.value {
            TestData::Person(p) => p,
            _ => panic!("Expected person"),
        };

        let age_category = match current.age {
            0..=17 => "child",
            18..=29 => "young adult",
            30..=59 => "adult",
            _ => "senior",
        };

        let changed_from_previous = !stream_item.previous.as_ref().is_some_and(|prev| {
            if let TestData::Person(prev_person) = &prev.value {
                prev_person.name == current.name
            } else {
                false
            }
        });

        PersonSummary::new(current.name.clone(), age_category, changed_from_previous)
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        PersonSummary::new(String::from("Alice"), "young adult", true)
    );

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        PersonSummary::new(String::from("Bob"), "adult", true)
    );

    tx.send(Sequenced::new(person_bob()))?; // Same person
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        PersonSummary::new(String::from("Bob"), "adult", false)
    );

    Ok(())
}

#[tokio::test]
async fn test_map_ordered_boolean_logic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        // Returns true if age increased from previous
        let current_age = match &stream_item.current.value {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        stream_item.previous.as_ref().is_some_and(|prev| {
            if let TestData::Person(p) = &prev.value {
                current_age > p.age
            } else {
                false
            }
        })
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert!(!unwrap_value(Some(unwrap_stream(&mut stream, 500).await))); // No previous

    tx.send(Sequenced::new(person_bob()))?; // Age 30
    assert!(unwrap_value(Some(unwrap_stream(&mut stream, 500).await))); // 30 > 25

    tx.send(Sequenced::new(person_charlie()))?; // Age 35
    assert!(unwrap_value(Some(unwrap_stream(&mut stream, 500).await))); // 35 > 30

    tx.send(Sequenced::new(person_dave()))?; // Age 28
    assert!(!unwrap_value(Some(unwrap_stream(&mut stream, 500).await))); // 28 < 35

    tx.send(Sequenced::new(person_alice()))?; // Age 25
    assert!(!unwrap_value(Some(unwrap_stream(&mut stream, 500).await))); // 25 < 28

    Ok(())
}
