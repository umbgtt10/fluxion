// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::DistinctUntilChangedByExt;
use fluxion_test_utils::{
    animal::Animal,
    helpers::{assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream},
    person::Person,
    sequenced::Sequenced,
    test_data::{animal_dog, animal_spider, person_alice, person_bob, person_diane, TestData},
};

#[tokio::test]
async fn test_distinct_until_changed_by_field_comparison() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
        assert_eq!(p.name, "Alice");
    } else {
        panic!("Expected Person");
    }

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Alice Updated".to_string(),
        25,
    ))))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Bob v2".to_string(),
        30,
    ))))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Alice v3".to_string(),
        25,
    ))))?;

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 25);
        assert_eq!(p.name, "Alice v3");
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_case_insensitive() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    // Case-insensitive comparison by name
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => {
            p1.name.to_lowercase() == p2.name.to_lowercase()
        }
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "hello".to_string(),
        25,
    ))))?;

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "hello");
    } else {
        panic!("Expected Person");
    }

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "HELLO".to_string(),
        25,
    ))))?;
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "HeLLo".to_string(),
        25,
    ))))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "world".to_string(),
        30,
    ))))?;

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "world");
    } else {
        panic!("Expected Person");
    }

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "WORLD".to_string(),
        30,
    ))))?;
    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act: Back to previous value with different case - filtered
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "World".to_string(),
        30,
    ))))?;
    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act: Truly different value
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "rust".to_string(),
        35,
    ))))?;
    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "rust");
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_threshold() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    // Only emit if age difference is >= 5 years
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => (p1.age as i32 - p2.age as i32).abs() < 5,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Alice".to_string(),
        20,
    ))))?;
    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 20);
    } else {
        panic!("Expected Person");
    }

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Bob".to_string(),
        22,
    ))))?;
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Carol".to_string(),
        24,
    ))))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_bob()))?; // Age 30

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Person(Person::new(
        "Eve".to_string(),
        33,
    ))))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(person_diane()))?; // Age 40

    // Assert
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 40);
    } else {
        panic!("Expected Person");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_custom_logic() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    // Consider values "same" if they have the same parity (both even or both odd)
    let mut distinct = stream.distinct_until_changed_by(|a, b| a % 2 == b % 2);

    // Act
    tx.unbounded_send(Sequenced::new(1))?; // Odd

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Act
    tx.unbounded_send(Sequenced::new(3))?;
    tx.unbounded_send(Sequenced::new(5))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(2))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );

    // Act
    tx.unbounded_send(Sequenced::new(4))?;
    tx.unbounded_send(Sequenced::new(6))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(7))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        7
    );

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act
    drop(tx);

    // Assert
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_single_value() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act
    tx.unbounded_send(Sequenced::new(42))?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        42
    );
    assert_stream_ended(&mut distinct, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_preserves_timestamps() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(1, 100))?;

    // Assert
    let first = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(first.timestamp(), 100);

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(1, 200))?;

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(2, 300))?;

    // Assert
    let second = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(second.timestamp(), 300);
    assert_eq!(second.into_inner(), 2);

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(3, 400))?;

    // Assert
    let third = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(third.timestamp(), 400);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_many_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Animal(a1), TestData::Animal(a2)) => a1.legs == a2.legs,
        _ => false,
    });

    // Act
    tx.unbounded_send(Sequenced::new(TestData::Animal(Animal::new(
        "Ant".to_string(),
        6,
    ))))?;

    // Assert
    let animal = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Animal(a) = animal {
        assert_eq!(a.legs, 6);
    } else {
        panic!("Expected Animal");
    }

    // Act
    for i in 0..25 {
        tx.unbounded_send(Sequenced::new(TestData::Animal(Animal::new(
            format!("Insect{}", i),
            6,
        ))))?;
    }

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    let animal = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Animal(a) = animal {
        assert_eq!(a.legs, 4);
    } else {
        panic!("Expected Animal");
    }

    // Act
    for i in 0..10 {
        tx.unbounded_send(Sequenced::new(TestData::Animal(Animal::new(
            format!("Mammal{}", i),
            4,
        ))))?;
    }

    // Assert
    assert_no_element_emitted(&mut distinct, 100).await;

    // Act
    tx.unbounded_send(Sequenced::new(animal_spider()))?;

    // Assert
    let result = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Animal(a) = result {
        assert_eq!(a.legs, 8);
    } else {
        panic!("Expected Animal");
    }

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_alternating() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();
    let mut distinct = stream.distinct_until_changed_by(|a, b| a == b);

    // Act
    for i in 0..10 {
        tx.unbounded_send(Sequenced::new(i % 2))?;

        // Assert
        let value = unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner();
        assert_eq!(value, i % 2);
    }

    Ok(())
}
