// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::fluxion_mutex::Mutex;
use fluxion_stream::{FilterOrderedExt, MapOrderedExt, TapExt};
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{
    animal_dog, person_alice, person_bob, person_charlie, TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, test_channel};
use fluxion_test_utils::{helpers::unwrap_stream, unwrap_value, Sequenced};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn test_tap_after_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel();
    let mut result = stream
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .tap(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(Sequenced::new(animal_dog()))?; // Filtered out - tap not called

    tx.try_send(Sequenced::new(person_bob()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_tap_before_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel();
    let mut result = stream
        .tap(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(Sequenced::new(animal_dog()))?;
    tx.try_send(Sequenced::new(person_bob()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter.load(Ordering::SeqCst), 3);

    Ok(())
}

#[tokio::test]
async fn test_tap_after_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_clone = observed.clone();

    let (tx, stream) = test_channel();
    let mut result = stream
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Dr. {}", p.name),
                    ..p
                }),
                other => other,
            })
        })
        .tap(move |value| {
            observed_clone.lock().push(value.clone());
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    let values = observed.lock();
    assert!(matches!(
        &values[0],
        TestData::Person(p) if p.name.starts_with("Dr. ")
    ));

    Ok(())
}

#[tokio::test]
async fn test_tap_before_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_clone = observed.clone();

    let (tx, stream) = test_channel();
    let mut result = stream
        .tap(move |value: &TestData| {
            observed_clone.lock().push(value.clone());
        })
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: format!("Dr. {}", p.name),
                    ..p
                }),
                other => other,
            })
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;

    // Assert
    let result_item = unwrap_value(Some(unwrap_stream(&mut result, 500).await));
    let values = observed.lock();
    assert_eq!(values[0], person_alice()); // Original
    assert!(matches!(
        &result_item.value,
        TestData::Person(p) if p.name.starts_with("Dr. ")
    )); // Transformed

    Ok(())
}

#[tokio::test]
async fn test_multiple_taps_in_pipeline() -> anyhow::Result<()> {
    // Arrange
    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter1_clone = counter1.clone();
    let counter2 = Arc::new(AtomicUsize::new(0));
    let counter2_clone = counter2.clone();
    let counter3 = Arc::new(AtomicUsize::new(0));
    let counter3_clone = counter3.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .tap(move |_| {
            counter1_clone.fetch_add(1, Ordering::SeqCst);
        })
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .tap(move |_| {
            counter2_clone.fetch_add(1, Ordering::SeqCst);
        })
        .map_ordered(|item: Sequenced<TestData>| item)
        .tap(move |_| {
            counter3_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(Sequenced::new(animal_dog()))?; // Filtered after first tap

    tx.try_send(Sequenced::new(person_bob()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert
    assert_eq!(counter1.load(Ordering::SeqCst), 3); // All items
    assert_eq!(counter2.load(Ordering::SeqCst), 2); // After filter
    assert_eq!(counter3.load(Ordering::SeqCst), 2); // After map

    Ok(())
}

#[tokio::test]
async fn test_tap_chained_taps() -> anyhow::Result<()> {
    // Arrange - multiple consecutive taps
    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter1_clone = counter1.clone();
    let counter2 = Arc::new(AtomicUsize::new(0));
    let counter2_clone = counter2.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .tap(move |_| {
            counter1_clone.fetch_add(1, Ordering::SeqCst);
        })
        .tap(move |_| {
            counter2_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    unwrap_stream(&mut result, 500).await;

    tx.try_send(Sequenced::new(person_bob()))?;
    unwrap_stream(&mut result, 500).await;

    // Assert - both taps called for each item
    assert_eq!(counter1.load(Ordering::SeqCst), 2);
    assert_eq!(counter2.load(Ordering::SeqCst), 2);

    Ok(())
}

#[tokio::test]
async fn test_tap_with_filter_that_blocks_all() -> anyhow::Result<()> {
    // Arrange
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut result = stream
        .filter_ordered(|_| false) // Block everything
        .tap(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

    // Act
    tx.try_send(Sequenced::new(person_alice()))?;
    tx.try_send(Sequenced::new(person_bob()))?;
    tx.try_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;
    assert_eq!(counter.load(Ordering::SeqCst), 0);

    Ok(())
}

#[tokio::test]
async fn test_tap_complex_pipeline() -> anyhow::Result<()> {
    // Arrange
    let observed_before_filter = Arc::new(Mutex::new(Vec::new()));
    let observed_before_filter_clone = observed_before_filter.clone();
    let observed_after_transform = Arc::new(Mutex::new(Vec::new()));
    let observed_after_transform_clone = observed_after_transform.clone();

    let (tx, stream) = test_channel();
    let mut result = stream
        .tap(move |value: &TestData| {
            observed_before_filter_clone.lock().push(value.clone());
        })
        .filter_ordered(|item| matches!(item, TestData::Person(_)))
        .map_ordered(|item: Sequenced<TestData>| {
            Sequenced::new(match item.value {
                TestData::Person(p) => TestData::Person(Person {
                    name: p.name.to_uppercase(),
                    ..p
                }),
                other => other,
            })
        })
        .tap(move |value: &TestData| {
            observed_after_transform_clone.lock().push(value.clone());
        });

    // Act - send items and consume results
    // Only Person items pass through filter, others are dropped
    tx.try_send(Sequenced::new(person_alice()))?; // passes filter
    tx.try_send(Sequenced::new(person_bob()))?; // passes filter
    drop(tx); // Close to end stream

    // Consume all items that pass the filter
    unwrap_stream(&mut result, 500).await; // alice
    unwrap_stream(&mut result, 500).await; // bob

    // Assert
    // Before filter tap sees only the items that were polled through
    // (filter drops non-matching items synchronously, tap still fires)
    let before = observed_before_filter.lock();
    assert_eq!(*before, [person_alice(), person_bob()]); // Only persons consumed

    let after = observed_after_transform.lock();
    assert_eq!(after.len(), 2); // Only persons after filter

    // Verify transformation was applied
    assert!(matches!(
        &after[0],
        TestData::Person(p) if p.name == p.name.to_uppercase()
    ));

    Ok(())
}
