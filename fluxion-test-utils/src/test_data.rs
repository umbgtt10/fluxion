// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{animal::Animal, person::Person, plant::Plant};
use core::fmt::{self, Display};

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

#[must_use]
pub fn person_alice() -> TestData {
    TestData::Person(Person::new("Alice".to_string(), 25))
}

#[must_use]
pub fn person_bob() -> TestData {
    TestData::Person(Person::new("Bob".to_string(), 30))
}

#[must_use]
pub fn person_charlie() -> TestData {
    TestData::Person(Person::new("Charlie".to_string(), 35))
}

#[must_use]
pub fn person_diane() -> TestData {
    TestData::Person(Person::new("Diane".to_string(), 40))
}

#[must_use]
pub fn animal_dog() -> TestData {
    TestData::Animal(Animal::new("Dog".to_string(), 4))
}

#[must_use]
pub fn animal_spider() -> TestData {
    TestData::Animal(Animal::new("Spider".to_string(), 8))
}

#[must_use]
pub fn animal_bird() -> TestData {
    TestData::Animal(Animal::new("Bird".to_string(), 2))
}

#[must_use]
pub fn plant_rose() -> TestData {
    TestData::Plant(Plant::new("Rose".to_string(), 15))
}

#[must_use]
pub fn plant_sunflower() -> TestData {
    TestData::Plant(Plant::new("Sunflower".to_string(), 180))
}

#[must_use]
pub fn person_dave() -> TestData {
    TestData::Person(Person::new("Dave".to_string(), 28))
}

#[must_use]
pub fn animal_ant() -> TestData {
    TestData::Animal(Animal::new("Ant".to_string(), 6))
}

#[must_use]
pub fn animal_cat() -> TestData {
    TestData::Animal(Animal::new("Cat".to_string(), 4))
}

#[must_use]
pub const fn animal(name: String, legs: u32) -> TestData {
    TestData::Animal(Animal::new(name, legs))
}

#[must_use]
pub const fn person(name: String, age: u32) -> TestData {
    TestData::Person(Person::new(name, age))
}

#[must_use]
pub const fn plant(name: String, height: u32) -> TestData {
    TestData::Plant(Plant::new(name, height))
}

#[must_use]
pub fn plant_fern() -> TestData {
    TestData::Plant(Plant::new("Fern".to_string(), 150))
}

#[must_use]
pub fn plant_oak() -> TestData {
    TestData::Plant(Plant::new("Oak".to_string(), 1000))
}

impl Display for TestData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Person(p) => write!(f, "{p}"),
            Self::Animal(a) => write!(f, "{a}"),
            Self::Plant(p) => write!(f, "{p}"),
        }
    }
}
