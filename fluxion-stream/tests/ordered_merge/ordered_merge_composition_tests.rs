// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::into_stream::IntoStream;
use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::{FilterOrderedExt, MapOrderedExt, OrderedStreamExt};
use fluxion_test_utils::helpers::{assert_stream_ended, test_channel, unwrap_stream};
use fluxion_test_utils::sequenced::Sequenced;
use fluxion_test_utils::test_data::{
    animal, animal_dog, animal_spider, person, person_alice, person_bob, plant_rose,
    plant_sunflower, TestData,
};

#[tokio::test]
async fn test_ordered_merge_then_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1
        .ordered_merge(vec![s2])
        .filter_ordered(|item| matches!(item, TestData::Person(_)));

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_bob(), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_spider(), 4))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &person_alice())
    );
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &person_bob()));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_map() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).map_ordered(|item| {
        let ts = item.timestamp();
        let name = match item.into_inner() {
            TestData::Person(p) => p.name,
            TestData::Animal(a) => a.species,
            TestData::Plant(pl) => pl.species,
        };
        Sequenced::with_timestamp(name, ts)
    });

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(plant_rose(), 3))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &"Alice".to_string())
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &"Dog".to_string())
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &"Rose".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();
    let (tx3, s3) = test_channel::<Sequenced<TestData>>();

    let merged_12 = s1.ordered_merge(vec![s2]);

    let mut result = merged_12.ordered_merge(vec![s3]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx3.unbounded_send(Sequenced::with_timestamp(plant_rose(), 2))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 3))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_bob(), 4))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &person_alice())
    );
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &plant_rose()));
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &animal_dog()));
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &person_bob()));

    Ok(())
}

#[tokio::test]
async fn test_filter_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let filtered1 = s1.filter_ordered(|item| matches!(item, TestData::Animal(_)));
    let filtered2 = s2.filter_ordered(|item| matches!(item, TestData::Animal(_)));

    let mut result = filtered1.ordered_merge(vec![filtered2]);

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(animal_spider(), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(plant_rose(), 4))?;

    // Assert
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &animal_dog()));
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &animal_spider())
    );

    Ok(())
}

#[tokio::test]
async fn test_map_then_ordered_merge() -> anyhow::Result<()> {
    // Arrange
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
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_bob(), 3))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &"Person: Alice".to_string())
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &"Animal: Dog".to_string())
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &"Person: Bob".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_with_complex_pipeline() -> anyhow::Result<()> {
    // Arrange
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
    tx1.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person_bob(), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(plant_sunflower(), 4))?;

    // Assert
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if *v == 30));
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if *v == 180));

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
    person_tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 1))?;
    animal_tx.unbounded_send(Sequenced::with_timestamp(animal_dog(), 2))?;
    person_tx.unbounded_send(Sequenced::with_timestamp(person_bob(), 3))?;
    animal_tx.unbounded_send(Sequenced::with_timestamp(animal_spider(), 4))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &person_alice())
    );
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == &person_bob()));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_preserves_ordering_through_map() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).map_ordered(|x| {
        let ts = x.timestamp();
        let name = match x.into_inner() {
            TestData::Person(p) => p.name,
            TestData::Animal(a) => a.species,
            TestData::Plant(pl) => pl.species,
        };
        Sequenced::with_timestamp(name, ts)
    });

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person("P3".to_string(), 30), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P1".to_string(), 10), 1))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person("P4".to_string(), 40), 4))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P2".to_string(), 20), 2))?;

    // Assert
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == "P1"));
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == "P2"));
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == "P3"));
    assert!(matches!(&unwrap_stream(&mut result, 200).await.into_inner(), v if v == "P4"));

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
    tx1.unbounded_send(Sequenced::with_timestamp(person("P1".to_string(), 1), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P2".to_string(), 3), 2))?;
    tx3.unbounded_send(Sequenced::with_timestamp(person("P3".to_string(), 5), 3))?;
    tx1.unbounded_send(Sequenced::with_timestamp(animal("A1".to_string(), 6), 4))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P4".to_string(), 7), 5))?;
    tx3.unbounded_send(Sequenced::with_timestamp(person("P5".to_string(), 9), 6))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), TestData::Person(ref p) if p.age == 3)
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), TestData::Animal(ref a) if a.legs == 6)
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), TestData::Person(ref p) if p.age == 9)
    );

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_empty_after_filter() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).filter_ordered(|x| match x {
        TestData::Person(p) => p.age > 100,
        TestData::Animal(a) => a.legs > 100,
        _ => false,
    });

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person("P1".to_string(), 1), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P2".to_string(), 2), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person("P3".to_string(), 3), 3))?;

    drop(tx1);
    drop(tx2);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_map_preserves_timestamps() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let mut result = s1.ordered_merge(vec![s2]).map_ordered(|x| {
        let ts = x.timestamp();
        let doubled = match x.into_inner() {
            TestData::Person(p) => person(p.name, p.age * 2),
            TestData::Animal(a) => animal(a.species, a.legs * 2),
            other => other,
        };
        Sequenced::with_timestamp(doubled, ts)
    });

    // Act
    tx1.unbounded_send(Sequenced::with_timestamp(person("P1".to_string(), 10), 5))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P2".to_string(), 20), 10))?;

    // Assert
    let first = unwrap_stream(&mut result, 200).await;
    assert_eq!(first.timestamp(), 5);
    assert!(matches!(&first.into_inner(), TestData::Person(ref p) if p.age == 20));

    let second = unwrap_stream(&mut result, 200).await;
    assert_eq!(second.timestamp(), 10);
    assert!(matches!(&second.into_inner(), TestData::Person(ref p) if p.age == 40));

    Ok(())
}

#[tokio::test]
async fn test_ordered_merge_filter_map_chain() -> anyhow::Result<()> {
    // Arrange
    let (tx1, s1) = test_channel::<Sequenced<TestData>>();
    let (tx2, s2) = test_channel::<Sequenced<TestData>>();

    let filtered1 = s1.filter_ordered(|x| match x {
        TestData::Person(p) => p.age % 2 == 0,
        TestData::Animal(a) => a.legs % 2 == 0,
        _ => false,
    });
    let filtered2 = s2.filter_ordered(|x| match x {
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
    tx1.unbounded_send(Sequenced::with_timestamp(person("P1".to_string(), 1), 1))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P2".to_string(), 2), 2))?;
    tx1.unbounded_send(Sequenced::with_timestamp(animal("A1".to_string(), 4), 3))?;
    tx2.unbounded_send(Sequenced::with_timestamp(person("P3".to_string(), 5), 4))?;
    tx1.unbounded_send(Sequenced::with_timestamp(person("P4".to_string(), 6), 5))?;

    // Assert
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), TestData::Person(ref p) if p.age == 1)
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), TestData::Animal(ref a) if a.legs == 2)
    );
    assert!(
        matches!(&unwrap_stream(&mut result, 200).await.into_inner(), TestData::Person(ref p) if p.age == 3)
    );

    Ok(())
}
