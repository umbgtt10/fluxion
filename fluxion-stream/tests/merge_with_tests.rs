// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{StreamItem, Timestamped};
use fluxion_stream::MergedStream;
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{
    animal::Animal,
    person::Person,
    plant::Plant,
    test_data::{
        animal_bird, animal_dog, animal_spider, person, person_alice, person_bob, person_charlie,
        person_dave, plant_fern, plant_oak, TestData,
    },
};
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_merge_with_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let empty_stream1 = UnboundedReceiverStream::new(rx1);
    drop(tx1);

    let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let empty_stream2 = UnboundedReceiverStream::new(rx2);
    drop(tx2);

    // Act
    let result_stream = MergedStream::seed(0)
        .merge_with::<_, _, _, _, Sequenced<i32>, i32, u64>(
            empty_stream1,
            |_item: TestData, state: &mut i32| {
                *state += 1;
                *state
            },
        )
        .merge_with::<_, _, _, _, Sequenced<i32>, i32, u64>(
            empty_stream2,
            |_item: TestData, state: &mut i32| {
                *state += 1;
                *state
            },
        );

    // Assert
    let result: Vec<StreamItem<Sequenced<i32>>> = result_stream.collect().await;
    assert_eq!(result, vec![], "Empty streams should produce empty result");

    Ok(())
}

#[tokio::test]
async fn test_merge_with_mixed_empty_and_non_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (non_empty_tx, non_empty_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let non_empty_stream = UnboundedReceiverStream::new(non_empty_rx);

    let (empty_tx, empty_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let empty_stream = UnboundedReceiverStream::new(empty_rx);
    drop(empty_tx);

    // Use a simple counter state to verify emissions from the non-empty stream
    let mut merged_stream = MergedStream::seed(0usize)
        .merge_with::<_, _, _, _, Sequenced<usize>, usize, u64>(
            non_empty_stream,
            |_item: TestData, state: &mut usize| {
                *state += 1;
                *state
            },
        )
        .merge_with::<_, _, _, _, Sequenced<usize>, usize, u64>(
            empty_stream,
            |_item: TestData, state: &mut usize| {
                *state += 1; // will never run in this test
                *state
            },
        );

    // Act
    non_empty_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        1,
        "First emission should increment counter to 1"
    );

    // Act
    non_empty_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        2,
        "Second emission should increment counter to 2"
    );

    // Act
    non_empty_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        3,
        "Third emission should increment counter to 3"
    );
    Ok(())
}

#[tokio::test]
async fn test_merge_with_similar_streams_emits() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream1 = UnboundedReceiverStream::new(rx1);

    let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream2 = UnboundedReceiverStream::new(rx2);

    let mut merged_stream = MergedStream::seed(Repository::new())
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            stream1,
            |item: TestData, state: &mut Repository| state.from_testdata(item),
        )
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            stream2,
            |item: TestData, state: &mut Repository| state.from_testdata(item),
        );

    // Act
    tx1.send(Sequenced::new(person_alice()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner().person_name,
        Some("Alice".to_string()),
        "Repository should contain Alice after first emission"
    );

    // Act
    tx2.send(Sequenced::new(person_bob()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner().person_name,
        Some("Bob".to_string()),
        "Repository should contain Bob after second emission"
    );

    // Act
    tx1.send(Sequenced::new(person_charlie()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner().person_name,
        Some("Charlie".to_string()),
        "Repository should contain Charlie after third emission"
    );

    // Act
    tx2.send(Sequenced::new(person_dave()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner().person_name,
        Some("Dave".to_string()),
        "Repository should contain Dave after fourth emission"
    );
    Ok(())
}

#[tokio::test]
async fn test_merge_with_parallel_processing() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream1 = UnboundedReceiverStream::new(rx1);

    let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream2 = UnboundedReceiverStream::new(rx2);

    let merged_stream = MergedStream::seed(Repository::new())
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            stream1,
            |item: TestData, state: &mut Repository| state.from_testdata(item),
        )
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            stream2,
            |item: TestData, state: &mut Repository| state.from_testdata(item),
        );

    // Act
    tokio::spawn(async move {
        tx1.send(Sequenced::new(person_alice())).unwrap();
        tx1.send(Sequenced::new(person_bob())).unwrap();
        tx1.send(Sequenced::new(person_charlie())).unwrap();
    });

    tokio::spawn(async move {
        tx2.send(Sequenced::new(animal_dog())).unwrap();
        tx2.send(Sequenced::new(animal_spider())).unwrap();
    });

    // Assert
    let result: Vec<StreamItem<Sequenced<Repository>>> = merged_stream.collect().await;
    let StreamItem::Value(last) = result.last().expect("at least one state").clone() else {
        panic!("Expected Value");
    };
    let last = last.into_inner();
    assert_eq!(
        last.person_name,
        Some("Charlie".to_string()),
        "Final repository should have Charlie as last person"
    );
    assert_eq!(
        last.animal_species,
        Some("Spider".to_string()),
        "Final repository should have Spider as last animal"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_large_streams_emits() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let large_stream1 = UnboundedReceiverStream::new(rx1);

    let (tx2, rx2) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let large_stream2 = UnboundedReceiverStream::new(rx2);

    let mut merged_stream = MergedStream::seed(0)
        .merge_with::<_, _, _, _, Sequenced<i32>, i32, u64>(
            large_stream1,
            |item: TestData, state: &mut i32| {
                let num: i32 = match item {
                    TestData::Person(p) => p.age as i32,
                    TestData::Animal(a) => a.legs as i32,
                    TestData::Plant(pl) => pl.height as i32,
                };
                *state += num;
                *state
            },
        )
        .merge_with::<_, _, _, _, Sequenced<i32>, i32, u64>(
            large_stream2,
            |item: TestData, state: &mut i32| {
                let num: i32 = match item {
                    TestData::Person(p) => p.age as i32,
                    TestData::Animal(a) => a.legs as i32,
                    TestData::Plant(pl) => pl.height as i32,
                };
                *state += num;
                *state
            },
        );

    // Act
    for i in 0..10000 {
        tx1.send(Sequenced::new(person(i.to_string(), i as u32)))?;
    }
    for i in 10000..20000 {
        tx2.send(Sequenced::new(person(i.to_string(), i as u32)))?;
    }

    // Assert
    let mut count = 0;
    let mut final_state = None;
    for _ in 0..20000 {
        let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
            panic!("Expected Value");
        };
        count += 1;
        final_state = Some(state.into_inner());
    }
    assert_eq!(count, 20000, "Should have processed all 20000 emissions");
    assert_eq!(
        final_state,
        Some(199990000),
        "Final accumulated sum should be correct"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_hybrid_using_repository_emits() -> anyhow::Result<()> {
    // Arrange
    let (animal_tx, animal_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let animal_stream = UnboundedReceiverStream::new(animal_rx);

    let (person_tx, person_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let person_stream = UnboundedReceiverStream::new(person_rx);

    let (plant_tx, plant_rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let plant_stream = UnboundedReceiverStream::new(plant_rx);

    let mut merged_stream = MergedStream::seed(Repository::new())
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            animal_stream,
            |item: TestData, state| state.from_testdata(item),
        )
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            person_stream,
            |item: TestData, state| state.from_testdata(item),
        )
        .merge_with::<_, _, _, _, Sequenced<Repository>, Repository, u64>(
            plant_stream,
            |item: TestData, state| state.from_testdata(item),
        );

    // Act
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.clone().into_inner(),
        Repository {
            last_animal: Some(Animal::new("Dog".to_string(), 4)),
            last_person: None,
            last_plant: None,
            animal_species: Some("Dog".to_string()),
            animal_legs: Some(4),
            person_name: None,
            person_age: None,
            plant_species: None,
            plant_height: None,
        }
    );

    // Act
    animal_tx.send(Sequenced::new(animal_bird()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: None,
            last_plant: None,
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: None,
            person_age: None,
            plant_species: None,
            plant_height: None,
        }
    );

    // Act
    person_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Alice".to_string(), 25)),
            last_plant: None,
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Alice".to_string()),
            person_age: Some(25),
            plant_species: None,
            plant_height: None,
        }
    );

    // Act
    person_tx.send(Sequenced::new(person_bob())).unwrap();

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Bob".to_string(), 30)),
            last_plant: None,
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Bob".to_string()),
            person_age: Some(30),
            plant_species: None,
            plant_height: None,
        }
    );

    // Act
    plant_tx.send(Sequenced::new(plant_fern()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Bob".to_string(), 30)),
            last_plant: Some(Plant::new("Fern".to_string(), 150)),
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Bob".to_string()),
            person_age: Some(30),
            plant_species: Some("Fern".to_string()),
            plant_height: Some(150),
        }
    );

    // Act
    plant_tx.send(Sequenced::new(plant_oak()))?;

    // Assert
    let StreamItem::Value(state) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        state.into_inner(),
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Bob".to_string(), 30)),
            last_plant: Some(Plant::new("Oak".to_string(), 1000)),
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Bob".to_string()),
            person_age: Some(30),
            plant_species: Some("Oak".to_string()),
            plant_height: Some(1000),
        }
    );

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Repository {
    pub last_animal: Option<Animal>,
    pub last_person: Option<Person>,
    pub last_plant: Option<Plant>,

    pub animal_species: Option<String>,
    pub animal_legs: Option<u32>,
    pub person_name: Option<String>,
    pub person_age: Option<u32>,
    pub plant_species: Option<String>,
    pub plant_height: Option<u32>,
}

impl Repository {
    pub fn new() -> Self {
        Self {
            last_animal: None,
            last_person: None,
            last_plant: None,
            animal_species: None,
            animal_legs: None,
            person_name: None,
            person_age: None,
            plant_species: None,
            plant_height: None,
        }
    }

    pub fn update_with_animal(&mut self, animal: Animal) {
        self.last_animal = Some(animal.clone());
        self.animal_species = Some(animal.name);
        self.animal_legs = Some(animal.legs);
    }

    pub fn update_with_person(&mut self, person: Person) {
        self.last_person = Some(person.clone());
        self.person_name = Some(person.name);
        self.person_age = Some(person.age);
    }

    pub fn update_with_plant(&mut self, plant: Plant) {
        self.last_plant = Some(plant.clone());
        self.plant_species = Some(plant.species);
        self.plant_height = Some(plant.height);
    }

    pub fn from_animal(&mut self, animal: Animal) -> Self {
        self.update_with_animal(animal);
        Self {
            last_animal: self.last_animal.clone(),
            last_person: self.last_person.clone(),
            last_plant: self.last_plant.clone(),
            animal_species: self.animal_species.clone(),
            animal_legs: self.animal_legs,
            person_name: self.person_name.clone(),
            person_age: self.person_age,
            plant_species: self.plant_species.clone(),
            plant_height: self.plant_height,
        }
    }

    pub fn from_person(&mut self, person: Person) -> Self {
        self.update_with_person(person);
        Self {
            last_animal: self.last_animal.clone(),
            last_person: self.last_person.clone(),
            last_plant: self.last_plant.clone(),
            animal_species: self.animal_species.clone(),
            animal_legs: self.animal_legs,
            person_name: self.person_name.clone(),
            person_age: self.person_age,
            plant_species: self.plant_species.clone(),
            plant_height: self.plant_height,
        }
    }

    pub fn from_plant(&mut self, plant: Plant) -> Self {
        self.update_with_plant(plant);
        Self {
            last_animal: self.last_animal.clone(),
            last_person: self.last_person.clone(),
            last_plant: self.last_plant.clone(),
            animal_species: self.animal_species.clone(),
            animal_legs: self.animal_legs,
            person_name: self.person_name.clone(),
            person_age: self.person_age,
            plant_species: self.plant_species.clone(),
            plant_height: self.plant_height,
        }
    }

    pub fn from_testdata(&mut self, data: TestData) -> Self {
        match data {
            TestData::Person(p) => self.from_person(p),
            TestData::Animal(a) => self.from_animal(a),
            TestData::Plant(pl) => self.from_plant(pl),
        }
    }

    /// Accept a Timestamped TestData and return a Timestamped Repository where
    /// the output preserves the incoming sequence. This centralizes sequence
    /// handling inside the repository helper instead of in every caller.
    pub fn from_testdata_timestamped(&mut self, ts: Sequenced<TestData>) -> Sequenced<Self> {
        let seq = ts.timestamp();
        let out = self.from_testdata(ts.into_inner());
        Sequenced::with_timestamp(out, seq)
    }
}

#[tokio::test]
#[should_panic(expected = "User closure panicked on purpose")]
async fn test_merge_with_user_closure_panics() {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = UnboundedReceiverStream::new(rx);

    // Create a merge_with stream where the closure panics on the second emission
    let mut merged_stream = MergedStream::seed(0usize)
        .merge_with::<_, _, _, _, Sequenced<usize>, usize, u64>(
            stream,
            |_item: TestData, state: &mut usize| {
                *state += 1;
                if *state == 2 {
                    panic!("User closure panicked on purpose");
                }
                *state
            },
        );

    // Act: First emission should succeed
    tx.send(Sequenced::new(person_alice())).unwrap();
    let StreamItem::Value(first) = merged_stream.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        first.into_inner(),
        1,
        "First emission should increment state to 1"
    );

    // Act: Second emission triggers panic
    tx.send(Sequenced::new(person_bob())).unwrap();
    let _second = merged_stream.next().await; // This will panic
}

#[tokio::test]
async fn test_merge_with_chaining_with_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with map_ordered that doubles the counter
    let mut result = MergedStream::seed(0)
        .merge_with::<_, _, _, _, Sequenced<usize>, usize, u64>(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 2)
        });

    // Send first value
    tx.send(Sequenced::new(person_alice()))?;

    // Assert first result: state=1, doubled=2
    let StreamItem::Value(first) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(first.into_inner(), 2, "First emission: (0+1)*2 = 2");

    // Send second value
    tx.send(Sequenced::new(person_bob()))?;

    // Assert second result: state=2, doubled=4
    let StreamItem::Value(second) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(second.into_inner(), 4, "Second emission: (1+1)*2 = 4");

    Ok(())
}
#[tokio::test]
async fn test_merge_with_chaining_with_filter_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with filter_ordered (only values > 2)
    let mut result = MergedStream::seed(0)
        .merge_with::<_, _, _, _, Sequenced<usize>, usize, u64>(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .filter_ordered(|&value| value > 2);

    // Send first value - state will be 1 (filtered out)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state will be 2 (filtered out)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state will be 3 (kept)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: only the third emission passes the filter
    let StreamItem::Value(first_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        first_kept.into_inner(),
        3,
        "Third emission passes filter: 3 > 2"
    );

    // Send fourth value - state will be 4 (kept)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: fourth emission also passes
    let StreamItem::Value(second_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        second_kept.into_inner(),
        4,
        "Fourth emission passes filter: 4 > 2"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_multiple_operators() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<Sequenced<TestData>>();
    let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);

    // Act: Chain merge_with with map, filter, and another map
    let mut result = MergedStream::seed(0)
        .merge_with::<_, _, _, _, Sequenced<usize>, usize, u64>(stream, |_item: TestData, state| {
            *state += 1;
            *state
        })
        .into_fluxion_stream()
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value * 3)
        })
        .filter_ordered(|&value| value > 6)
        .map_ordered(|seq| {
            let value = seq.into_inner();
            Sequenced::new(value + 10)
        });

    // Send first value - state: 1, *3=3 (filtered out: 3 <= 6)
    tx.send(Sequenced::new(person_alice()))?;

    // Send second value - state: 2, *3=6 (filtered out: 6 <= 6)
    tx.send(Sequenced::new(person_bob()))?;

    // Send third value - state: 3, *3=9, +10=19 (kept: 9 > 6)
    tx.send(Sequenced::new(person_charlie()))?;

    // Assert: first kept value
    let StreamItem::Value(first_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        first_kept.into_inner(),
        19,
        "Third emission: 3*3=9, 9+10=19"
    );

    // Send fourth value - state: 4, *3=12, +10=22 (kept: 12 > 6)
    tx.send(Sequenced::new(person_dave()))?;

    // Assert: second kept value
    let StreamItem::Value(second_kept) = result.next().await.unwrap() else {
        panic!("Expected Value");
    };
    assert_eq!(
        second_kept.into_inner(),
        22,
        "Fourth emission: 4*3=12, 12+10=22"
    );

    Ok(())
}
