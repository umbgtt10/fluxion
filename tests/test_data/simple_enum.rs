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
pub enum SimpleEnum {
    Person(Person),
    Animal(Animal),
    Plant(Plant),
}

// Constants for test data
pub fn alice() -> SimpleEnum {
    SimpleEnum::Person(Person::new("Alice".to_string(), 25))
}

pub fn bob() -> SimpleEnum {
    SimpleEnum::Person(Person::new("Bob".to_string(), 30))
}

pub fn charlie() -> SimpleEnum {
    SimpleEnum::Person(Person::new("Charlie".to_string(), 35))
}

pub fn diane() -> SimpleEnum {
    SimpleEnum::Person(Person::new("Diane".to_string(), 40))
}

pub fn dog() -> SimpleEnum {
    SimpleEnum::Animal(Animal::new("Dog".to_string(), 4))
}

pub fn spider() -> SimpleEnum {
    SimpleEnum::Animal(Animal::new("Spider".to_string(), 8))
}

pub fn rose() -> SimpleEnum {
    SimpleEnum::Plant(Plant::new("Rose".to_string(), 15))
}

pub fn sunflower() -> SimpleEnum {
    SimpleEnum::Plant(Plant::new("Sunflower".to_string(), 180))
}

pub fn dave() -> SimpleEnum {
    SimpleEnum::Person(Person::new("Dave".to_string(), 28))
}

pub fn ant() -> SimpleEnum {
    SimpleEnum::Animal(Animal::new("Ant".to_string(), 6))
}

pub fn cat() -> SimpleEnum {
    SimpleEnum::Animal(Animal::new("Cat".to_string(), 4))
}

// Helper functions for creating dynamic instances
pub fn animal(name: String, legs: u32) -> SimpleEnum {
    SimpleEnum::Animal(Animal::new(name, legs))
}

pub fn person(name: String, age: u32) -> SimpleEnum {
    SimpleEnum::Person(Person::new(name, age))
}

pub fn send(order: &Order, senders: &[UnboundedSender<SimpleEnum>]) {
    match order {
        Order::Person => send_alice(&senders[0]),
        Order::Animal => send_dog(&senders[1]),
        Order::Plant => send_rose(&senders[2]),
    }
}

pub async fn assert(
    order: &Order,
    results: impl futures::Stream<Item = Sequenced<SimpleEnum>> + Send,
) {
    match order {
        Order::Animal => assert_animal_received(results).await,
        Order::Person => assert_person_received(results).await,
        Order::Plant => assert_plant_received(results).await,
    }
}

pub async fn assert_person_received(results: impl Stream<Item = Sequenced<SimpleEnum>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleEnum::Person(p) => {
            assert_eq!(p, Person::new("Alice".to_string(), 25));
        }
        _ => panic!("Expected Person, got {:?}", state),
    }
}

pub async fn assert_animal_received(results: impl Stream<Item = Sequenced<SimpleEnum>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleEnum::Animal(a) => {
            assert_eq!(a, Animal::new("Dog".to_string(), 4));
        }
        _ => panic!("Expected Animal, got {:?}", state),
    }
}

pub async fn assert_plant_received(results: impl Stream<Item = Sequenced<SimpleEnum>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    match state.value {
        SimpleEnum::Plant(p) => {
            assert_eq!(p, Plant::new("Rose".to_string(), 15));
        }
        _ => panic!("Expected Plant, got {:?}", state),
    }
}

pub fn send_alice(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Person(Person::new("Alice".to_string(), 25)))
        .unwrap()
}

pub fn send_bob(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Person(Person::new("Bob".to_string(), 30)))
        .unwrap()
}

pub fn send_charlie(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Person(Person::new("Charlie".to_string(), 35)))
        .unwrap()
}

pub fn send_diane(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Person(Person::new("Diane".to_string(), 40)))
        .unwrap()
}

pub fn send_dave(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Person(Person::new("Dave".to_string(), 45)))
        .unwrap()
}

pub fn send_dog(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Animal(Animal::new("Dog".to_string(), 4)))
        .unwrap()
}

pub fn send_spider(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Animal(Animal::new("Spider".to_string(), 8)))
        .unwrap()
}

pub fn send_ant(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Animal(Animal::new("Ant".to_string(), 6)))
        .unwrap()
}

pub fn send_cat(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Animal(Animal::new("Cat".to_string(), 4)))
        .unwrap()
}

pub fn send_sunflower(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Plant(Plant::new("Sunflower".to_string(), 180)))
        .unwrap()
}

pub fn send_rose(sender: &UnboundedSender<SimpleEnum>) {
    sender
        .send(SimpleEnum::Plant(Plant::new("Rose".to_string(), 15)))
        .unwrap()
}

impl Display for SimpleEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleEnum::Person(p) => write!(f, "{}", p),
            SimpleEnum::Animal(a) => write!(f, "{}", a),
            SimpleEnum::Plant(p) => write!(f, "{}", p),
        }
    }
}
