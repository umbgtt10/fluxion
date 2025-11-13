use crate::fluxion_channel::sequenced_channel::UnboundedSender;
use crate::sequenced::Sequenced;
use crate::{animal::Animal, person::Person, plant::Plant};
use futures::Stream;
use futures::StreamExt;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum DataVariant {
    Animal,
    Person,
    Plant,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestData {
    Person(Person),
    Animal(Animal),
    Plant(Plant),
}

pub fn push<T>(value: T, sender: &UnboundedSender<T>) {
    sender.send(value).unwrap()
}

pub fn person_alice() -> TestData {
    TestData::Person(Person::new("Alice".to_string(), 25))
}

pub fn person_bob() -> TestData {
    TestData::Person(Person::new("Bob".to_string(), 30))
}

pub fn person_charlie() -> TestData {
    TestData::Person(Person::new("Charlie".to_string(), 35))
}

pub fn person_diane() -> TestData {
    TestData::Person(Person::new("Diane".to_string(), 40))
}

pub fn animal_dog() -> TestData {
    TestData::Animal(Animal::new("Dog".to_string(), 4))
}

pub fn animal_spider() -> TestData {
    TestData::Animal(Animal::new("Spider".to_string(), 8))
}

pub fn animal_bird() -> TestData {
    TestData::Animal(Animal::new("Bird".to_string(), 2))
}

pub fn plant_rose() -> TestData {
    TestData::Plant(Plant::new("Rose".to_string(), 15))
}

pub fn plant_sunflower() -> TestData {
    TestData::Plant(Plant::new("Sunflower".to_string(), 180))
}

pub fn person_dave() -> TestData {
    TestData::Person(Person::new("Dave".to_string(), 28))
}

pub fn animal_ant() -> TestData {
    TestData::Animal(Animal::new("Ant".to_string(), 6))
}

pub fn animal_cat() -> TestData {
    TestData::Animal(Animal::new("Cat".to_string(), 4))
}

pub fn animal(name: String, legs: u32) -> TestData {
    TestData::Animal(Animal::new(name, legs))
}

pub fn person(name: String, age: u32) -> TestData {
    TestData::Person(Person::new(name, age))
}

pub fn plant(name: String, height: u32) -> TestData {
    TestData::Plant(Plant::new(name, height))
}

pub fn plant_fern() -> TestData {
    TestData::Plant(Plant::new("Fern".to_string(), 150))
}

pub fn plant_oak() -> TestData {
    TestData::Plant(Plant::new("Oak".to_string(), 1000))
}

pub fn send_variant(variant: &DataVariant, senders: &[UnboundedSender<TestData>]) {
    match variant {
        DataVariant::Person => push(person_alice(), &senders[0]),
        DataVariant::Animal => push(animal_dog(), &senders[1]),
        DataVariant::Plant => push(plant_rose(), &senders[2]),
    }
}

pub async fn expect_variant(
    variant: &DataVariant,
    results: impl futures::Stream<Item = Sequenced<TestData>> + Send,
) {
    match variant {
        DataVariant::Animal => expect_animal(results).await,
        DataVariant::Person => expect_person(results).await,
        DataVariant::Plant => expect_plant(results).await,
    }
}

pub async fn expect_person(results: impl Stream<Item = Sequenced<TestData>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    assert_eq!(state.value, person_alice());
}

pub async fn expect_animal(results: impl Stream<Item = Sequenced<TestData>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    assert_eq!(state.value, animal_dog());
}

pub async fn expect_plant(results: impl Stream<Item = Sequenced<TestData>> + Send) {
    let state = Box::pin(results).next().await.unwrap();
    assert_eq!(state.value, plant_rose());
}

impl Display for TestData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestData::Person(p) => write!(f, "{}", p),
            TestData::Animal(a) => write!(f, "{}", a),
            TestData::Plant(p) => write!(f, "{}", p),
        }
    }
}
