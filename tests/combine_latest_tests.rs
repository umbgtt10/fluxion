use futures::StreamExt;
use std::fmt::{self, Display};
use stream_processing::combine_latest::{CombineLatestExt, CombinedState};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

mod infra;
mod test_data;
use crate::{
    infra::infrastructure::assert_no_element_emitted,
    test_data::{animal::Animal, person::Person, plant::Plant},
};

const BUFFER_SIZE: usize = 10;
static FILTER: fn(&CombinedState<StreamValue>) -> bool = |_: &CombinedState<StreamValue>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() {
    // Arrange
    let (_, person_receiver) = mpsc::channel::<StreamValue>(BUFFER_SIZE);
    let person_stream = ReceiverStream::new(person_receiver);

    let (_, animal_receiver) = mpsc::channel::<StreamValue>(BUFFER_SIZE);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (_, plant_receiver) = mpsc::channel::<StreamValue>(BUFFER_SIZE);
    let plant_stream = ReceiverStream::new(plant_receiver);

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Assert
    let next_item = combined_stream.next().await;
    assert!(
        next_item.is_none(),
        "Expected no items from an empty combined stream"
    );
}

#[tokio::test]
async fn test_combine_latest_not_all_streams_have_published_does_not_emit() {
    // Arrange
    let (person_sender, person_receiver) = mpsc::channel(BUFFER_SIZE);
    let person_stream = ReceiverStream::new(person_receiver);

    let (animal_sender, animal_receiver) = mpsc::channel(BUFFER_SIZE);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (_, plant_receiver) = mpsc::channel(BUFFER_SIZE);
    let plant_stream = ReceiverStream::new(plant_receiver);

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 25)))
        .await
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 4)))
        .await
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_different_order_emits_updates() {
    // Arrange
    let (person_sender, person_receiver) = mpsc::channel(BUFFER_SIZE);
    let person_stream = ReceiverStream::new(person_receiver);

    let (animal_sender, animal_receiver) = mpsc::channel(BUFFER_SIZE);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (plant_sender, plant_receiver) = mpsc::channel(BUFFER_SIZE);
    let plant_stream = ReceiverStream::new(plant_receiver);

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    plant_sender
        .send(StreamValue::Plant(Plant::new("Rose".to_string(), 15)))
        .await
        .unwrap();
    animal_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 4)))
        .await
        .unwrap();
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 25)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person(Person::new("Alice".to_string(), 25)),
            StreamValue::Animal(Animal::new("Dog".to_string(), 4)),
            StreamValue::Plant(Plant::new("Rose".to_string(), 15))
        ]
    );
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() {
    // Arrange
    let (person_sender, person_receiver) = mpsc::channel(BUFFER_SIZE);
    let person_stream = ReceiverStream::new(person_receiver);

    let (animal_sender, animal_receiver) = mpsc::channel(BUFFER_SIZE);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (plant_sender, plant_receiver) = mpsc::channel(BUFFER_SIZE);
    let plant_stream = ReceiverStream::new(plant_receiver);

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 25)))
        .await
        .unwrap();
    animal_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 4)))
        .await
        .unwrap();
    plant_sender
        .send(StreamValue::Plant(Plant::new("Rose".to_string(), 15)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person(Person::new("Alice".to_string(), 25)),
            StreamValue::Animal(Animal::new("Dog".to_string(), 4)),
            StreamValue::Plant(Plant::new("Rose".to_string(), 15))
        ]
    );

    // Act
    person_sender
        .send(StreamValue::Person(Person::new("Bob".to_string(), 30)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person(Person::new("Bob".to_string(), 30)),
            StreamValue::Animal(Animal::new("Dog".to_string(), 4)),
            StreamValue::Plant(Plant::new("Rose".to_string(), 15))
        ]
    );

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Spider".to_string(), 8)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person(Person::new("Bob".to_string(), 30)),
            StreamValue::Animal(Animal::new("Spider".to_string(), 8)),
            StreamValue::Plant(Plant::new("Rose".to_string(), 15))
        ]
    );

    // Act
    plant_sender
        .send(StreamValue::Plant(Plant::new("Sunflower".to_string(), 180)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person(Person::new("Bob".to_string(), 30)),
            StreamValue::Animal(Animal::new("Spider".to_string(), 8)),
            StreamValue::Plant(Plant::new("Sunflower".to_string(), 180))
        ]
    );
}

#[tokio::test]
async fn test_combine_latest_with_identical_streams_emits_updates() {
    // Arrange
    #[derive(Debug, Clone, PartialEq)]
    enum StreamValue {
        Person1(Person),
        Person2(Person),
    }

    static FILTER: fn(&CombinedState<StreamValue>) -> bool = |_: &CombinedState<StreamValue>| true;

    let (stream1_sender, stream1_receiver) = mpsc::channel(BUFFER_SIZE);
    let stream1 = ReceiverStream::new(stream1_receiver);

    let (stream2_sender, stream2_receiver) = mpsc::channel(BUFFER_SIZE);
    let stream2 = ReceiverStream::new(stream2_receiver);

    let combined_stream = stream1.combine_latest(vec![stream2], FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    stream1_sender
        .send(StreamValue::Person1(Person::new("Alice".to_string(), 25)))
        .await
        .unwrap();
    stream2_sender
        .send(StreamValue::Person2(Person::new("Bob".to_string(), 30)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person1(Person::new("Alice".to_string(), 25)),
            StreamValue::Person2(Person::new("Bob".to_string(), 30))
        ]
    );

    // Act
    stream1_sender
        .send(StreamValue::Person1(Person::new("Charlie".to_string(), 35)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person1(Person::new("Charlie".to_string(), 35)),
            StreamValue::Person2(Person::new("Bob".to_string(), 30))
        ]
    );

    // Act
    stream2_sender
        .send(StreamValue::Person2(Person::new("Diane".to_string(), 40)))
        .await
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    assert_eq!(
        state.get_state(),
        &vec![
            StreamValue::Person1(Person::new("Charlie".to_string(), 35)),
            StreamValue::Person2(Person::new("Diane".to_string(), 40))
        ]
    );
}

#[derive(Debug, Clone, PartialEq)]
enum StreamValue {
    Person(Person),
    Animal(Animal),
    Plant(Plant),
}

impl Display for StreamValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamValue::Person(p) => write!(f, "{}", p),
            StreamValue::Animal(a) => write!(f, "{}", a),
            StreamValue::Plant(p) => write!(f, "{}", p),
        }
    }
}
