use fluxion_stream::take_while::TakeWhileStreamExt;
use fluxion_test_utils::FluxionChannel;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::push;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, person_alice, person_bob, person_charlie,
};
use futures::StreamExt;

#[tokio::test]
async fn test_take_while_basic() {
    // Arrange: Create channels and set up stream
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Send filter value first (true)
    push(true, &filter.sender);

    // Act: Send some source values
    push(animal_cat(), &source.sender);

    // Assert: First value emitted
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    // Act: Send more source values
    push(animal_dog(), &source.sender);

    // Assert: Second value emitted
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    // Act: Send third source value
    push(person_alice(), &source.sender);

    // Assert: Third value emitted
    let item = result_stream.next().await.expect("expected third item");
    assert_eq!(item, person_alice());

    // Act: Change filter to false
    push(false, &filter.sender);

    // Act: Send more source values (should not be emitted)
    push(person_bob(), &source.sender);
    push(person_charlie(), &source.sender);

    // Assert: No more values emitted
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_false_immediately() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Send filter value first (false)
    push(false, &filter.sender);

    // Act: Send source values
    push(animal_cat(), &source.sender);
    push(animal_dog(), &source.sender);

    // Assert: No values should be emitted since filter is false from the start
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_always_true() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Send filter value (true)
    push(true, &filter.sender);

    // Act & Assert: Send source values and verify they're all emitted
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    push(person_alice(), &source.sender);
    let item = result_stream.next().await.expect("expected third item");
    assert_eq!(item, person_alice());
}

#[tokio::test]
async fn test_take_while_complex_predicate() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val: &i32| {
            *filter_val < 10
        });
    let mut result_stream = Box::pin(result_stream);

    // Act: Send initial filter value (5, which is < 10)
    push(5, &filter.sender);

    // Act & Assert: Send source values, they should be emitted
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    // Act: Change filter to a value that fails the predicate (10, which is not < 10)
    push(10, &filter.sender);

    // Act: Send more source values
    push(person_alice(), &source.sender);
    push(person_bob(), &source.sender);

    // Assert: No more values emitted after filter fails
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_interleaved_updates() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert: Interleave source and filter updates
    push(true, &filter.sender);

    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(true, &filter.sender);

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    push(true, &filter.sender);

    push(person_alice(), &source.sender);
    let item = result_stream.next().await.expect("expected third item");
    assert_eq!(item, person_alice());

    push(false, &filter.sender);

    push(person_bob(), &source.sender);
    push(person_charlie(), &source.sender);

    // Assert: No more values after filter becomes false
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_no_filter_value() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Send source values before any filter value
    push(animal_cat(), &source.sender);
    push(animal_dog(), &source.sender);

    // Assert: No emission yet since no filter value
    assert_no_element_emitted(&mut result_stream, 100).await;

    // Act: Now send filter value
    push(true, &filter.sender);

    // Act & Assert: Send more source values, they should be emitted now
    push(person_alice(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, person_alice());

    push(person_bob(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, person_bob());
}

#[tokio::test]
async fn test_take_while_empty_source() {
    // Arrange
    let source = FluxionChannel::<bool>::new();
    let filter = FluxionChannel::<bool>::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Send filter value
    push(true, &filter.sender);

    // Act: Drop source without sending anything
    drop(source.sender);

    // Assert: No values emitted
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_empty_filter() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::<bool>::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Send source values but no filter values
    push(animal_cat(), &source.sender);
    push(animal_dog(), &source.sender);

    // Assert: No filter values, so no source values should be emitted
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() {
    // Arrange: Test that once filter becomes false, stream terminates even if filter becomes true again
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Start with filter true
    push(true, &filter.sender);

    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    // Act: Change filter to false
    push(false, &filter.sender);

    push(animal_dog(), &source.sender);

    // Assert: No emission after filter becomes false
    assert_no_element_emitted(&mut result_stream, 100).await;

    // Act: Change filter back to true
    push(true, &filter.sender);

    push(person_alice(), &source.sender);

    // Assert: Stream should still be terminated (take_while stops permanently once predicate fails)
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() {
    // Arrange: Test that multiple source items emitted with same filter value
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Set filter to true
    push(true, &filter.sender);

    // Act: Push multiple source items without changing filter
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    push(person_alice(), &source.sender);
    let item = result_stream.next().await.expect("expected third item");
    assert_eq!(item, person_alice());

    push(person_bob(), &source.sender);
    let item = result_stream.next().await.expect("expected fourth item");
    assert_eq!(item, person_bob());

    // Act: Now change filter to false
    push(false, &filter.sender);

    push(person_charlie(), &source.sender);

    // Assert: No more items after filter becomes false
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() {
    // Arrange: Test that filter can update multiple times before any source items
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act: Update filter multiple times before sending source
    push(false, &filter.sender);
    push(true, &filter.sender);
    push(true, &filter.sender);

    // Assert: No emission yet since no source
    assert_no_element_emitted(&mut result_stream, 100).await;

    // Act: Now send source - should use latest filter value (true)
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());
}
