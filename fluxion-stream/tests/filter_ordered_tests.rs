// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::{FluxionStream, Ordered};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_channel;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, person_charlie, person_dave, person_diane,
    plant_rose, TestData,
};
use futures::StreamExt;

#[tokio::test]
async fn test_filter_ordered_basic_predicate() {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream =
        FluxionStream::new(stream).filter_ordered(|data| matches!(data, TestData::Person(_)));

    // Act & Assert
    tx.send(Sequenced::new(person_alice())).unwrap();
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &person_alice());

    tx.send(Sequenced::new(animal_dog())).unwrap();
    tx.send(Sequenced::new(person_bob())).unwrap();

    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &person_bob()); // Animal filtered out
}

#[tokio::test]
async fn test_filter_ordered_age_threshold() {
    // Arrange - filter people by age > 30
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.age > 30,
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap(); // 25 - filtered
    tx.send(Sequenced::new(person_bob())).unwrap(); // 30 - filtered
    tx.send(Sequenced::new(person_charlie())).unwrap(); // 35 - kept
    tx.send(Sequenced::new(person_diane())).unwrap(); // 40 - kept

    // Assert
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &person_charlie());

    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &person_diane());
}

#[tokio::test]
async fn test_filter_ordered_empty_stream() {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut stream = FluxionStream::new(stream).filter_ordered(|_| true);

    // Act
    drop(tx); // Close the channel

    // Assert
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_filter_ordered_all_filtered_out() {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|_| false); // Filter everything

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    tx.send(Sequenced::new(person_bob())).unwrap();
    tx.send(Sequenced::new(animal_dog())).unwrap();
    drop(tx);

    // Assert
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_filter_ordered_none_filtered() {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|_| true); // Keep everything

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    tx.send(Sequenced::new(animal_dog())).unwrap();
    tx.send(Sequenced::new(plant_rose())).unwrap();

    // Assert
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_alice());
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &animal_dog());
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &plant_rose());
}

#[tokio::test]
async fn test_filter_ordered_preserves_ordering() {
    // Arrange
    let (tx, stream) = test_channel();

    // Keep only people with even ages
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.age % 2 == 0,
        _ => false,
    });

    // Act - send in sequence order
    tx.send(Sequenced::new(person_alice())).unwrap(); // 25 - odd, filtered
    tx.send(Sequenced::new(person_bob())).unwrap(); // 30 - even, kept
    tx.send(Sequenced::new(person_charlie())).unwrap(); // 35 - odd, filtered
    tx.send(Sequenced::new(person_diane())).unwrap(); // 40 - even, kept
    tx.send(Sequenced::new(person_dave())).unwrap(); // 28 - even, kept

    // Assert - ordering preserved for kept items
    let r1 = stream.next().await.unwrap().unwrap();
    let r2 = stream.next().await.unwrap().unwrap();
    let r3 = stream.next().await.unwrap().unwrap();

    assert_eq!(r1.get(), &person_bob());
    assert_eq!(r2.get(), &person_diane());
    assert_eq!(r3.get(), &person_dave());

    // Verify sequence numbers are in order
    assert!(r1.order() < r2.order());
    assert!(r2.order() < r3.order());
}

#[tokio::test]
async fn test_filter_ordered_multiple_types() {
    // Arrange - keep only animals
    let (tx, stream) = test_channel();
    let mut stream =
        FluxionStream::new(stream).filter_ordered(|data| matches!(data, TestData::Animal(_)));

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    tx.send(Sequenced::new(animal_dog())).unwrap();
    tx.send(Sequenced::new(plant_rose())).unwrap();
    tx.send(Sequenced::new(animal_spider())).unwrap();
    tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &animal_dog());

    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &animal_spider());
}

#[tokio::test]
async fn test_filter_ordered_complex_predicate() {
    // Arrange - complex predicate: people with age between 30-40 OR animals
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.age >= 30 && p.age <= 40,
        TestData::Animal(_) => true,
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap(); // 25 - filtered
    tx.send(Sequenced::new(person_bob())).unwrap(); // 30 - kept
    tx.send(Sequenced::new(animal_dog())).unwrap(); // kept
    tx.send(Sequenced::new(plant_rose())).unwrap(); // filtered
    tx.send(Sequenced::new(person_diane())).unwrap(); // 40 - kept

    // Assert
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_bob());
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &animal_dog());
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_diane());
}

#[tokio::test]
async fn test_filter_ordered_single_item() {
    // Arrange
    let (tx, stream) = test_channel();
    let mut stream =
        FluxionStream::new(stream).filter_ordered(|data| matches!(data, TestData::Person(_)));

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap();
    drop(tx);

    // Assert
    let result = stream.next().await.unwrap().unwrap();
    assert_eq!(result.get(), &person_alice());
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn test_filter_ordered_with_pattern_matching() {
    // Arrange - filter by name pattern
    let (tx, stream) = test_channel();
    let mut stream = FluxionStream::new(stream).filter_ordered(|data| match data {
        TestData::Person(p) => p.name.starts_with('A') || p.name.starts_with('D'),
        _ => false,
    });

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap(); // Alice - kept
    tx.send(Sequenced::new(person_bob())).unwrap(); // Bob - filtered
    tx.send(Sequenced::new(person_charlie())).unwrap(); // Charlie - filtered
    tx.send(Sequenced::new(person_dave())).unwrap(); // Dave - kept
    tx.send(Sequenced::new(person_diane())).unwrap(); // Diane - kept

    // Assert
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_alice());
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_dave());
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_diane());
}

#[tokio::test]
async fn test_filter_ordered_alternating_pattern() {
    // Arrange - Keep every other person by creating a stateful filter
    let (tx, stream) = test_channel();

    let mut count = 0;
    let mut stream = FluxionStream::new(stream).filter_ordered(move |data| {
        if matches!(data, TestData::Person(_)) {
            count += 1;
            count % 2 == 1 // Keep odd-numbered people
        } else {
            false
        }
    });

    // Act
    tx.send(Sequenced::new(person_alice())).unwrap(); // 1st person - kept
    tx.send(Sequenced::new(person_bob())).unwrap(); // 2nd person - filtered
    tx.send(Sequenced::new(animal_dog())).unwrap(); // not a person - filtered
    tx.send(Sequenced::new(person_charlie())).unwrap(); // 3rd person - kept
    tx.send(Sequenced::new(person_diane())).unwrap(); // 4th person - filtered

    // Assert
    assert_eq!(stream.next().await.unwrap().unwrap().get(), &person_alice());
    assert_eq!(
        stream.next().await.unwrap().unwrap().get(),
        &person_charlie()
    );
}
