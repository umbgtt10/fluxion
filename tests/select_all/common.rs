use crate::select_all::test_data::{animal::Animal, person::Person, plant::Plant};
use fluxion::sequenced::Sequenced;
use fluxion::sequenced_channel::UnboundedSender;
use futures::{Stream, StreamExt};
use std::fmt::{self, Display};

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
