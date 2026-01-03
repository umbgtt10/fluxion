// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::{FluxionError, StreamItem, SubjectError};
use fluxion_stream::prelude::*;
use fluxion_stream::ShareExt;
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{
    person_alice, person_bob, person_charlie, person_diane, TestData,
};
use fluxion_test_utils::{
    assert_stream_ended, test_channel, test_channel_with_errors, unwrap_stream, unwrap_value,
    Sequenced,
};
use futures::StreamExt;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[tokio::test]
async fn share_broadcasts_to_multiple_subscribers() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub1 = shared.subscribe().unwrap();
    let mut sub2 = shared.subscribe().unwrap();

    // Act
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap();

    // Assert - both subscribers receive the same value
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub1, 500).await)).into_inner(),
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub2, 500).await)).into_inner(),
        person_alice()
    );
}

#[tokio::test]
async fn share_completes_subscribers_when_source_completes() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub1 = shared.subscribe().unwrap();

    // Act - drop sender to complete source
    tx.unbounded_send(Sequenced::new(person_bob())).unwrap();
    drop(tx);

    // Assert - subscriber receives value then completes
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub1, 500).await)).into_inner(),
        person_bob()
    );
    assert_stream_ended(&mut sub1, 500).await;
}

#[tokio::test]
async fn share_propagates_errors_to_all_subscribers() {
    // Arrange
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub1 = shared.subscribe().unwrap();
    let mut sub2 = shared.subscribe().unwrap();

    // Act - send an error
    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error("test error")))
        .unwrap();

    // Assert - both subscribers receive the error
    assert!(matches!(
        unwrap_stream(&mut sub1, 500).await,
        StreamItem::Error(_)
    ));
    assert!(matches!(
        unwrap_stream(&mut sub2, 500).await,
        StreamItem::Error(_)
    ));

    // Assert - subscribers complete after error
    assert_stream_ended(&mut sub1, 500).await;
    assert_stream_ended(&mut sub2, 500).await;
}

#[tokio::test]
async fn late_subscriber_does_not_receive_past_items() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();

    // Need an initial subscriber to consume items, otherwise they're buffered
    let mut _consumer = shared.subscribe().unwrap();

    // Send before late subscriber subscribes
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap();
    tx.unbounded_send(Sequenced::new(person_bob())).unwrap();

    // Consume items to ensure they're processed
    let _ = unwrap_stream(&mut _consumer, 500).await;
    let _ = unwrap_stream(&mut _consumer, 500).await;

    // Late subscriber
    let mut late_sub = shared.subscribe().unwrap();

    // Send after subscribing
    tx.unbounded_send(Sequenced::new(person_charlie())).unwrap();

    // Assert - late subscriber only sees Charlie
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut late_sub, 500).await)).into_inner(),
        person_charlie()
    );
}

#[tokio::test]
async fn subscriber_count_tracks_active_subscribers() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();

    // Assert - no subscribers initially
    assert_eq!(shared.subscriber_count(), 0);

    // Add subscribers
    let mut sub1 = shared.subscribe().unwrap();
    assert_eq!(shared.subscriber_count(), 1);

    let sub2 = shared.subscribe().unwrap();
    assert_eq!(shared.subscriber_count(), 2);

    // Drop one subscriber and send to trigger cleanup
    drop(sub2);
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap();

    // Consume to trigger cleanup
    let _ = unwrap_stream(&mut sub1, 500).await;

    // Count decreases after cleanup
    assert_eq!(shared.subscriber_count(), 1);
}

#[tokio::test]
async fn is_closed_reflects_state() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub = shared.subscribe().unwrap();

    // Assert - initially open
    assert!(!shared.is_closed());

    // Act - complete the source
    drop(tx);

    // Wait for stream to close by checking subscriber completes
    assert_stream_ended(&mut sub, 500).await;

    // Assert - now closed
    assert!(shared.is_closed());
}

#[tokio::test]
async fn subscribe_after_close_returns_error() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub = shared.subscribe().unwrap();

    // Close by dropping sender and wait for it to propagate
    drop(tx);
    assert_stream_ended(&mut sub, 500).await;

    // Act
    let result = shared.subscribe();

    // Assert
    assert!(matches!(result, Err(SubjectError::Closed)));
}

#[tokio::test]
async fn each_subscriber_can_chain_independently() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();

    // Each subscriber chains differently
    // Filter: only people older than 30
    let mut filtered = shared
        .subscribe()
        .unwrap()
        .filter_ordered(|data| matches!(data, TestData::Person(p) if p.age > 30));

    // Map: add 10 years to age
    let mut mapped = shared.subscribe().unwrap().map_ordered(|data| {
        let updated = match data.into_inner() {
            TestData::Person(p) => TestData::Person(Person::new(p.name, p.age + 10)),
            other => other,
        };
        Sequenced::new(updated)
    });

    // Act
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap(); // Alice(25): filtered out, mapped to 35
    tx.unbounded_send(Sequenced::new(person_charlie())).unwrap(); // Charlie(35): passes filter, mapped to 45

    // Assert - filtered subscriber only sees Charlie (age > 30)
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut filtered, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Charlie" && p.age == 35
    ));

    // Assert - mapped subscriber sees both with +10 years
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut mapped, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Alice" && p.age == 35
    ));
    assert!(matches!(
        unwrap_value(Some(unwrap_stream(&mut mapped, 500).await)).into_inner(),
        TestData::Person(ref p) if p.name == "Charlie" && p.age == 45
    ));
}

#[tokio::test]
async fn source_operators_run_once_per_emission() {
    // Arrange - track how many times the map runs
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let (tx, rx) = test_channel::<Sequenced<TestData>>();

    // Source with tracked transformation (runs ONCE per item)
    let source = rx.map_ordered(move |data| {
        call_count_clone.fetch_add(1, Ordering::SeqCst);
        // Add 100 years to age
        let updated = match data.into_inner() {
            TestData::Person(p) => TestData::Person(Person::new(p.name, p.age + 100)),
            other => other,
        };
        Sequenced::new(updated)
    });

    let shared = source.share();

    let mut sub1 = shared.subscribe().unwrap();
    let mut sub2 = shared.subscribe().unwrap();
    let mut sub3 = shared.subscribe().unwrap();

    // Act - send one item
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap();

    // Consume from all subscribers
    let _ = sub1.next().await;
    let _ = sub2.next().await;
    let _ = sub3.next().await;

    // Assert - map_ordered ran exactly ONCE (not 3 times)
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn drop_closes_subject_and_cancels_task() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub = shared.subscribe().unwrap();

    // Drop the shared (should close subject)
    drop(shared);

    // Try to send - should work (sender not connected to shared)
    let _ = tx.unbounded_send(Sequenced::new(person_diane()));

    // Subscriber should complete (subject closed)
    assert_stream_ended(&mut sub, 500).await;
}

#[tokio::test]
async fn empty_source_stream_completes_subscribers_immediately() {
    // Arrange - create a channel and immediately drop the sender
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    drop(tx); // Source completes immediately with no items

    let source = rx;
    let shared = source.share();

    // Subscribe after source is already closed
    let mut sub = shared.subscribe().unwrap();

    // Assert - subscriber completes without receiving any items
    assert_stream_ended(&mut sub, 500).await;
}

#[tokio::test]
async fn subscriber_dropped_mid_stream_does_not_affect_others() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();
    let mut sub1 = shared.subscribe().unwrap();
    let mut sub2 = shared.subscribe().unwrap();
    let sub3 = shared.subscribe().unwrap(); // Will be dropped

    // Act - send first item
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap();

    // Consume from all three
    let _ = unwrap_stream(&mut sub1, 500).await;
    let _ = unwrap_stream(&mut sub2, 500).await;

    // Drop sub3 mid-stream (simulating subscriber being dropped)
    drop(sub3);

    // Send more items - remaining subscribers should still receive them
    tx.unbounded_send(Sequenced::new(person_bob())).unwrap();
    tx.unbounded_send(Sequenced::new(person_charlie())).unwrap();

    // Assert - sub1 and sub2 still receive items
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub1, 500).await)).into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub2, 500).await)).into_inner(),
        person_bob()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub1, 500).await)).into_inner(),
        person_charlie()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut sub2, 500).await)).into_inner(),
        person_charlie()
    );
}

#[tokio::test]
async fn high_subscriber_count_broadcasts_correctly() {
    // Arrange
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let source = rx;

    let shared = source.share();

    // Create many subscribers
    const SUBSCRIBER_COUNT: usize = 50;
    let mut subscribers: Vec<_> = (0..SUBSCRIBER_COUNT)
        .map(|_| shared.subscribe().unwrap())
        .collect();

    // Act - send items
    tx.unbounded_send(Sequenced::new(person_alice())).unwrap();
    tx.unbounded_send(Sequenced::new(person_bob())).unwrap();

    // Assert - all subscribers receive both items
    for (i, sub) in subscribers.iter_mut().enumerate() {
        let first = unwrap_value(Some(unwrap_stream(sub, 500).await)).into_inner();
        assert_eq!(
            first,
            person_alice(),
            "Subscriber {i} did not receive Alice"
        );

        let second = unwrap_value(Some(unwrap_stream(sub, 500).await)).into_inner();
        assert_eq!(second, person_bob(), "Subscriber {i} did not receive Bob");
    }

    // Verify subscriber count
    assert_eq!(shared.subscriber_count(), SUBSCRIBER_COUNT);
}
