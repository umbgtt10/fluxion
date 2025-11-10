use fluxion::{combine_latest::CombinedState, take_latest_when::TakeLatestWhenExt};
use futures::StreamExt;
use std::{
    fmt::{self, Display},
    thread::sleep,
    time::Duration,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

mod infra;
mod test_data;
use crate::{
    infra::infrastructure::assert_no_element_emitted,
    test_data::{animal::Animal, person::Person},
};

const BUFFER_SIZE: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StreamValue {
    Person(Person),
    Animal(Animal),
}

impl Display for StreamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamValue::Person(p) => write!(f, "{}", p),
            StreamValue::Animal(a) => write!(f, "{}", a),
        }
    }
}

#[tokio::test]
async fn test_take_latest_when_empty_streams() {
    static FILTER: fn(&CombinedState<StreamValue>) -> bool = |_: &CombinedState<StreamValue>| true;

    // Arrange
    let (_, source_receiver) = mpsc::channel::<StreamValue>(BUFFER_SIZE);
    let source_stream = ReceiverStream::new(source_receiver);

    let (_, filter_receiver) = mpsc::channel::<StreamValue>(BUFFER_SIZE);
    let filter_stream = ReceiverStream::new(filter_receiver);

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
    let (source_sender, source_receiver) = mpsc::channel(BUFFER_SIZE);
    let source_stream = ReceiverStream::new(source_receiver);

    let (filter_sender, filter_receiver) = mpsc::channel(BUFFER_SIZE);
    let filter_stream = ReceiverStream::new(filter_receiver);

    static FILTER: fn(&CombinedState<StreamValue>) -> bool = |state| {
        let state = state.get_state().first().unwrap().clone();
        match state {
            StreamValue::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act
    source_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();

    filter_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 4)))
        .await
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
}

#[tokio::test]
async fn test_take_latest_when_filter_satisfied_emits() {
    // Arrange
    let (source_sender, source_receiver) = mpsc::channel(BUFFER_SIZE);
    let source_stream = ReceiverStream::new(source_receiver);

    let (filter_sender, filter_receiver) = mpsc::channel(BUFFER_SIZE);
    let filter_stream = ReceiverStream::new(filter_receiver);

    static FILTER: fn(&CombinedState<StreamValue>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            StreamValue::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act
    source_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();

    filter_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 6)))
        .await
        .unwrap();

    // Assert
    let emitted_item = output_stream.next().await.unwrap();
    assert_eq!(
        emitted_item,
        StreamValue::Person(Person::new("Alice".to_string(), 30)),
        "Expected the source item to be emitted when the filter is satisfied"
    );
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_satisfied() {
    // Arrange
    let (source_sender, source_receiver) = mpsc::channel(BUFFER_SIZE);
    let source_stream = ReceiverStream::new(source_receiver);

    let (filter_sender, filter_receiver) = mpsc::channel(BUFFER_SIZE);
    let filter_stream = ReceiverStream::new(filter_receiver);

    static FILTER: fn(&CombinedState<StreamValue>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            StreamValue::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act
    source_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();

    filter_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 6)))
        .await
        .unwrap();

    // Assert
    let first_item = output_stream.next().await.unwrap();
    assert_eq!(
        first_item,
        StreamValue::Person(Person::new("Alice".to_string(), 30)),
        "First emitted item did not match expected"
    );

    // Act
    source_sender
        .send(StreamValue::Person(Person::new("Bob".to_string(), 40)))
        .await
        .unwrap();

    // Assert
    let second_item = output_stream.next().await.unwrap();
    assert_eq!(
        second_item,
        StreamValue::Person(Person::new("Bob".to_string(), 40)),
        "Second emitted item did not match expected"
    );
}

#[tokio::test]
#[ignore]
async fn test_take_latest_when_multiple_emissions_filter_not_satisfied() {
    // Arrange
    let (source_sender, source_receiver) = mpsc::channel(BUFFER_SIZE);
    let source_stream = ReceiverStream::new(source_receiver);

    let (filter_sender, filter_receiver) = mpsc::channel(BUFFER_SIZE);
    let filter_stream = ReceiverStream::new(filter_receiver);

    static FILTER: fn(&CombinedState<StreamValue>) -> bool = |state| {
        let filter_value = state.get_state()[1].clone();

        match filter_value {
            StreamValue::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {:?} instead!",
                    filter_value
                );
            }
        }
    };

    let output_stream = source_stream.take_latest_when(filter_stream, FILTER);
    let mut output_stream = Box::pin(output_stream);

    // Act
    filter_sender
        .send(StreamValue::Animal(Animal::new("Ant".to_string(), 6)))
        .await
        .unwrap();

    source_sender
        .send(StreamValue::Person(Person::new("Charlie".to_string(), 25)))
        .await
        .unwrap();

    // Assert
    let first_item = output_stream.next().await.unwrap();
    assert_eq!(
        first_item,
        StreamValue::Person(Person::new("Charlie".to_string(), 25)),
        "First emitted item did not match expected"
    );

    // Act
    filter_sender
        .send(StreamValue::Animal(Animal::new("Cat".to_string(), 4)))
        .await
        .unwrap();

    source_sender
        .send(StreamValue::Person(Person::new("Dave".to_string(), 28)))
        .await
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut output_stream, 100).await;
}
