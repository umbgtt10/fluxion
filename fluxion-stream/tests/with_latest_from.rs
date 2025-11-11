use fluxion_stream::combine_latest::CombinedState;
use fluxion_stream::sequenced::Sequenced;
use fluxion_stream::sequenced_channel::unbounded_channel;
use fluxion_stream::with_latest_from::WithLatestFromExt;
use fluxion_test_utils::helpers::assert_no_element_emitted;
use fluxion_test_utils::push;
use fluxion_test_utils::test_value::{TestValue, person_alice, animal, person_bob, animal_cat, animal_dog};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

static FILTER: fn(&CombinedState<Sequenced<TestValue>>) -> bool =
    |_: &CombinedState<Sequenced<TestValue>>| true;

#[tokio::test]
async fn test_with_latest_from_complete() {
    // Arrange
    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    push(animal_cat(), &animal_sender);
    push(person_alice(), &person_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_alice(), animal_cat()));

    // Act
    push(animal_dog(), &animal_sender);

    // Assert
    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_alice(), animal_dog()));

    // Act
    push(person_bob(), &person_sender);

    // Assert
    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_bob(), animal_dog()));
}

#[tokio::test]
async fn test_with_latest_from_second_stream_does_not_emit_no_output() {
    // Arrange
    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    push(animal_cat(), &animal_sender);

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_with_latest_from_secondary_completes_early() {
    // Arrange
    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    push(person_alice(), &person_sender);
    drop(person_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    push(animal_cat(), &animal_sender);

    // Assert
    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_alice(), animal_cat()));

    // Act
    push(animal_dog(), &animal_sender);

    // Assert
    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_alice(), animal_dog()));
}

#[tokio::test]
async fn test_with_latest_from_primary_completes_early() {
    // Arrange
    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    push(animal_cat(), &animal_sender);
    push(person_alice(), &person_sender);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_alice(), animal_cat()));

    // Act
    drop(animal_sender);
    push(person_bob(), &person_sender);

    // Assert
    let (p, a) = combined_stream.next().await.unwrap();
    assert_eq!((p.value, a.value), (person_bob(), animal_cat()));
}

#[tokio::test]
async fn test_large_number_of_emissions() {
    // Arrange
    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let combined_stream = animal_stream.with_latest_from(person_stream, FILTER);

    // Act
    push(person_alice(), &person_sender);

    for i in 0..1000 {
        animal_sender
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
