use fluxion_stream::{
    combine_latest::CombinedState, timestamped_channel::unbounded_channel,
    take_latest_when::TakeLatestWhenExt,
};
use fluxion_test_utils::{
    helpers::assert_no_element_emitted,
    push,
    test_data::{
        TestData, animal_ant, animal_cat, animal_dog, person_alice, person_bob, person_charlie,
        person_dave,
    },
};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_take_latest_when_empty_streams() {
    static FILTER: fn(&CombinedState<TestData>) -> bool = |_: &CombinedState<TestData>| true;

    // Arrange
    let (_, source_receiver) = unbounded_channel();
    let source_stream = UnboundedReceiverStream::new(source_receiver.into_inner());

    let (_, filter_receiver) = unbounded_channel();
    let filter_stream = UnboundedReceiverStream::new(filter_receiver.into_inner());

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act & Assert
    let next_item = output_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty stream with `take_latest_when`"
    );
}

#[tokio::test]
async fn test_take_latest_when_filter_not_satisfied_does_not_emit() {
    // Arrange
    let (source_sender, source_receiver) = unbounded_channel();
    let source_stream = UnboundedReceiverStream::new(source_receiver.into_inner());

    let (filter_sender, filter_receiver) = unbounded_channel();
    let filter_stream = UnboundedReceiverStream::new(filter_receiver.into_inner());

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let state = state.get_state().first().unwrap().clone();
        match state {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act
    push(person_alice(), &source_sender);
    push(animal_dog(), &filter_sender);

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
}

#[tokio::test]
async fn test_take_latest_when_filter_satisfied_emits() {
    // Arrange
    let (source_sender, source_receiver) = unbounded_channel();
    let source_stream = UnboundedReceiverStream::new(source_receiver.into_inner());

    let (filter_sender, filter_receiver) = unbounded_channel();
    let filter_stream = UnboundedReceiverStream::new(filter_receiver.into_inner());

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);

    // Act
    push(person_alice(), &source_sender);
    push(animal_ant(), &filter_sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);
    let emitted_item = output_stream.next().await.unwrap();
    assert_eq!(
        emitted_item,
        person_alice(),
        "Expected the source item to be emitted when the filter is satisfied"
    );
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_satisfied() {
    // Arrange
    let (source_sender, source_receiver) = unbounded_channel();
    let source_stream = UnboundedReceiverStream::new(source_receiver.into_inner());

    let (filter_sender, filter_receiver) = unbounded_channel();
    let filter_stream = UnboundedReceiverStream::new(filter_receiver.into_inner());

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);

    // Act
    push(person_alice(), &source_sender);
    push(animal_ant(), &filter_sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);

    let first_item = output_stream.next().await.unwrap();
    assert_eq!(
        first_item,
        person_alice(),
        "First emitted item did not match expected"
    );

    // Act
    push(person_bob(), &source_sender);

    // Assert
    let second_item = output_stream.next().await.unwrap();
    assert_eq!(
        second_item,
        person_bob(),
        "Second emitted item did not match expected"
    );
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_not_satisfied() {
    // Arrange
    let (source_sender, source_receiver) = unbounded_channel();
    let source_stream = UnboundedReceiverStream::new(source_receiver.into_inner());

    let (filter_sender, filter_receiver) = unbounded_channel();
    let filter_stream = UnboundedReceiverStream::new(filter_receiver.into_inner());

    static FILTER: fn(&CombinedState<TestData>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);

    // Act
    push(animal_ant(), &filter_sender);
    push(person_charlie(), &source_sender);

    // Assert
    let mut output_stream = Box::pin(output_stream);

    let first_item = output_stream.next().await.unwrap();
    assert_eq!(
        first_item,
        person_charlie(),
        "First emitted item did not match expected"
    );

    // Act
    push(animal_cat(), &filter_sender);
    push(person_dave(), &source_sender);

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
}
