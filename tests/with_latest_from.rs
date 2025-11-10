use fluxion::combine_latest::CombinedState;
use fluxion::with_latest_from::WithLatestFromExt;
use tokio::sync::mpsc;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

mod infra;
mod test_data;
use crate::infra::infrastructure::assert_no_element_emitted;
use crate::test_data::animal::Animal;
use crate::test_data::person::Person;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StreamValue {
    Animal(Animal),
    Person(Person),
}

static FILTER: fn(&CombinedState<StreamValue>) -> bool = |_: &CombinedState<StreamValue>| true;

#[tokio::test]
async fn test_with_latest_from_complete() {
    // Arrange
    let (animal_sender, animal_receiver) = mpsc::channel(10);
    let animal_stream = ReceiverStream::new(animal_receiver);
    let (person_sender, person_receiver) = mpsc::channel(10);
    let person_stream = ReceiverStream::new(person_receiver);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Cat".to_string(), 4)))
        .await
        .unwrap();
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Cat".to_string(), 4)),
            StreamValue::Person(Person::new("Alice".to_string(), 30))
        )
    );

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 4)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Dog".to_string(), 4)),
            StreamValue::Person(Person::new("Alice".to_string(), 30))
        )
    );

    // Act
    person_sender
        .send(StreamValue::Person(Person::new("Bob".to_string(), 25)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Dog".to_string(), 4)),
            StreamValue::Person(Person::new("Bob".to_string(), 25))
        )
    );
}

#[tokio::test]
async fn test_with_latest_from_second_stream_does_not_emit_no_output() {
    // Arrange
    let (animal_sender, animal_receiver) = mpsc::channel(10);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (_, person_receiver) = mpsc::channel(10);
    let person_stream = ReceiverStream::new(person_receiver);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Cat".to_string(), 4)))
        .await
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() {
    // Arrange
    let (animal_sender, animal_receiver) = mpsc::channel(10);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (person_sender, person_receiver) = mpsc::channel(10);
    let person_stream = ReceiverStream::new(person_receiver);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();
    drop(person_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Cat".to_string(), 4)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Cat".to_string(), 4)),
            StreamValue::Person(Person::new("Alice".to_string(), 30))
        )
    );

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Dog".to_string(), 4)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Dog".to_string(), 4)),
            StreamValue::Person(Person::new("Alice".to_string(), 30))
        )
    );
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() {
    // Arrange
    let (animal_sender, animal_receiver) = mpsc::channel(10);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (person_sender, person_receiver) = mpsc::channel(10);
    let person_stream = ReceiverStream::new(person_receiver);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    animal_sender
        .send(StreamValue::Animal(Animal::new("Cat".to_string(), 4)))
        .await
        .unwrap();
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Cat".to_string(), 4)),
            StreamValue::Person(Person::new("Alice".to_string(), 30))
        )
    );

    // Act
    drop(animal_sender);
    person_sender
        .send(StreamValue::Person(Person::new("Bob".to_string(), 25)))
        .await
        .unwrap();

    // Assert
    assert_eq!(
        combined_stream.next().await.unwrap(),
        (
            StreamValue::Animal(Animal::new("Cat".to_string(), 4)),
            StreamValue::Person(Person::new("Bob".to_string(), 25))
        )
    );
}

#[tokio::test]
async fn test_large_number_of_emissions() {
    // Arrange
    let (animal_sender, animal_receiver) = mpsc::channel(1000);
    let animal_stream = ReceiverStream::new(animal_receiver);

    let (person_sender, person_receiver) = mpsc::channel(1000);
    let person_stream = ReceiverStream::new(person_receiver);

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    person_sender
        .send(StreamValue::Person(Person::new("Alice".to_string(), 30)))
        .await
        .unwrap();

    for i in 0..1000 {
        animal_sender
            .send(StreamValue::Animal(Animal::new(format!("Animal{}", i), 4)))
            .await
            .unwrap();
    }

    // Assert
    for i in 0..1000 {
        assert_eq!(
            combined_stream.next().await.unwrap(),
            (
                StreamValue::Animal(Animal::new(format!("Animal{}", i), 4)),
                StreamValue::Person(Person::new("Alice".to_string(), 30))
            )
        );
    }
}
