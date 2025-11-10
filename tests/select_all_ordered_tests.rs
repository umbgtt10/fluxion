use crate::test_data::{animal::Animal, person::Person, plant::Plant};
use fluxion::select_all_ordered::SelectAllExt;
use fluxion::sequenced::Sequenced;
use fluxion::sequenced_channel::{UnboundedSender, unbounded_channel};
use futures::{Stream, StreamExt};
use std::fmt::{self, Display};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[path = "./test_data/mod.rs"]
pub mod test_data;

#[tokio::test]
async fn test_select_all_ordered_all_permutations() {
    select_all_ordered_template_test(Order::Person, Order::Animal, Order::Plant).await;
    select_all_ordered_template_test(Order::Person, Order::Plant, Order::Animal).await;
    select_all_ordered_template_test(Order::Plant, Order::Animal, Order::Person).await;
    select_all_ordered_template_test(Order::Plant, Order::Person, Order::Animal).await;
    select_all_ordered_template_test(Order::Animal, Order::Person, Order::Plant).await;
    select_all_ordered_template_test(Order::Animal, Order::Plant, Order::Person).await;
}

/// Test template for `select_all_ordered` that verifies temporal order preservation in stream merging.
/// Sets up channels for Person/Animal/Plant, merges streams, sends in specified order, and asserts correct reception.
/// Crucial for deterministic async processing; ensures `select_all_ordered` avoids race conditions of `select_all`.
/// Called with different permutations to validate robustness across send sequences.
/// Validates sequence-based ordering over poll-time randomness for reliable event handling.
async fn select_all_ordered_template_test(order1: Order, order2: Order, order3: Order) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];
    let streams = vec![person_stream, animal_stream, plant_stream];

    let results = streams.select_all_ordered();

    // Act
    send(order1.clone(), senders.clone());
    send(order2.clone(), senders.clone());
    send(order3.clone(), senders.clone());

    // Assert
    let mut results = Box::pin(results);

    assert(order1, &mut results).await;
    assert(order2, &mut results).await;
    assert(order3, &mut results).await;
}

#[derive(Debug, Clone)]
pub enum Order {
    Animal,
    Person,
    Plant,
}

// Simple enum without sequence numbers - just the data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleValue {
    Person(Person),
    Animal(Animal),
    Plant(Plant),
}

// The type used in streams - automatically sequenced
pub type StreamValue = Sequenced<SimpleValue>;

pub fn send(order: Order, senders: Vec<UnboundedSender<SimpleValue>>) {
    match order {
        Order::Person => send_person(senders[0].clone()),
        Order::Animal => send_animal(senders[1].clone()),
        Order::Plant => send_plant(senders[2].clone()),
    }
}

pub async fn assert(order: Order, results: impl futures::Stream<Item = StreamValue> + Send) {
    match order {
        Order::Animal => assert_animal_received(results).await,
        Order::Person => assert_person_received(results).await,
        Order::Plant => assert_plant_received(results).await,
    }
}

pub fn send_person(sender: UnboundedSender<SimpleValue>) {
    sender
        .send(SimpleValue::Person(Person::new("Alice".to_string(), 25)))
        .unwrap()
}

pub fn send_animal(sender: UnboundedSender<SimpleValue>) {
    sender
        .send(SimpleValue::Animal(Animal::new("Dog".to_string(), 4)))
        .unwrap()
}

pub fn send_plant(sender: UnboundedSender<SimpleValue>) {
    sender
        .send(SimpleValue::Plant(Plant::new("Rose".to_string(), 15)))
        .unwrap()
}

pub async fn assert_person_received(results: impl Stream<Item = StreamValue> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleValue::Person(p) => {
            assert_eq!(p, Person::new("Alice".to_string(), 25));
        }
        _ => panic!("Expected Person, got {:?}", state),
    }
}

pub async fn assert_animal_received(results: impl Stream<Item = StreamValue> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleValue::Animal(a) => {
            assert_eq!(a, Animal::new("Dog".to_string(), 4));
        }
        _ => panic!("Expected Animal, got {:?}", state),
    }
}

pub async fn assert_plant_received(results: impl Stream<Item = StreamValue> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleValue::Plant(p) => {
            assert_eq!(p, Plant::new("Rose".to_string(), 15));
        }
        _ => panic!("Expected Plant, got {:?}", state),
    }
}

impl Display for SimpleValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleValue::Person(p) => write!(f, "{}", p),
            SimpleValue::Animal(a) => write!(f, "{}", a),
            SimpleValue::Plant(p) => write!(f, "{}", p),
        }
    }
}
