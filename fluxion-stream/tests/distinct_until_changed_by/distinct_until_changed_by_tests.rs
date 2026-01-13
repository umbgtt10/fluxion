// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::HasTimestamp;
use fluxion_stream::DistinctUntilChangedByExt;
use fluxion_test_utils::{
    animal::Animal,
    helpers::{assert_no_element_emitted, assert_stream_ended, unwrap_stream},
    person::Person,
    test_channel,
    test_data::{animal_dog, animal_spider, person_alice, person_bob, person_diane, TestData},
    Sequenced,
};

#[tokio::test]
async fn test_distinct_until_changed_by_field_comparison() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    // Compare by age only, ignoring name
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Person(p1), TestData::Person(p2)) => p1.age == p2.age,
        _ => false,
    });

    // Act & Assert: First value always emitted
    tx.try_send(Sequenced::new(person_alice()))?;
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

    // Same age, different name - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Alice Updated".to_string(),
        25,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different age - emitted
    tx.try_send(Sequenced::new(person_bob()))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    // Same age as previous - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Bob v2".to_string(),
        30,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Back to age 25 - emitted (different from previous age 30)
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Alice v3".to_string(),
        25,
    ))))?;
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

    // Act & Assert
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "hello".to_string(),
        25,
    ))))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "hello");
    } else {
        panic!("Expected Person");
    }

    // Same name, different case - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "HELLO".to_string(),
        25,
    ))))?;
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "HeLLo".to_string(),
        25,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Different name - emitted
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "world".to_string(),
        30,
    ))))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.name, "world");
    } else {
        panic!("Expected Person");
    }

    // Same name, different case - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "WORLD".to_string(),
        30,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Back to previous value with different case - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "World".to_string(),
        30,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Truly different value
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "rust".to_string(),
        35,
    ))))?;
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

    // Act & Assert
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Alice".to_string(),
        20,
    ))))?;
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 20);
    } else {
        panic!("Expected Person");
    }

    // Small change - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Bob".to_string(),
        22,
    ))))?;
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Carol".to_string(),
        24,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Large change - emitted
    tx.try_send(Sequenced::new(person_bob()))?; // Age 30
    let person = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Person(p) = person {
        assert_eq!(p.age, 30);
    } else {
        panic!("Expected Person");
    }

    // Small change from 30 - filtered
    tx.try_send(Sequenced::new(TestData::Person(Person::new(
        "Eve".to_string(),
        33,
    ))))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Large change - emitted
    tx.try_send(Sequenced::new(person_diane()))?; // Age 40
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

    // Act & Assert
    tx.try_send(Sequenced::new(1))?; // Odd
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        1
    );

    // Another odd - filtered
    tx.try_send(Sequenced::new(3))?;
    tx.try_send(Sequenced::new(5))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Even - emitted (different parity)
    tx.try_send(Sequenced::new(2))?;
    assert_eq!(
        unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner(),
        2
    );

    // Another even - filtered
    tx.try_send(Sequenced::new(4))?;
    tx.try_send(Sequenced::new(6))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // Back to odd - emitted
    tx.try_send(Sequenced::new(7))?;
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
    tx.try_send(Sequenced::new(42))?;
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

    // Act & Assert
    tx.try_send(Sequenced::with_timestamp(1, 100))?;
    let first = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(first.timestamp(), 100);

    // Duplicate - filtered
    tx.try_send(Sequenced::with_timestamp(1, 200))?;
    assert_no_element_emitted(&mut distinct, 100).await;

    // New value with specific timestamp
    tx.try_send(Sequenced::with_timestamp(2, 300))?;
    let second = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(second.timestamp(), 300);
    assert_eq!(second.into_inner(), 2);

    // Another new value
    tx.try_send(Sequenced::with_timestamp(3, 400))?;
    let third = unwrap_stream(&mut distinct, 500).await.unwrap();
    assert_eq!(third.timestamp(), 400);

    Ok(())
}

#[tokio::test]
async fn test_distinct_until_changed_by_many_duplicates() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    // Compare by legs
    let mut distinct = stream.distinct_until_changed_by(|a, b| match (a, b) {
        (TestData::Animal(a1), TestData::Animal(a2)) => a1.legs == a2.legs,
        _ => false,
    });

    // Act & Assert
    tx.try_send(Sequenced::new(TestData::Animal(Animal::new(
        "Ant".to_string(),
        6,
    ))))?;
    let animal = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Animal(a) = animal {
        assert_eq!(a.legs, 6);
    } else {
        panic!("Expected Animal");
    }

    // Many 6-legged animals - all filtered
    for i in 0..25 {
        tx.try_send(Sequenced::new(TestData::Animal(Animal::new(
            format!("Insect{}", i),
            6,
        ))))?;
    }
    assert_no_element_emitted(&mut distinct, 100).await;

    // 4-legged animal - emitted
    tx.try_send(Sequenced::new(animal_dog()))?;
    let animal = unwrap_stream(&mut distinct, 500)
        .await
        .unwrap()
        .into_inner();
    if let TestData::Animal(a) = animal {
        assert_eq!(a.legs, 4);
    } else {
        panic!("Expected Animal");
    }

    // More 4-legged animals - filtered
    for i in 0..10 {
        tx.try_send(Sequenced::new(TestData::Animal(Animal::new(
            format!("Mammal{}", i),
            4,
        ))))?;
    }
    assert_no_element_emitted(&mut distinct, 100).await;

    // 8-legged animal - emitted
    tx.try_send(Sequenced::new(animal_spider()))?;
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

    // Act & Assert: Alternating values should all be emitted
    for i in 0..10 {
        tx.try_send(Sequenced::new(i % 2))?;
        let value = unwrap_stream(&mut distinct, 500)
            .await
            .unwrap()
            .into_inner();
        assert_eq!(value, i % 2);
    }

    Ok(())
}
