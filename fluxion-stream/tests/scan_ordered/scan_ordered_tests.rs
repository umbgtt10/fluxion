// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{HasTimestamp, StreamItem};
use fluxion_stream::ScanOrderedExt;
use fluxion_test_utils::{
    assert_stream_ended,
    helpers::unwrap_stream,
    test_channel,
    test_data::{person_alice, person_bob, person_charlie, person_dave, TestData},
    unwrap_value, Sequenced,
};

#[tokio::test]
async fn test_scan_ordered_count_people() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    let mut counts = stream.scan_ordered(0, |count: &mut i32, _: &TestData| {
        *count += 1;
        *count
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    let result = unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 1);

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    let result = unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 2);

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    let result = unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 3);

    drop(tx);
    assert_stream_ended(&mut counts, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_collect_names() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut names = stream.scan_ordered(Vec::<String>::new(), |list, data| {
        if let TestData::Person(p) = data {
            list.push(p.name.clone());
        }
        list.clone()
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<Vec<String>>, _>(&mut names, 500).await,
    ));
    assert_eq!(result.into_inner(), vec!["Alice"]);

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    let result = unwrap_stream::<Sequenced<Vec<String>>, _>(&mut names, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), vec!["Alice", "Bob"]);

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    let result = unwrap_stream::<Sequenced<Vec<String>>, _>(&mut names, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), vec!["Alice", "Bob", "Charlie"]);

    drop(tx);
    assert_stream_ended(&mut names, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_total_age() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut total_ages = stream.scan_ordered(0u32, |total, data| {
        if let TestData::Person(p) = data {
            *total += p.age;
        }
        *total
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age 25
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<u32>, _>(&mut total_ages, 500).await,
    ));
    assert_eq!(result.into_inner(), 25);

    tx.unbounded_send(Sequenced::new(person_bob()))?; // age 30
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut total_ages, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 55);

    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age 35
    let result = unwrap_stream::<Sequenced<u32>, _>(&mut total_ages, 500)
        .await
        .unwrap();
    assert_eq!(result.into_inner(), 90);

    drop(tx);
    assert_stream_ended(&mut total_ages, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_format_summary() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut summaries = stream.scan_ordered(0, |count: &mut i32, data: &TestData| {
        *count += 1;
        format!("Item #{}: {}", count, data)
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<String>, _>(&mut summaries, 500).await,
    ));
    assert_eq!(result.into_inner(), "Item #1: Person[name=Alice, age=25]");

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    let result = unwrap_stream(&mut summaries, 500).await.unwrap();
    assert_eq!(result.into_inner(), "Item #2: Person[name=Bob, age=30]");

    drop(tx);
    assert_stream_ended(&mut summaries, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_min_max_age() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut age_range = stream.scan_ordered((u32::MAX, u32::MIN), |state, data| {
        if let TestData::Person(p) = data {
            state.0 = state.0.min(p.age);
            state.1 = state.1.max(p.age);
        }
        *state
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_bob()))?; // age 30
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<(u32, u32)>, _>(&mut age_range, 500).await,
    ));
    assert_eq!(result.into_inner(), (30, 30));

    tx.unbounded_send(Sequenced::new(person_alice()))?; // age 25
    let result = unwrap_stream(&mut age_range, 500).await.unwrap();
    assert_eq!(result.into_inner(), (25, 30));

    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age 35
    let result = unwrap_stream(&mut age_range, 500).await.unwrap();
    assert_eq!(result.into_inner(), (25, 35));

    drop(tx);
    assert_stream_ended(&mut age_range, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_empty_stream() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut counts = stream.scan_ordered(0, |count, _| {
        *count += 1;
        *count
    });

    // Act: Drop sender immediately
    drop(tx);

    // Assert: Stream should end without emitting
    assert_stream_ended::<_, StreamItem<Sequenced<i32>>>(&mut counts, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_single_element() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut counts = stream.scan_ordered(100, |count, _| {
        *count += 1;
        *count
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500).await,
    ));
    assert_eq!(result.into_inner(), 101);

    drop(tx);
    assert_stream_ended(&mut counts, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_timestamp_preservation() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut counts = stream.scan_ordered(0, |count, _| {
        *count += 1;
        *count
    });

    // Act
    tx.unbounded_send(Sequenced::with_timestamp(person_alice(), 100))?;
    let result1 = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500).await,
    ));

    tx.unbounded_send(Sequenced::with_timestamp(person_bob(), 200))?;
    let result2 = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500).await,
    ));

    tx.unbounded_send(Sequenced::with_timestamp(person_charlie(), 300))?;
    let result3 = unwrap_value(Some(
        unwrap_stream::<Sequenced<i32>, _>(&mut counts, 500).await,
    ));

    // Assert: Timestamps are preserved from source
    assert_eq!(result1.timestamp(), 100);
    assert_eq!(result1.into_inner(), 1);

    assert_eq!(result2.timestamp(), 200);
    assert_eq!(result2.into_inner(), 2);

    assert_eq!(result3.timestamp(), 300);
    assert_eq!(result3.into_inner(), 3);

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_build_roster() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut roster = stream.scan_ordered(String::new(), |acc, data| {
        if !acc.is_empty() {
            acc.push_str(", ");
        }
        acc.push_str(&data.to_string());
        acc.clone()
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<String>, _>(&mut roster, 500).await,
    ));
    assert_eq!(result.into_inner(), "Person[name=Alice, age=25]");

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    let result = unwrap_stream(&mut roster, 500).await.unwrap();
    assert_eq!(
        result.into_inner(),
        "Person[name=Alice, age=25], Person[name=Bob, age=30]"
    );

    drop(tx);
    assert_stream_ended(&mut roster, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_average_age() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut averages =
        stream.scan_ordered((0u32, 0u32), |state: &mut (u32, u32), data: &TestData| {
            if let TestData::Person(p) = data {
                state.0 += p.age;
                state.1 += 1;
            }
            if state.1 > 0 {
                state.0 / state.1
            } else {
                0
            }
        });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?; // age 25
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<u32>, _>(&mut averages, 500).await,
    ));
    assert_eq!(result.into_inner(), 25);

    tx.unbounded_send(Sequenced::new(person_bob()))?; // age 30
    let result = unwrap_stream(&mut averages, 500).await.unwrap();
    assert_eq!(result.into_inner(), 27); // (25+30)/2

    tx.unbounded_send(Sequenced::new(person_charlie()))?; // age 35
    let result = unwrap_stream(&mut averages, 500).await.unwrap();
    assert_eq!(result.into_inner(), 30); // (25+30+35)/3

    drop(tx);
    assert_stream_ended(&mut averages, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_scan_ordered_multiple_people() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<TestData>>();
    let mut history = stream.scan_ordered(Vec::<TestData>::new(), |list, data| {
        list.push(data.clone());
        list.clone()
    });

    // Act & Assert
    tx.unbounded_send(Sequenced::new(person_alice()))?;
    let result = unwrap_value(Some(
        unwrap_stream::<Sequenced<Vec<TestData>>, _>(&mut history, 500).await,
    ));
    assert_eq!(result.into_inner(), vec![person_alice()]);

    tx.unbounded_send(Sequenced::new(person_bob()))?;
    let result = unwrap_stream(&mut history, 500).await.unwrap();
    assert_eq!(result.into_inner(), vec![person_alice(), person_bob()]);

    tx.unbounded_send(Sequenced::new(person_charlie()))?;
    let result = unwrap_stream(&mut history, 500).await.unwrap();
    assert_eq!(
        result.into_inner(),
        vec![person_alice(), person_bob(), person_charlie()]
    );

    tx.unbounded_send(Sequenced::new(person_dave()))?;
    let result = unwrap_stream(&mut history, 500).await.unwrap();
    assert_eq!(
        result.into_inner(),
        vec![
            person_alice(),
            person_bob(),
            person_charlie(),
            person_dave()
        ]
    );

    drop(tx);
    assert_stream_ended(&mut history, 100).await;

    Ok(())
}
