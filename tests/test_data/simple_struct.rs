use crate::test_data::{animal::Animal, person::Person, plant::Plant};
use fluxion::{sequenced::Sequenced, sequenced_channel::UnboundedSender};
use futures::Stream;
use futures::StreamExt;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Order {
    Animal,
    Person,
    Plant,
}

// Simple enum without sequence numbers - just the data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimpleStruct {
    Person(Person),
    Animal(Animal),
    Plant(Plant),
}

pub fn send(order: &Order, senders: &[UnboundedSender<SimpleStruct>]) {
    match order {
        Order::Person => send_person(&senders[0]),
        Order::Animal => send_animal(&senders[1]),
        Order::Plant => send_plant(&senders[2]),
    }
}

pub async fn assert(
    order: &Order,
    results: impl futures::Stream<Item = Sequenced<SimpleStruct>> + Send,
) {
    match order {
        Order::Animal => assert_animal_received(results).await,
        Order::Person => assert_person_received(results).await,
        Order::Plant => assert_plant_received(results).await,
    }
}

pub fn send_person(sender: &UnboundedSender<SimpleStruct>) {
    sender
        .send(SimpleStruct::Person(Person::new("Alice".to_string(), 25)))
        .unwrap()
}

pub fn send_animal(sender: &UnboundedSender<SimpleStruct>) {
    sender
        .send(SimpleStruct::Animal(Animal::new("Dog".to_string(), 4)))
        .unwrap()
}

pub fn send_plant(sender: &UnboundedSender<SimpleStruct>) {
    sender
        .send(SimpleStruct::Plant(Plant::new("Rose".to_string(), 15)))
        .unwrap()
}

pub async fn assert_person_received(results: impl Stream<Item = Sequenced<SimpleStruct>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleStruct::Person(p) => {
            assert_eq!(p, Person::new("Alice".to_string(), 25));
        }
        _ => panic!("Expected Person, got {:?}", state),
    }
}

pub async fn assert_animal_received(results: impl Stream<Item = Sequenced<SimpleStruct>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleStruct::Animal(a) => {
            assert_eq!(a, Animal::new("Dog".to_string(), 4));
        }
        _ => panic!("Expected Animal, got {:?}", state),
    }
}

pub async fn assert_plant_received(results: impl Stream<Item = Sequenced<SimpleStruct>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleStruct::Plant(p) => {
            assert_eq!(p, Plant::new("Rose".to_string(), 15));
        }
        _ => panic!("Expected Plant, got {:?}", state),
    }
}

impl Display for SimpleStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleStruct::Person(p) => write!(f, "{}", p),
            SimpleStruct::Animal(a) => write!(f, "{}", a),
            SimpleStruct::Plant(p) => write!(f, "{}", p),
        }
    }
}
