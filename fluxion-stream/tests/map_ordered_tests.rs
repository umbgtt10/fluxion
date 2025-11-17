// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::combine_with_previous::CombineWithPreviousExt;
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_dave, TestData,
};
use fluxion_test_utils::unwrap_value;
use futures::StreamExt;

#[tokio::test]
async fn test_map_ordered_basic_transformation() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        format!(
            "Previous: {:?}, Current: {}",
            stream_item.previous.map(|p| p.get().to_string()),
            stream_item.current.get()
        )
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        "Previous: None, Current: Person[name=Alice, age=25]"
    );

    tx.send(Sequenced::new(person_bob())).unwrap();
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        "Previous: Some(\"Person[name=Alice, age=25]\"), Current: Person[name=Bob, age=30]"
    );

    tx.send(Sequenced::new(person_charlie())).unwrap();
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        "Previous: Some(\"Person[name=Bob, age=30]\"), Current: Person[name=Charlie, age=35]"
    );
}

#[tokio::test]
async fn test_map_ordered_to_struct() {
    // Arrange
    #[derive(Debug, PartialEq)]
    struct AgeComparison {
        previous_age: Option<u32>,
        current_age: u32,
        age_increased: bool,
    }

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        let current_age = match &stream_item.current.get() {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        let previous_age = stream_item
            .previous
            .as_ref()
            .and_then(|prev| match &prev.get() {
                TestData::Person(p) => Some(p.age),
                _ => None,
            });
        let age_increased = previous_age.is_some_and(|prev| current_age > prev);
        AgeComparison {
            previous_age,
            current_age,
            age_increased,
        }
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        AgeComparison {
            previous_age: None,
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(25),
            current_age: 30,
            age_increased: true,
        }
    );

    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25 again
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(30),
            current_age: 25,
            age_increased: false,
        }
    );

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        AgeComparison {
            previous_age: Some(25),
            current_age: 35,
            age_increased: true,
        }
    );
}

#[tokio::test]
async fn test_map_ordered_extract_age_difference() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| -> i32 {
            let current_age = match &stream_item.current.get() {
                TestData::Person(p) => p.age as i32,
                _ => 0,
            };
            let previous_age = stream_item
                .previous
                .as_ref()
                .and_then(|prev| match &prev.get() {
                    TestData::Person(p) => Some(p.age as i32),
                    _ => None,
                });
            current_age - previous_age.unwrap_or(current_age)
        });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25
    let result = unwrap_value(stream.next().await);
    assert_eq!(result, 0); // No previous

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = unwrap_value(stream.next().await);
    assert_eq!(result, 5); // 30 - 25 = 5

    tx.send(Sequenced::new(person_dave())).unwrap(); // Age 28
    let result = unwrap_value(stream.next().await);
    assert_eq!(result, -2); // 28 - 30 = -2

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    let result = unwrap_value(stream.next().await);
    assert_eq!(result, 7); // 35 - 28 = 7
}

#[tokio::test]
async fn test_map_ordered_single_value() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| stream_item.current.get().to_string());

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();
    let result = unwrap_value(stream.next().await);
    assert_eq!(result, "Person[name=Alice, age=25]");
}

#[tokio::test]
async fn test_map_ordered_empty_stream() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream
        .combine_with_previous()
        .map_ordered(|stream_item| stream_item.current.get().to_string());

    // Act
    drop(tx); // Close the stream

    // Assert
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_map_ordered_preserves_ordering() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        // Extract name from current
        match &stream_item.current.get() {
            TestData::Person(p) => p.name.clone(),
            _ => String::from("Unknown"),
        }
    });

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    tx.send(Sequenced::new(person_bob())).unwrap();
    tx.send(Sequenced::new(person_charlie())).unwrap();
    tx.send(Sequenced::new(person_dave())).unwrap();

    // Assert - order should be preserved
    assert_eq!(unwrap_value(stream.next().await), "Alice");
    assert_eq!(unwrap_value(stream.next().await), "Bob");
    assert_eq!(unwrap_value(stream.next().await), "Charlie");
    assert_eq!(unwrap_value(stream.next().await), "Dave");
}

#[tokio::test]
async fn test_map_ordered_multiple_transformations() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        // First transformation: extract age
        match &stream_item.current.get() {
            TestData::Person(p) => p.age,
            _ => 0,
        }
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();
    assert_eq!(unwrap_value(stream.next().await), 25);

    tx.send(Sequenced::new(person_bob())).unwrap();
    assert_eq!(unwrap_value(stream.next().await), 30);

    tx.send(Sequenced::new(person_charlie())).unwrap();
    assert_eq!(unwrap_value(stream.next().await), 35);
}

#[tokio::test]
async fn test_map_ordered_with_complex_closure() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    #[derive(Debug, PartialEq)]
    struct PersonSummary {
        name: String,
        age_category: &'static str,
        changed_from_previous: bool,
    }

    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        let current = match &stream_item.current.get() {
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
            if let TestData::Person(prev_person) = &prev.get() {
                prev_person.name == current.name
            } else {
                false
            }
        });

        PersonSummary {
            name: current.name.clone(),
            age_category,
            changed_from_previous,
        }
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        PersonSummary {
            name: String::from("Alice"),
            age_category: "young adult",
            changed_from_previous: true,
        }
    );

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        PersonSummary {
            name: String::from("Bob"),
            age_category: "adult",
            changed_from_previous: true,
        }
    );

    tx.send(Sequenced::new(person_bob())).unwrap(); // Same person
    let result = unwrap_value(stream.next().await);
    assert_eq!(
        result,
        PersonSummary {
            name: String::from("Bob"),
            age_category: "adult",
            changed_from_previous: false,
        }
    );
}

#[tokio::test]
async fn test_map_ordered_boolean_logic() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = stream.combine_with_previous().map_ordered(|stream_item| {
        // Returns true if age increased from previous
        let current_age = match &stream_item.current.get() {
            TestData::Person(p) => p.age,
            _ => 0,
        };
        stream_item.previous.as_ref().is_some_and(|prev| {
            if let TestData::Person(p) = &prev.get() {
                current_age > p.age
            } else {
                false
            }
        })
    });

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25
    assert!(!unwrap_value(stream.next().await)); // No previous

    tx.send(Sequenced::new(person_bob())).unwrap(); // Age 30
    assert!(unwrap_value(stream.next().await)); // 30 > 25

    tx.send(Sequenced::new(person_charlie())).unwrap(); // Age 35
    assert!(unwrap_value(stream.next().await)); // 35 > 30

    tx.send(Sequenced::new(person_dave())).unwrap(); // Age 28
    assert!(!unwrap_value(stream.next().await)); // 28 < 35

    tx.send(Sequenced::new(person_alice())).unwrap(); // Age 25
    assert!(!unwrap_value(stream.next().await)); // 25 < 28
}
