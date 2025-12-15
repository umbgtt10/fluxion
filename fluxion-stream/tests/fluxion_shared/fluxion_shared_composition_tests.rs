// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition tests for `FluxionShared` with other operators.

use fluxion_core::Timestamped;
use fluxion_stream::FluxionStream;
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{TestData, person_alice, person_bob, person_charlie, person_diane};
use fluxion_test_utils::{test_channel, unwrap_stream, unwrap_value, Sequenced};

#[tokio::test]
async fn shared_with_filter_ordered_each_subscriber_filters_independently() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = FluxionStream::new(rx);

    let shared = source.share();

    // Two subscribers with different filter predicates
    let mut adults_only = FluxionStream::new(shared.subscribe().unwrap())
        .filter_ordered(|data| matches!(data, TestData::Person(p) if p.age >= 30));

    let mut young_only = FluxionStream::new(shared.subscribe().unwrap())
        .filter_ordered(|data| matches!(data, TestData::Person(p) if p.age < 30));

    // Act - send people of various ages
    tx.send(Sequenced::new(person_alice()))?; // 25 - young
    tx.send(Sequenced::new(person_bob()))?; // 30 - adult
    tx.send(Sequenced::new(person_charlie()))?; // 35 - adult

    // Assert - adults_only sees Bob and Charlie
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut adults_only, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Bob"
    ));
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut adults_only, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Charlie"
    ));

    // Assert - young_only sees Alice
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut young_only, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Alice"
    ));

    Ok(())
}

#[tokio::test]
async fn shared_with_map_ordered_transforms_per_subscriber() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = FluxionStream::new(rx);

    let shared = source.share();

    // Two subscribers with different map transformations
    let mut add_10_years = FluxionStream::new(shared.subscribe().unwrap()).map_ordered(|data| {
        let updated = match data.into_inner() {
            TestData::Person(p) => TestData::Person(Person::new(p.name, p.age + 10)),
            other => other,
        };
        Sequenced::new(updated)
    });

    let mut double_age = FluxionStream::new(shared.subscribe().unwrap()).map_ordered(|data| {
        let updated = match data.into_inner() {
            TestData::Person(p) => TestData::Person(Person::new(p.name, p.age * 2)),
            other => other,
        };
        Sequenced::new(updated)
    });

    // Act
    tx.send(Sequenced::new(person_alice()))?; // Alice, 25

    // Assert - add_10_years sees Alice at 35
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut add_10_years, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Alice" && p.age == 35
    ));

    // Assert - double_age sees Alice at 50
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut double_age, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Alice" && p.age == 50
    ));

    Ok(())
}

#[tokio::test]
async fn shared_with_combine_with_previous_tracks_state_per_subscriber() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = FluxionStream::new(rx);

    let shared = source.share();

    // Two subscribers both using combine_with_previous
    let mut sub1 = FluxionStream::new(shared.subscribe().unwrap()).combine_with_previous();
    let mut sub2 = FluxionStream::new(shared.subscribe().unwrap()).combine_with_previous();

    // Act - send two items
    tx.send(Sequenced::new(person_alice()))?;
    tx.send(Sequenced::new(person_bob()))?;

    // Assert sub1 - first item has no previous
    let first1 = unwrap_value(Some(unwrap_stream(&mut sub1, 500).await));
    assert!(!first1.has_previous());
    assert!(matches!(
        first1.current.into_inner(),
        TestData::Person(ref p) if p.name == "Alice"
    ));

    // Assert sub1 - second item has Alice as previous
    let second1 = unwrap_value(Some(unwrap_stream(&mut sub1, 500).await));
    assert!(second1.has_previous());
    assert!(matches!(
        second1.current.into_inner(),
        TestData::Person(ref p) if p.name == "Bob"
    ));
    assert!(matches!(
        second1.previous.unwrap().into_inner(),
        TestData::Person(ref p) if p.name == "Alice"
    ));

    // Assert sub2 sees the same pattern independently
    let first2 = unwrap_value(Some(unwrap_stream(&mut sub2, 500).await));
    assert!(!first2.has_previous());

    let second2 = unwrap_value(Some(unwrap_stream(&mut sub2, 500).await));
    assert!(second2.has_previous());

    Ok(())
}

#[tokio::test]
async fn shared_with_scan_ordered_accumulates_per_subscriber() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = FluxionStream::new(rx);

    let shared = source.share();

    // Two subscribers with different scan accumulators
    let mut sum_ages = FluxionStream::new(shared.subscribe().unwrap()).scan_ordered(
        0u32,
        |acc, data: &TestData| {
            if let TestData::Person(p) = data {
                *acc += p.age;
            }
            *acc
        },
    );

    let mut count_people = FluxionStream::new(shared.subscribe().unwrap()).scan_ordered(
        0u32,
        |acc, data: &TestData| {
            if matches!(data, TestData::Person(_)) {
                *acc += 1;
            }
            *acc
        },
    );

    // Act
    tx.send(Sequenced::new(person_alice()))?; // age 25
    tx.send(Sequenced::new(person_bob()))?; // age 30
    tx.send(Sequenced::new(person_charlie()))?; // age 35

    // Assert sum_ages: 25, 55, 90
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut sum_ages, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 25);
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut sum_ages, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 55);
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut sum_ages, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 90);

    // Assert count_people: 1, 2, 3
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut count_people, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 1);
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut count_people, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 2);
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut count_people, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 3);

    Ok(())
}

#[tokio::test]
async fn shared_with_mixed_combine_latest_combines_subscribers() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = test_channel::<Sequenced<TestData>>();
    let (tx2, rx2) = test_channel::<Sequenced<TestData>>();
    let source = FluxionStream::new(rx1);
    let stream = FluxionStream::new(rx2);

    let shared = source.share();

    // Two subscribers from the same shared source, combined with combine_latest
    let mut combined =
        FluxionStream::new(shared.subscribe().unwrap()).combine_latest(vec![stream], |_| true);

    // Act - send items
    tx1.send(Sequenced::new(person_charlie()))?;
    tx2.send(Sequenced::new(person_alice()))?;

    // Assert - combine_latest emits when both have values
    // After Alice is sent, both subscribers receive it, so we get (Alice, Alice)
    assert_eq!(
        unwrap_stream(&mut combined, 500)
            .await
            .unwrap()
            .clone()
            .into_inner()
            .values()
            .clone(),
        vec![person_charlie(), person_alice()]
    );

    // Send Bob
    tx2.send(Sequenced::new(person_bob()))?;

    // After Bob is sent, we get a combination with Bob on one side
    assert_eq!(
        unwrap_stream(&mut combined, 500)
            .await
            .unwrap()
            .clone()
            .into_inner()
            .values()
            .clone(),
        vec![person_charlie(), person_bob()]
    );

    // Send Bob
    tx1.send(Sequenced::new(person_diane()))?;

    // After Bob is sent, we get a combination with Bob on one side
    assert_eq!(
        unwrap_stream(&mut combined, 500)
            .await
            .unwrap()
            .clone()
            .into_inner()
            .values()
            .clone(),
        vec![person_diane(), person_bob()]
    );

    Ok(())
}

#[tokio::test]
async fn shared_with_transientcombine_latest_combines_subscribers() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = FluxionStream::new(rx);

    let shared = source.share();

    // Two subscribers from the same shared source, combined with combine_latest
    let mut combined = FluxionStream::new(shared.subscribe().unwrap())
        .combine_latest(vec![shared.subscribe().unwrap()], |_| true);

    // Act - send items
    tx.send(Sequenced::new(person_alice()))?;

    // Assert - combine_latest emits when both have values
    // After Alice is sent, both subscribers receive it, so we get (Alice, Alice)
    assert_eq!(
        unwrap_stream(&mut combined, 500)
            .await
            .unwrap()
            .clone()
            .into_inner()
            .values()
            .clone(),
        vec![person_alice(), person_alice()]
    );

    // Send Bob
    tx.send(Sequenced::new(person_bob()))?;

    // After Bob is sent, we get a combination with Bob on one side
    assert_eq!(
        unwrap_stream(&mut combined, 500)
            .await
            .unwrap()
            .clone()
            .into_inner()
            .values()
            .clone(),
        vec![person_bob(), person_alice()]
    );

    assert_eq!(
        unwrap_stream(&mut combined, 500)
            .await
            .unwrap()
            .clone()
            .into_inner()
            .values()
            .clone(),
        vec![person_bob(), person_bob()]
    );

    Ok(())
}
