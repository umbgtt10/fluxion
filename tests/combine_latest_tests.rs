use fluxion::{
    combine_latest::{CombineLatestExt, CombinedState},
    sequenced::Sequenced,
    sequenced_channel::{UnboundedSender, unbounded_channel},
};
use futures::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

mod infra;
mod test_data;
use crate::{
    infra::infrastructure::assert_no_element_emitted,
    test_data::{animal::Animal, person::Person, plant::Plant, simple_struct::SimpleStruct},
};

static FILTER2: fn(&CombinedState<Sequenced<SimpleStruct>>) -> bool =
    |_: &CombinedState<Sequenced<SimpleStruct>>| true;

#[tokio::test]
async fn test_combine_latest_empty_streams() {
    // Arrange
    let (_, person_receiver) = unbounded_channel::<SimpleStruct>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (_, animal_receiver) = unbounded_channel::<SimpleStruct>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel::<SimpleStruct>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);
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
    let (person_sender, person_receiver) = unbounded_channel();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (_, plant_receiver) = unbounded_channel();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    person_sender
        .send(SimpleStruct::Person(Person::new("Alice".to_string(), 25)))
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;

    // Act
    animal_sender
        .send(SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)))
        .unwrap();

    // Assert
    assert_no_element_emitted(&mut combined_stream, 100).await;
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_different_order_emits_updates() {
    combine_latest_template_test(Order::Plant, Order::Animal, Order::Person).await;
    combine_latest_template_test(Order::Plant, Order::Person, Order::Animal).await;
    combine_latest_template_test(Order::Animal, Order::Plant, Order::Person).await;
    combine_latest_template_test(Order::Animal, Order::Person, Order::Plant).await;
    combine_latest_template_test(Order::Person, Order::Animal, Order::Plant).await;
    combine_latest_template_test(Order::Person, Order::Plant, Order::Animal).await;
}

/// Test template for `combine_latest` that verifies temporal ordering based on send sequence.
/// Sets up channels for Person/Animal/Plant, combines streams, sends in specified order, and asserts
/// that results are in temporal order (matching the send order) due to Sequenced wrapper and select_all_ordered.
/// Called with different permutations to validate temporal ordering is maintained across different send sequences.
async fn combine_latest_template_test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);

    // Act
    send(&order1, &senders);
    send(&order2, &senders);
    send(&order3, &senders);

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();

    let order_to_value = |order: &Order| match order {
        Order::Person => SimpleStruct::Person(Person::new("Alice".to_string(), 25)),
        Order::Animal => SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)),
        Order::Plant => SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)),
    };

    let expected = vec![
        order_to_value(&order1),
        order_to_value(&order2),
        order_to_value(&order3),
    ];
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_all_streams_have_published_emits_updates() {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel::<SimpleStruct>();
    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());

    let (animal_sender, animal_receiver) = unbounded_channel::<SimpleStruct>();
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());

    let (plant_sender, plant_receiver) = unbounded_channel::<SimpleStruct>();
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let combined_stream = person_stream.combine_latest(vec![animal_stream, plant_stream], FILTER2);

    // Act
    person_sender
        .send(SimpleStruct::Person(Person::new("Alice".to_string(), 25)))
        .unwrap();
    animal_sender
        .send(SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)))
        .unwrap();
    plant_sender
        .send(SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)))
        .unwrap();

    // Assert
    let mut combined_stream = Box::pin(combined_stream);

    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Person(Person::new("Alice".to_string(), 25)),
        SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)),
        SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)),
    ];
    assert_eq!(actual, expected);

    // Act
    person_sender
        .send(SimpleStruct::Person(Person::new("Bob".to_string(), 30)))
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)),
        SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)),
        SimpleStruct::Person(Person::new("Bob".to_string(), 30)),
    ];
    assert_eq!(actual, expected);

    // Act
    animal_sender
        .send(SimpleStruct::Animal(Animal::new("Spider".to_string(), 8)))
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)),
        SimpleStruct::Person(Person::new("Bob".to_string(), 30)),
        SimpleStruct::Animal(Animal::new("Spider".to_string(), 8)),
    ];
    assert_eq!(actual, expected);

    // Act
    plant_sender
        .send(SimpleStruct::Plant(Plant::new(
            "Sunflower".to_string(),
            180,
        )))
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Person(Person::new("Bob".to_string(), 30)),
        SimpleStruct::Animal(Animal::new("Spider".to_string(), 8)),
        SimpleStruct::Plant(Plant::new("Sunflower".to_string(), 180)),
    ];
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn test_combine_latest_with_identical_streams_emits_updates() {
    // Arrange
    let (stream1_sender, stream1_receiver) = unbounded_channel();
    let stream1 = UnboundedReceiverStream::new(stream1_receiver.into_inner());

    let (stream2_sender, stream2_receiver) = unbounded_channel();
    let stream2 = UnboundedReceiverStream::new(stream2_receiver.into_inner());

    let combined_stream = stream1.combine_latest(vec![stream2], FILTER2);
    let mut combined_stream = Box::pin(combined_stream);

    // Act
    stream1_sender
        .send(SimpleStruct::Person(Person::new("Alice".to_string(), 25)))
        .unwrap();
    stream2_sender
        .send(SimpleStruct::Person(Person::new("Bob".to_string(), 30)))
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Person(Person::new("Alice".to_string(), 25)),
        SimpleStruct::Person(Person::new("Bob".to_string(), 30)),
    ];
    assert_eq!(actual, expected);

    // Act
    stream1_sender
        .send(SimpleStruct::Person(Person::new("Charlie".to_string(), 35)))
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Person(Person::new("Bob".to_string(), 30)),
        SimpleStruct::Person(Person::new("Charlie".to_string(), 35)),
    ];
    assert_eq!(actual, expected);

    // Act
    stream2_sender
        .send(SimpleStruct::Person(Person::new("Diane".to_string(), 40)))
        .unwrap();

    // Assert
    let state = combined_stream.next().await.unwrap();
    let actual: Vec<SimpleStruct> = state.get_state().iter().map(|s| s.value.clone()).collect();
    let expected = vec![
        SimpleStruct::Person(Person::new("Charlie".to_string(), 35)),
        SimpleStruct::Person(Person::new("Diane".to_string(), 40)),
    ];
    assert_eq!(actual, expected);
}

#[derive(Debug, Clone)]
pub enum Order {
    Person,
    Animal,
    Plant,
}

fn send(order: &Order, senders: &[UnboundedSender<SimpleStruct>]) {
    match order {
        Order::Person => send_person(&senders[0]),
        Order::Animal => send_animal(&senders[1]),
        Order::Plant => send_plant(&senders[2]),
    }
}

fn send_person(sender: &UnboundedSender<SimpleStruct>) {
    sender
        .send(SimpleStruct::Person(Person::new("Alice".to_string(), 25)))
        .unwrap()
}

fn send_animal(sender: &UnboundedSender<SimpleStruct>) {
    sender
        .send(SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)))
        .unwrap()
}

fn send_plant(sender: &UnboundedSender<SimpleStruct>) {
    sender
        .send(SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)))
        .unwrap()
}
