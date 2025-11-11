use fluxion_stream::combine_latest::CombinedState;
use fluxion_stream::timestamped::Timestamped;
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_test_utils::push;
use fluxion_test_utils::test_data::{
    TestData, animal, animal_cat, animal_dog, person_alice, person_bob,
};
use fluxion_test_utils::{
    TestChannel, TestChannels,
    helpers::{assert_no_element_emitted, expect_next_pair},
};
use futures::StreamExt;

static FILTER: fn(&CombinedState<Timestamped<TestData>>) -> bool =
    |_: &CombinedState<Timestamped<TestData>>| true;

#[tokio::test]
async fn test_with_latest_from_complete() {
    // Arrange
    let animal = TestChannel::new();
    let person = TestChannel::new();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);

    // Act
    push(animal_cat(), &animal.sender);
    push(person_alice(), &person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_pair(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    push(animal_dog(), &animal.sender);

    // Assert
    expect_next_pair(&mut combined_stream, person_alice(), animal_dog()).await;

    // Act
    push(person_bob(), &person.sender);

    // Assert
    expect_next_pair(&mut combined_stream, person_bob(), animal_dog()).await;
}

#[tokio::test]
async fn test_with_latest_from_second_stream_does_not_emit_no_output() {
    // Arrange
    let animal = TestChannel::new();
    let person = TestChannel::new();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(animal_cat(), &animal.sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_with_latest_from_primary_waits_for_secondary() {
    // Arrange: Secondary must publish first for any emission
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Primary publishes before secondary
    push(animal_cat(), &primary.sender);

    // Assert: No emission yet
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Secondary publishes
    push(person_alice(), &secondary.sender);

    // Assert: Now emission occurs pairing latest secondary with primary
    expect_next_pair(&mut combined_stream, person_alice(), animal_cat()).await;
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() {
    // Arrange
    let (animal, person) = TestChannels::two();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);

    // Act
    push(person_alice(), &person.sender);
    drop(person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    push(animal_cat(), &animal.sender);

    // Assert
    expect_next_pair(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    push(animal_dog(), &animal.sender);

    // Assert
    expect_next_pair(&mut combined_stream, person_alice(), animal_dog()).await;
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() {
    // Arrange
    let (animal, person) = TestChannels::two();

    let combined_stream = animal.stream.with_latest_from(person.stream, FILTER);

    // Act
    push(animal_cat(), &animal.sender);
    push(person_alice(), &person.sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    expect_next_pair(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act
    drop(animal.sender);
    push(person_bob(), &person.sender);

    // Assert
    expect_next_pair(&mut combined_stream, person_bob(), animal_cat()).await;
}

#[tokio::test]
async fn test_large_number_of_emissions() {
    // Arrange
    let (animal_channel, person) = TestChannels::two();

    let combined_stream = animal_channel
        .stream
        .with_latest_from(person.stream, FILTER);

    // Act
    push(person_alice(), &person.sender);

    for i in 0..1000 {
        animal_channel
            .sender
            .send(animal(format!("Animal{}", i), 4))
            .unwrap();
    }

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    for i in 0..1000 {
        let (p, a) = combined_stream.next().await.unwrap();
        assert_eq!(
            (p.value, a.value),
            (person_alice(), animal(format!("Animal{}", i), 4))
        );
    }
}

#[tokio::test]
async fn test_with_latest_from_rapid_primary_only_updates() {
    // Arrange: Test that rapid primary emissions pair with the latest secondary value
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Secondary publishes once
    push(person_alice(), &secondary.sender);

    // Act: Primary publishes rapidly multiple times (secondary stays Alice)
    push(animal_dog(), &primary.sender);
    push(animal_cat(), &primary.sender);

    // Assert: Each primary emission pairs with the same latest secondary value (Alice)
    // Note: Order is (secondary, primary) = (person, animal)
    expect_next_pair(&mut combined_stream, person_alice(), animal_dog()).await;
    expect_next_pair(&mut combined_stream, person_alice(), animal_cat()).await;

    // Act: Another rapid sequence of primary emissions (secondary still Alice)
    for i in 0..10 {
        primary
            .sender
            .send(animal(format!("Animal{}", i), 4))
            .unwrap();
    }

    // Assert: All primary emissions pair with Alice
    for i in 0..10 {
        let (p, a) = combined_stream.next().await.unwrap();
        assert_eq!(
            (p.value, a.value),
            (person_alice(), animal(format!("Animal{}", i), 4))
        );
    }
}

#[tokio::test]
async fn test_with_latest_from_both_streams_close_before_any_emission() {
    // Arrange: Both streams close without the secondary ever publishing
    let (primary, secondary) = TestChannels::two();

    let combined_stream = primary.stream.with_latest_from(secondary.stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act: Primary publishes but secondary never does
    push(animal_dog(), &primary.sender);

    // Assert: No emission yet (waiting for secondary)
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act: Close both streams by dropping senders
    drop(primary.sender);
    drop(secondary.sender);

    // Assert: Stream completes with no emissions
    let next_item = combined_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected stream to close with no emissions when both streams close before secondary publishes"
    );
}
