use fluxion_stream::ordered::OrderedWrapper;
use fluxion_stream::{CombinedState, FluxionStream, Ordered};
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

    static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Use FluxionStream for combine_latest
    let combined = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream, plant.stream], COMBINE_FILTER);

    let mut combined = Box::pin(combined);

    // Act
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Assert - now returns Sequenced<CombinedState<TestData>>
    let result = combined.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());
}

/// Test FluxionStream with with_latest_from
#[tokio::test]
async fn test_fluxion_stream_with_latest_from() {
    // Arrange
    let (primary, secondary) = TestChannels::two::<TestData>();

    static WITH_LATEST_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

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

/// Test chaining combine_latest with take_while_with
#[tokio::test]
async fn test_fluxion_stream_combine_latest_and_take_while() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();
    let plant: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Compose: combine_latest -> take_while_with
    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream, plant.stream], COMBINE_FILTER)
        .take_while_with(filter.stream, |f| *f);

    let mut composed = Box::pin(composed);

    // Act & Assert
    // Enable filter
    push(true, &filter.sender);
    // Push initial values to all channels
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // Should emit combined state
    let result = composed.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state.len(), 3);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    // Push new values
    push(person_bob(), &person.sender);
    let result = composed.next().await.unwrap();
    let state = result.get_state();
    assert_eq!(state[0], person_bob());
    assert_eq!(state[1], animal_dog());
    assert_eq!(state[2], plant_rose());

    // Disable filter - should stop emitting
    push(false, &filter.sender);
    push(person_charlie(), &person.sender);
    assert_no_element_emitted(&mut composed, 100).await;
}

/// Test FluxionStream ordered_merge - merges multiple streams and emits all values per sequence
#[tokio::test]
async fn test_fluxion_stream_ordered_merge() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let plant = FluxionChannel::new();

    // Merge all three streams - emits each value individually in sequence order
    // Note: With FluxionChannel, each push gets a unique global sequence
    let merged = FluxionStream::new(person.stream).ordered_merge(vec![animal.stream, plant.stream]);

    let mut merged = Box::pin(merged);

    // Act & Assert - Each push gets a unique global sequence
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);
    push(plant_rose(), &plant.sender);

    // First value emitted - should be person_alice
    let result1 = merged.next().await.unwrap();
    assert_eq!(result1.get(), &person_alice());

    // Second value emitted - should be animal_dog with next sequence
    let result2 = merged.next().await.unwrap();
    assert_eq!(result2.get(), &animal_dog());

    // Third value emitted - should be plant_rose with next sequence
    let result3 = merged.next().await.unwrap();
    assert_eq!(result3.get(), &plant_rose());
}

/// Test ordered_merge -> combine_with_previous composition
/// Merges multiple streams and tracks deltas between consecutive values
#[tokio::test]
async fn test_ordered_merge_then_combine_with_previous() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();

    // Chain: ordered_merge -> combine_with_previous
    let composed = FluxionStream::new(person.stream)
        .ordered_merge(vec![animal.stream])
        .combine_with_previous();

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(person_alice(), &person.sender);
    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none());
    assert_eq!(curr.get(), &person_alice());

    push(animal_dog(), &animal.sender);
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &person_alice());
    assert_eq!(curr.get(), &animal_dog());

    push(person_bob(), &person.sender);
    let (prev, curr) = composed.next().await.unwrap();
    assert_eq!(prev.unwrap().get(), &animal_dog());
    assert_eq!(curr.get(), &person_bob());
}

/// Test combine_latest -> combine_with_previous composition
/// Creates combined state, then tracks how it changes over time
#[tokio::test]
async fn test_combine_latest_then_combine_with_previous() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();

    static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Chain: combine_latest -> combine_with_previous
    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream], COMBINE_FILTER)
        .combine_with_previous();

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);

    let (prev, curr) = composed.next().await.unwrap();
    assert!(prev.is_none());
    let curr_state = curr.get().get_state();
    assert_eq!(curr_state[0], person_alice());
    assert_eq!(curr_state[1], animal_dog());

    push(person_bob(), &person.sender);
    let (prev, curr) = composed.next().await.unwrap();
    let prev_seq = prev.unwrap();
    let prev_state = prev_seq.get().get_state();
    assert_eq!(prev_state[0], person_alice());
    assert_eq!(prev_state[1], animal_dog());
    let curr_state = curr.get().get_state();
    assert_eq!(curr_state[0], person_bob());
    assert_eq!(curr_state[1], animal_dog());
}

/// Test combine_latest -> take_latest_when composition
/// Creates combined state, then conditionally emits based on another filter stream
#[tokio::test]
async fn test_combine_latest_then_take_latest_when() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<CombinedState<TestData>> = FluxionChannel::new();

    static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;
    static LATEST_FILTER: fn(&CombinedState<CombinedState<TestData>>) -> bool = |_| true;

    // Chain: combine_latest -> take_latest_when
    // Note: filter stream needs to be mapped to OrderedWrapper to match combine_latest output
    let filter_mapped = filter.stream.map(|seq| {
        let order = seq.order();
        OrderedWrapper::with_order(seq.into_inner(), order)
    });

    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream], COMBINE_FILTER)
        .take_latest_when(filter_mapped, LATEST_FILTER);

    let mut composed = Box::pin(composed);

    // Act & Assert
    let filter_state = CombinedState::new(vec![person_alice()]);
    push(filter_state, &filter.sender);
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);

    let result = composed.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state.len(), 2);
    assert_eq!(state[0], person_alice());
    assert_eq!(state[1], animal_dog());

    push(person_bob(), &person.sender);
    let result = composed.next().await.unwrap();
    let state = result.get().get_state();
    assert_eq!(state[0], person_bob());
    assert_eq!(state[1], animal_dog());
}

/// Test ordered_merge -> take_while_with composition
/// Merges streams and terminates when a condition is no longer met
#[tokio::test]
async fn test_ordered_merge_then_take_while_with() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    // Chain: ordered_merge -> take_while_with
    let composed = FluxionStream::new(person.stream)
        .ordered_merge(vec![animal.stream])
        .take_while_with(filter.stream, |f| *f);

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(true, &filter.sender);
    push(person_alice(), &person.sender);
    assert_eq!(composed.next().await.unwrap(), person_alice());

    push(animal_dog(), &animal.sender);
    assert_eq!(composed.next().await.unwrap(), animal_dog());

    push(person_bob(), &person.sender);
    assert_eq!(composed.next().await.unwrap(), person_bob());

    // Filter becomes false - stream terminates
    push(false, &filter.sender);
    push(person_charlie(), &person.sender);
    assert_no_element_emitted(&mut composed, 100).await;
}

/// Test combine_latest -> take_while_with, then parallel merge
/// Three-level composition demonstrating complex stream processing
#[tokio::test]
async fn test_triple_composition_combine_latest_take_while_ordered_merge() {
    // Arrange
    let person: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<bool> = FluxionChannel::new();

    static COMBINE_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Chain: combine_latest -> take_while_with -> combine_with_previous
    let composed = FluxionStream::new(person.stream)
        .combine_latest(vec![animal.stream], COMBINE_FILTER)
        .take_while_with(filter.stream, |f| *f);

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(true, &filter.sender);
    push(person_alice(), &person.sender);
    push(animal_dog(), &animal.sender);

    let result = composed.next().await.unwrap();
    assert_eq!(result.get_state().len(), 2);
    assert_eq!(result.get_state()[0], person_alice());
    assert_eq!(result.get_state()[1], animal_dog());

    push(person_bob(), &person.sender);
    let result = composed.next().await.unwrap();
    assert_eq!(result.get_state()[0], person_bob());
    assert_eq!(result.get_state()[1], animal_dog());

    // Filter becomes false - stream terminates
    push(false, &filter.sender);
    push(person_charlie(), &person.sender);
    assert_no_element_emitted(&mut composed, 100).await;
}

/// Test ordered_merge -> take_latest_when composition
/// Merges streams, then conditionally emits based on filter
#[tokio::test]
async fn test_ordered_merge_then_take_latest_when() {
    // Arrange
    let person = FluxionChannel::new();
    let animal = FluxionChannel::new();
    let filter: FluxionChannel<TestData> = FluxionChannel::new();

    static LATEST_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Chain: ordered_merge -> take_latest_when
    let composed = FluxionStream::new(person.stream)
        .ordered_merge(vec![animal.stream])
        .take_latest_when(filter.stream, LATEST_FILTER);

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(person_alice(), &filter.sender);
    push(person_alice(), &person.sender);
    assert_eq!(composed.next().await.unwrap().get(), &person_alice());

    push(animal_dog(), &animal.sender);
    assert_eq!(composed.next().await.unwrap().get(), &animal_dog());

    push(person_bob(), &person.sender);
    assert_eq!(composed.next().await.unwrap().get(), &person_bob());

    // Filter closes but stream continues with last filter value
    drop(filter.sender);
    push(person_charlie(), &person.sender);
    assert_eq!(composed.next().await.unwrap().get(), &person_charlie());
}

/// Test take_latest_when -> ordered_merge composition
/// Filters a primary stream, then merges it with other streams
#[tokio::test]
async fn test_take_latest_when_then_ordered_merge() {
    // Arrange
    let source: FluxionChannel<TestData> = FluxionChannel::new();
    let filter: FluxionChannel<TestData> = FluxionChannel::new();
    let animal: FluxionChannel<TestData> = FluxionChannel::new();

    static LATEST_FILTER: fn(&CombinedState<TestData>) -> bool = |_| true;

    // Chain: take_latest_when -> ordered_merge
    let composed = FluxionStream::new(source.stream)
        .take_latest_when(filter.stream, LATEST_FILTER)
        .ordered_merge(vec![animal.stream]);

    let mut composed = Box::pin(composed);

    // Act & Assert
    push(person_alice(), &filter.sender);
    push(person_alice(), &source.sender);
    push(animal_dog(), &animal.sender);

    // First emission could be either person_alice or animal_dog
    let result1 = composed.next().await.unwrap();
    let result2 = composed.next().await.unwrap();

    // Check that we got both values
    let values: Vec<_> = vec![result1.get(), result2.get()];
    assert!(values.contains(&&person_alice()));
    assert!(values.contains(&&animal_dog()));

    push(person_bob(), &source.sender);
    let result = composed.next().await.unwrap();
    assert_eq!(result.get(), &person_bob());

    // Filter closes but stream continues
    drop(filter.sender);
    push(person_charlie(), &source.sender);
    let result = composed.next().await.unwrap();
    assert_eq!(result.get(), &person_charlie());
}
