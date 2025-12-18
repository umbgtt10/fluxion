// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! Composition tests for `ordered_merge` operator.
//!
//! Tests combining ordered_merge with other operators.

use fluxion_core::into_stream::IntoStream;
use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::{FilterOrderedExt, MapOrderedExt, OrderedStreamExt};
use fluxion_test_utils::test_data::{
    animal, animal_dog, animal_spider, person, person_alice, person_bob, plant_rose,
    plant_sunflower, TestData,
};
use fluxion_test_utils::{assert_stream_ended, test_channel, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_ordered_merge_then_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .into_stream()
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act: Mix person and animal data
    tx1.send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.send(Sequenced::with_timestamp(person_bob(), 3))?;
    tx2.send(Sequenced::with_timestamp(animal_spider(), 4))?;

    // Assert: Only persons should pass through
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        person_bob()
    );

    drop(tx1);
    drop(tx2);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_map() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .into_stream()
        .map_ordered(|item| {
            let ts = item.timestamp();
            let name = match item.into_inner() {
                TestData::Person(p) => p.name,
                TestData::Animal(a) => a.species,
                TestData::Plant(pl) => pl.species,
            };
            Sequenced::with_timestamp(name, ts)
        });

    // Act
    tx1.send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.send(Sequenced::with_timestamp(plant_rose(), 3))?;

    // Assert: Values should be mapped to names/species
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        "Alice".to_string()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        "Dog".to_string()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        "Rose".to_string()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange: Chain two ordered_merge operations
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();
    let (tx3, s3) = test_channel::<Sequenced<TestData>>();

    // First merge s1 and s2
    let merged_12 = s1.ordered_merge(vec![s2]);

    // Then merge result with s3
    let mut result = merged_12.ordered_merge(vec![s3]);

    // Act: Send interleaved timestamps
    tx1.send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx3.send(Sequenced::with_timestamp(plant_rose(), 2))?;
    tx2.send(Sequenced::with_timestamp(animal_dog(), 3))?;
    tx1.send(Sequenced::with_timestamp(person_bob(), 4))?;

    // Assert: All values emitted in timestamp order
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        plant_rose()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        animal_dog()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange: Filter before merging
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let filtered1 = s1.filter_ordered(|item| matches!(item, TestData::Animal(_)));
    let filtered2 = s2.filter_ordered(|item| matches!(item, TestData::Animal(_)));

    let mut result = filtered1.ordered_merge(vec![filtered2]);

    // Act
    tx1.send(Sequenced::with_timestamp(person_alice(), 1))?; // filtered out
    tx2.send(Sequenced::with_timestamp(animal_dog(), 2))?; // passes
    tx1.send(Sequenced::with_timestamp(animal_spider(), 3))?; // passes
    tx2.send(Sequenced::with_timestamp(plant_rose(), 4))?; // filtered out

    // Assert: Only animals should be merged
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        animal_dog()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        animal_spider()
    );

    drop(tx1);
    drop(tx2);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_map_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange: Map before merging
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mapped1 = s1.map_ordered(|item| {
        let ts = item.timestamp();
        let name = match item.into_inner() {
            TestData::Person(p) => format!("Person: {}", p.name),
            _ => "Other".to_string(),
        };
        Sequenced::with_timestamp(name, ts)
    });
    let mapped2 = s2.map_ordered(|item| {
        let ts = item.timestamp();
        let name = match item.into_inner() {
            TestData::Animal(a) => format!("Animal: {}", a.species),
            _ => "Other".to_string(),
        };
        Sequenced::with_timestamp(name, ts)
    });

    let mut result = mapped1.ordered_merge(vec![mapped2]);

    // Act
    tx1.send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.send(Sequenced::with_timestamp(person_bob(), 3))?;

    // Assert: Values should be transformed before merging
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        "Person: Alice".to_string()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        "Animal: Dog".to_string()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        "Person: Bob".to_string()
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_with_complex_pipeline() -> anyhow::Result<()> {
    // Arrange: ordered_merge -> map -> filter
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .map_ordered(|item| {
            let ts = item.timestamp();
            let val = match item.into_inner() {
                TestData::Person(p) => p.age,
                TestData::Animal(a) => a.legs,
                TestData::Plant(pl) => pl.height,
            };
            Sequenced::with_timestamp(val, ts)
        })
        .filter_ordered(|x| *x > 25);

    // Act
    tx1.send(Sequenced::with_timestamp(person_alice(), 1))?; // age 25, filtered
    tx2.send(Sequenced::with_timestamp(animal_dog(), 2))?; // legs 4, filtered
    tx1.send(Sequenced::with_timestamp(person_bob(), 3))?; // age 30, passes
    tx2.send(Sequenced::with_timestamp(plant_sunflower(), 4))?; // height 180, passes

    // Assert
    assert_eq!(unwrap_stream(&mut result, 200).await.into_inner(), 30);
    assert_eq!(unwrap_stream(&mut result, 200).await.into_inner(), 180);

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_mixed_testdata_then_filter() -> anyhow::Result<()> {
    // Arrange
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();

    let mut result = person_stream
        .ordered_merge(vec![animal_stream])
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act
    person_tx.send(Sequenced::with_timestamp(person_alice(), 1))?;
    animal_tx.send(Sequenced::with_timestamp(animal_dog(), 2))?;
    person_tx.send(Sequenced::with_timestamp(person_bob(), 3))?;
    animal_tx.send(Sequenced::with_timestamp(animal_spider(), 4))?;

    // Assert: Only persons should pass through
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_stream(&mut result, 200).await.into_inner(),
        person_bob()
    );

    drop(person_tx);
    drop(animal_tx);
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_preserves_ordering_through_map() -> anyhow::Result<()> {
    // Arrange: Verify that timestamp ordering is preserved through transformations
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).into_stream().map_ordered(|x| {
        let ts = x.timestamp();
        let name = match x.into_inner() {
            TestData::Person(p) => p.name,
            TestData::Animal(a) => a.species,
            TestData::Plant(pl) => pl.species,
        };
        Sequenced::with_timestamp(name, ts)
    });

    // Act: Send out of order
    tx1.send(Sequenced::with_timestamp(person("P3".to_string(), 30), 3))?;
    tx2.send(Sequenced::with_timestamp(person("P1".to_string(), 10), 1))?;
    tx1.send(Sequenced::with_timestamp(person("P4".to_string(), 40), 4))?;
    tx2.send(Sequenced::with_timestamp(person("P2".to_string(), 20), 2))?;

    // Assert: Should be in timestamp order after mapping
    assert_eq!(unwrap_stream(&mut result, 200).await.into_inner(), "P1");
    assert_eq!(unwrap_stream(&mut result, 200).await.into_inner(), "P2");
    assert_eq!(unwrap_stream(&mut result, 200).await.into_inner(), "P3");
    assert_eq!(unwrap_stream(&mut result, 200).await.into_inner(), "P4");

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_three_streams_with_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();
    let (tx3, s3) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2, s3]).filter_ordered(|x| match x {
        TestData::Person(p) => p.age % 3 == 0,
        TestData::Animal(a) => a.legs % 3 == 0,
        _ => false,
    });

    // Act
    tx1.send(Sequenced::with_timestamp(person("P1".to_string(), 1), 1))?;
    tx2.send(Sequenced::with_timestamp(person("P2".to_string(), 3), 2))?; // divisible by 3
    tx3.send(Sequenced::with_timestamp(person("P3".to_string(), 5), 3))?;
    tx1.send(Sequenced::with_timestamp(animal("A1".to_string(), 6), 4))?; // divisible by 3
    tx2.send(Sequenced::with_timestamp(person("P4".to_string(), 7), 5))?;
    tx3.send(Sequenced::with_timestamp(person("P5".to_string(), 9), 6))?; // divisible by 3

    // Assert: Only multiples of 3
    let item1 = unwrap_stream(&mut result, 200).await.into_inner();
    if let TestData::Person(p) = item1 {
        assert_eq!(p.age, 3);
    }
    let item2 = unwrap_stream(&mut result, 200).await.into_inner();
    if let TestData::Animal(a) = item2 {
        assert_eq!(a.legs, 6);
    }
    let item3 = unwrap_stream(&mut result, 200).await.into_inner();
    if let TestData::Person(p) = item3 {
        assert_eq!(p.age, 9);
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_empty_after_filter() -> anyhow::Result<()> {
    // Arrange: All items filtered out
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).filter_ordered(|x| match x {
        TestData::Person(p) => p.age > 100,
        TestData::Animal(a) => a.legs > 100,
        _ => false,
    });

    // Act: Send values all below threshold
    tx1.send(Sequenced::with_timestamp(person("P1".to_string(), 1), 1))?;
    tx2.send(Sequenced::with_timestamp(person("P2".to_string(), 2), 2))?;
    tx1.send(Sequenced::with_timestamp(person("P3".to_string(), 3), 3))?;

    drop(tx1);
    drop(tx2);

    // Assert: Stream should end without emitting
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_preserves_timestamps() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).into_stream().map_ordered(|x| {
        let ts = x.timestamp();
        let doubled = match x.into_inner() {
            TestData::Person(p) => person(p.name, p.age * 2),
            TestData::Animal(a) => animal(a.species, a.legs * 2),
            other => other,
        };
        Sequenced::with_timestamp(doubled, ts)
    });

    // Act
    tx1.send(Sequenced::with_timestamp(person("P1".to_string(), 10), 5))?;
    tx2.send(Sequenced::with_timestamp(person("P2".to_string(), 20), 10))?;

    // Assert: Timestamps should be preserved after map
    let first = unwrap_stream(&mut result, 200).await;
    assert_eq!(first.timestamp(), 5);
    if let TestData::Person(p) = first.into_inner() {
        assert_eq!(p.age, 20);
    }

    let second = unwrap_stream(&mut result, 200).await;
    assert_eq!(second.timestamp(), 10);
    if let TestData::Person(p) = second.into_inner() {
        assert_eq!(p.age, 40);
    }

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_filter_map_chain() -> anyhow::Result<()> {
    // Arrange: filter -> ordered_merge -> map
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let filtered1 = s1.into_stream().filter_ordered(|x| match x {
        TestData::Person(p) => p.age % 2 == 0,
        TestData::Animal(a) => a.legs % 2 == 0,
        _ => false,
    });
    let filtered2 = s2.into_stream().filter_ordered(|x| match x {
        TestData::Person(p) => p.age % 2 == 0,
        TestData::Animal(a) => a.legs % 2 == 0,
        _ => false,
    });

    let mut result = filtered1
        .ordered_merge(vec![filtered2])
        .into_stream()
        .map_ordered(|x| {
            let ts = x.timestamp();
            let halved = match x.into_inner() {
                TestData::Person(p) => person(p.name, p.age / 2),
                TestData::Animal(a) => animal(a.species, a.legs / 2),
                other => other,
            };
            Sequenced::with_timestamp(halved, ts)
        });

    // Act
    tx1.send(Sequenced::with_timestamp(person("P1".to_string(), 1), 1))?; // odd, filtered
    tx2.send(Sequenced::with_timestamp(person("P2".to_string(), 2), 2))?; // even -> 1
    tx1.send(Sequenced::with_timestamp(animal("A1".to_string(), 4), 3))?; // even -> 2
    tx2.send(Sequenced::with_timestamp(person("P3".to_string(), 5), 4))?; // odd, filtered
    tx1.send(Sequenced::with_timestamp(person("P4".to_string(), 6), 5))?; // even -> 3

    // Assert
    let item1 = unwrap_stream(&mut result, 200).await.into_inner();
    if let TestData::Person(p) = item1 {
        assert_eq!(p.age, 1);
    }
    let item2 = unwrap_stream(&mut result, 200).await.into_inner();
    if let TestData::Animal(a) = item2 {
        assert_eq!(a.legs, 2);
    }
    let item3 = unwrap_stream(&mut result, 200).await.into_inner();
    if let TestData::Person(p) = item3 {
        assert_eq!(p.age, 3);
    }

    Ok(())
}
