use fluxion_stream::{CombinedState, FluxionStream};
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::test_data::{
    TestData, animal_dog, person_alice, person_bob, person_charlie, person_dave, plant_rose,
};
use fluxion_test_utils::{FluxionChannel, TestChannels, push};
use futures::StreamExt;

static FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

/// Test demonstrating composition of multiple FluxionStream extensions
/// This test chains: take_latest_when -> combine_with_previous -> with_latest_from
#[tokio::test]
async fn test_fluxion_stream_composition() {
    // Arrange
    let (source, filter, _secondary) = TestChannels::three::<TestData>();

    // Compose multiple operations using FluxionStream
    let composed = FluxionStream::new(source.stream)
        .take_latest_when(filter.stream, FILTER) // Filter when to emit
        .combine_with_previous(); // Track previous values

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(person_alice(), &filter.sender);
    push(person_alice(), &source.sender);

    // First emission from take_latest_when -> combine_with_previous
    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none(), "First emission should have no previous");
    assert_eq!(curr.get(), &person_alice());

    // Second emission
    push(person_bob(), &source.sender);
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &person_bob());

    // Third emission
    push(person_charlie(), &source.sender);
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_bob());
    assert_eq!(curr.get(), &person_charlie());

    // Filter closes but stream continues with last filter value
    drop(filter.sender);
    push(person_dave(), &source.sender);
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_charlie());
    assert_eq!(curr.get(), &person_dave());
}

/// Test FluxionStream with combine_latest composition
#[tokio::test]
async fn test_fluxion_stream_combine_latest_composition() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    static COMBINE_FILTER: fn(&CombinedState<fluxion_stream::Sequenced<TestData>>) -> bool =
        |_| true;

    // Use FluxionStream for combine_latest
    let combined = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream, plant.stream], COMBINE_FILTER);

    let mut combined = Box::pin(combined);

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Assert
    let result = combined.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0].get(), &person_alice());
}

/// Test FluxionStream with with_latest_from
#[tokio::test]
async fn test_fluxion_stream_with_latest_from() {
    // Arrange
    let (primary, secondary) = TestChannels::two::<TestData>();

    static WITH_LATEST_FILTER: fn(&CombinedState<fluxion_stream::Sequenced<TestData>>) -> bool =
        |_| true;

    // Use FluxionStream for with_latest_from
    let combined =
        FluxionStream::new(primary.stream).with_latest_from(secondary.stream, WITH_LATEST_FILTER);

    let mut combined = Box::pin(combined);

    // Act
    push(person_alice(), &secondary.sender); // Secondary first
    push(person_bob(), &primary.sender); // Primary triggers emission

    // Assert - with_latest_from returns (secondary, primary)
    let (sec, prim) = combined.next().await.unwrap();
    assert_eq!(prim.get(), &person_bob());
    assert_eq!(sec.get(), &person_alice());

    // Second emission
    push(person_charlie(), &primary.sender);
    let (sec, prim) = combined.next().await.unwrap();
    assert_eq!(prim.get(), &person_charlie());
    assert_eq!(sec.get(), &person_alice()); // Still has alice from secondary
}

/// Test FluxionStream basic wrapper functionality
#[tokio::test]
async fn test_fluxion_stream_combine_with_previous() {
    // Arrange
    let channel: FluxionChannel<TestData> = FluxionChannel::new();

    let stream = FluxionStream::new(channel.stream).combine_with_previous();

    let mut stream = Box::pin(stream);

    // Act & Assert
    push(person_alice(), &channel.sender);
    let (prev, curr) = stream.next().await.unwrap();
    assert!(prev.is_none());
    assert_eq!(curr.get(), &person_alice());

    push(person_bob(), &channel.sender);
    let (prev, curr) = stream.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &person_bob());
}

/// Test FluxionStream with take_while_with
#[tokio::test]
async fn test_fluxion_stream_take_while_with() {
    // Arrange
    let source: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    let composed = FluxionStream::new(source.stream).take_while_with(filter.stream, |f| *f);

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(true, &filter.sender);
    push(person_alice(), &source.sender);
    assert_eq!(composed.next().await.unwrap(), person_alice());

    push(person_bob(), &source.sender);
    assert_eq!(composed.next().await.unwrap(), person_bob());

    // Filter becomes false
    push(false, &filter.sender);
    push(person_charlie(), &source.sender);

    // Stream should not emit when filter is false
    assert_no_element_emitted(&mut composed, 100).await;
}

/// Test FluxionStream composition chaining take_latest_when with take_while_with
/// This demonstrates filtering emissions with two different criteria
#[tokio::test]
async fn test_fluxion_stream_take_latest_when_take_while() {
    // Arrange
    let source: FluxionChannel<TestData> = FluxionChannel::new();
    let latest_filter = FluxionChannel::new();
    let while_filter: FluxionChannel<bool> = FluxionChannel::new();

    static LATEST_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Compose: take_latest_when -> take_while_with
    // First operator filters when to emit, second stops emitting when condition is false
    let composed = FluxionStream::new(source.stream)
        .take_latest_when(latest_filter.stream, LATEST_FILTER)
        .take_while_with(while_filter.stream, |f| *f);

    let mut composed = Box::pin(composed);

    // Act & Assert - Set filter values FIRST before pushing to source
    push(true, &while_filter.sender);
    push(person_alice(), &latest_filter.sender);

    // Now push to source - take_latest_when will emit Sequenced<TestData>
    push(person_alice(), &source.sender);

    // take_while_with unwraps Sequenced and returns TestData
    let result = composed.next().await.unwrap();
    assert_eq!(result, person_alice());

    // Second emission with while_filter still true
    push(person_bob(), &source.sender);
    let result = composed.next().await.unwrap();
    assert_eq!(result, person_bob());

    // while_filter becomes false - stream terminates (take_while_with terminates on false)
    push(false, &while_filter.sender);
    push(person_charlie(), &source.sender);
    assert_no_element_emitted(&mut composed, 100).await;
}
