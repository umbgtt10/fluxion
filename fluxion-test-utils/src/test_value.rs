use crate::{animal::Animal, person::Person, plant::Plant};
use fluxion_stream::{sequenced::Sequenced, sequenced_channel::UnboundedSender};
use futures::Stream;
use futures::StreamExt;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Variant {
    Animal,
    Person,
    Plant,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestValue {
    Person(Person),
    Animal(Animal),
    Plant(Plant),
}

pub fn push<T>(value: T, sender: &UnboundedSender<T>) {
    sender.send(value).unwrap()
}

pub fn person_alice() -> TestValue {
    TestValue::Person(Person::new("Alice".to_string(), 25))
}

pub fn person_bob() -> TestValue {
    TestValue::Person(Person::new("Bob".to_string(), 30))
}

pub fn person_charlie() -> TestValue {
    TestValue::Person(Person::new("Charlie".to_string(), 35))
}

pub fn person_diane() -> TestValue {
    TestValue::Person(Person::new("Diane".to_string(), 40))
}

pub fn animal_dog() -> TestValue {
    TestValue::Animal(Animal::new("Dog".to_string(), 4))
}

pub fn animal_spider() -> TestValue {
    TestValue::Animal(Animal::new("Spider".to_string(), 8))
}

pub fn plant_rose() -> TestValue {
    TestValue::Plant(Plant::new("Rose".to_string(), 15))
}

pub fn plant_sunflower() -> TestValue {
    TestValue::Plant(Plant::new("Sunflower".to_string(), 180))
}

pub fn person_dave() -> TestValue {
    TestValue::Person(Person::new("Dave".to_string(), 28))
}

pub fn animal_ant() -> TestValue {
    TestValue::Animal(Animal::new("Ant".to_string(), 6))
}

pub fn animal_cat() -> TestValue {
    TestValue::Animal(Animal::new("Cat".to_string(), 4))
}

pub fn animal(name: String, legs: u32) -> TestValue {
    TestValue::Animal(Animal::new(name, legs))
}

pub fn person(name: String, age: u32) -> TestValue {
    TestValue::Person(Person::new(name, age))
}

pub fn send(variant: &Variant, senders: &[UnboundedSender<TestValue>]) {
    match variant {
        Variant::Person => push(person_alice(), &senders[0]),
        Variant::Animal => push(animal_dog(), &senders[1]),
        Variant::Plant => push(plant_rose(), &senders[2]),
    }
}

pub async fn expect_variant(
    variant: &Variant,
    results: impl futures::Stream<Item = Sequenced<TestValue>> + Send,
) {
    match variant {
        Variant::Animal => expect_animal(results).await,
        Variant::Person => expect_person(results).await,
        Variant::Plant => expect_plant(results).await,
    }
}

pub async fn expect_person(results: impl Stream<Item = Sequenced<TestValue>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    assert_eq!(state.value, person_alice());
}

pub async fn expect_animal(results: impl Stream<Item = Sequenced<TestValue>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    assert_eq!(state.value, animal_dog());
}

pub async fn expect_plant(results: impl Stream<Item = Sequenced<TestValue>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    assert_eq!(state.value, plant_rose());
}

impl Display for TestValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestValue::Person(p) => write!(f, "{}", p),
            TestValue::Animal(a) => write!(f, "{}", a),
            TestValue::Plant(p) => write!(f, "{}", p),
        }
    }
}
