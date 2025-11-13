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
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    push(true, &filter.sender);
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    push(person_alice(), &source.sender);
    let item = result_stream.next().await.expect("expected third item");
    assert_eq!(item, person_alice());

    push(false, &filter.sender);
    push(person_bob(), &source.sender);
    push(person_charlie(), &source.sender);
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

    // Act & Assert
    push(false, &filter.sender);
    push(animal_cat(), &source.sender);
    push(animal_dog(), &source.sender);
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

    // Act & Assert
    push(true, &filter.sender);
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

    // Act & Assert
    push(5, &filter.sender);
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());

    push(10, &filter.sender);
    push(person_alice(), &source.sender);
    push(person_bob(), &source.sender);
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

    // Act & Assert
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

    // Act & Assert
    push(animal_cat(), &source.sender);
    push(animal_dog(), &source.sender);
    assert_no_element_emitted(&mut result_stream, 100).await;

    push(true, &filter.sender);
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

    // Act
    push(true, &filter.sender);
    drop(source.sender);

    // Assert
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

    // Act
    push(animal_cat(), &source.sender);
    push(animal_dog(), &source.sender);

    // Assert
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_changes_back_to_true() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    push(true, &filter.sender);
    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(false, &filter.sender);
    push(animal_dog(), &source.sender);
    assert_no_element_emitted(&mut result_stream, 100).await;

    push(true, &filter.sender);
    push(person_alice(), &source.sender);
    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_multiple_source_items_same_filter() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    push(true, &filter.sender);
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

    push(false, &filter.sender);
    push(person_charlie(), &source.sender);

    assert_no_element_emitted(&mut result_stream, 100).await;
}

#[tokio::test]
async fn test_take_while_filter_updates_without_source() {
    // Arrange
    let source = FluxionChannel::new();
    let filter = FluxionChannel::new();

    let result_stream =
        TakeWhileStreamExt::take_while(source.stream, filter.stream, |filter_val| *filter_val);
    let mut result_stream = Box::pin(result_stream);

    // Act & Assert
    push(false, &filter.sender);
    push(true, &filter.sender);
    push(true, &filter.sender);

    assert_no_element_emitted(&mut result_stream, 100).await;

    push(animal_cat(), &source.sender);
    let item = result_stream.next().await.expect("expected first item");
    assert_eq!(item, animal_cat());

    push(animal_dog(), &source.sender);
    let item = result_stream.next().await.expect("expected second item");
    assert_eq!(item, animal_dog());
}
