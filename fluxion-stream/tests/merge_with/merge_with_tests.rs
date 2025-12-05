// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream::MergedStream;
use fluxion_test_utils::Sequenced;
use fluxion_test_utils::{
    animal::Animal,
    assert_stream_ended,
    person::Person,
    plant::Plant,
    test_channel,
    test_data::{
        animal_bird, animal_dog, animal_spider, person, person_alice, person_bob, person_charlie,
        person_dave, plant_fern, plant_oak, TestData,
    },
    unwrap_stream,
};
use tokio::spawn;

#[tokio::test]
async fn test_merge_with_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, empty_stream1) = test_channel::<Sequenced<TestData>>();
    drop(tx1);

    let (tx2, empty_stream2) = test_channel::<Sequenced<TestData>>();
    drop(tx2);

    // Act
    let mut result_stream = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(empty_stream1, |_item: TestData, state: &mut i32| {
            *state += 1;
            *state
        })
        .merge_with(empty_stream2, |_item: TestData, state: &mut i32| {
            *state += 1;
            *state
        });

    // Assert
    assert_stream_ended(&mut result_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_merge_with_mixed_empty_and_non_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let (non_empty_tx, non_empty_stream) = test_channel::<Sequenced<TestData>>();
    let (empty_tx, empty_stream) = test_channel::<Sequenced<TestData>>();
    drop(empty_tx);

    // Use a simple counter state to verify emissions from the non-empty stream
    let mut result = MergedStream::seed::<Sequenced<usize>>(0usize)
        .merge_with(non_empty_stream, |_item: TestData, state: &mut usize| {
            *state += 1;
            *state
        })
        .merge_with(empty_stream, |_item: TestData, state: &mut usize| {
            *state += 1; // will never run in this test
            *state
        });

    // Act
    non_empty_tx.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.into_inner(),
        1,
        "First emission should increment counter to 1"
    );

    // Act
    non_empty_tx.send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.into_inner(),
        2,
        "Second emission should increment counter to 2"
    );

    // Act
    non_empty_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100).await.into_inner(),
        3,
        "Third emission should increment counter to 3"
    );
    Ok(())
}

#[tokio::test]
async fn test_merge_with_similar_streams_emits() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel::<Sequenced<TestData>>();

    let mut result = MergedStream::seed::<Sequenced<Repository>>(Repository::new())
        .merge_with(stream1, |item: TestData, state: &mut Repository| {
            state.from_testdata(item)
        })
        .merge_with(stream2, |item: TestData, state: &mut Repository| {
            state.from_testdata(item)
        });

    // Act
    tx1.send(Sequenced::new(person_alice()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100)
            .await
            .into_inner()
            .person_name,
        Some("Alice".to_string()),
        "Repository should contain Alice after first emission"
    );

    // Act
    tx2.send(Sequenced::new(person_bob()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100)
            .await
            .into_inner()
            .person_name,
        Some("Bob".to_string()),
        "Repository should contain Bob after second emission"
    );

    // Act
    tx1.send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100)
            .await
            .into_inner()
            .person_name,
        Some("Charlie".to_string()),
        "Repository should contain Charlie after third emission"
    );

    // Act
    tx2.send(Sequenced::new(person_dave()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 100)
            .await
            .into_inner()
            .person_name,
        Some("Dave".to_string()),
        "Repository should contain Dave after fourth emission"
    );

    Ok(())
}

#[tokio::test]
async fn test_merge_with_parallel_processing() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<Sequenced<TestData>>();
    let (tx2, stream2) = test_channel::<Sequenced<TestData>>();

    let mut result = MergedStream::seed::<Sequenced<Repository>>(Repository::new())
        .merge_with(stream1, |item: TestData, state: &mut Repository| {
            state.from_testdata(item)
        })
        .merge_with(stream2, |item: TestData, state: &mut Repository| {
            state.from_testdata(item)
        });

    // Act
    spawn(async move {
        tx1.send(Sequenced::new(person_alice())).unwrap();
        tx1.send(Sequenced::new(person_bob())).unwrap();
        tx1.send(Sequenced::new(person_charlie())).unwrap();
    });

    spawn(async move {
        tx2.send(Sequenced::new(animal_dog())).unwrap();
        tx2.send(Sequenced::new(animal_spider())).unwrap();
    });

    // Assert - Wait for all 5 emissions (3 person + 2 animal)
    let mut last = None;
    for _ in 0..5 {
        let item = unwrap_stream(&mut result, 500).await;
        last = Some(item.into_inner());
    }

    let last = last.expect("Should have received at least one emission");
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
    let (tx1, large_stream1) = test_channel::<Sequenced<TestData>>();
    let (tx2, large_stream2) = test_channel::<Sequenced<TestData>>();

    let mut result = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(large_stream1, |item: TestData, state: &mut i32| {
            let num: i32 = match item {
                TestData::Person(p) => p.age as i32,
                TestData::Animal(a) => a.legs as i32,
                TestData::Plant(pl) => pl.height as i32,
            };
            *state += num;
            *state
        })
        .merge_with(large_stream2, |item: TestData, state: &mut i32| {
            let num: i32 = match item {
                TestData::Person(p) => p.age as i32,
                TestData::Animal(a) => a.legs as i32,
                TestData::Plant(pl) => pl.height as i32,
            };
            *state += num;
            *state
        });

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
        let state = unwrap_stream(&mut result, 100).await;
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
    let (animal_tx, animal_stream) = test_channel::<Sequenced<TestData>>();
    let (person_tx, person_stream) = test_channel::<Sequenced<TestData>>();
    let (plant_tx, plant_stream) = test_channel::<Sequenced<TestData>>();

    let mut result = MergedStream::seed::<Sequenced<Repository>>(Repository::new())
        .merge_with(animal_stream, |item: TestData, state| {
            state.from_testdata(item)
        })
        .merge_with(person_stream, |item: TestData, state| {
            state.from_testdata(item)
        })
        .merge_with(plant_stream, |item: TestData, state| {
            state.from_testdata(item)
        });

    // Act
    animal_tx.send(Sequenced::new(animal_dog()))?;

    // Assert
    let state = unwrap_stream(&mut result, 100).await;
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
    let state = unwrap_stream(&mut result, 100).await;
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
    let state = unwrap_stream(&mut result, 100).await;
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
    let state = unwrap_stream(&mut result, 100).await;
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
    let state = unwrap_stream(&mut result, 100).await;
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
    let state = unwrap_stream(&mut result, 100).await;
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
        self.animal_species = Some(animal.species);
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
    let (tx, stream) = test_channel::<Sequenced<TestData>>();

    // Create a merge_with stream where the closure panics on the second emission
    let mut result = MergedStream::seed::<Sequenced<usize>>(0usize).merge_with(
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
    assert_eq!(
        unwrap_stream(&mut result, 100).await.into_inner(),
        1,
        "First emission should increment state to 1"
    );

    // Act: Second emission triggers panic
    tx.send(Sequenced::new(person_bob())).unwrap();
    let _second = unwrap_stream(&mut result, 100).await; // This will panic
}

#[tokio::test]
async fn test_merge_with_into_fluxion_stream_standalone() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Convert to FluxionStream without chaining
    let mut fluxion_stream = merged.into_fluxion_stream();

    // Assert: Can still use as stream
    tx.send(Sequenced::with_timestamp(10, 1))?;
    assert_eq!(
        unwrap_stream(&mut fluxion_stream, 100).await.into_inner(),
        10
    );

    tx.send(Sequenced::with_timestamp(20, 2))?;
    assert_eq!(
        unwrap_stream(&mut fluxion_stream, 100).await.into_inner(),
        30
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_into_fluxion_stream_empty() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act: Convert empty stream to FluxionStream
    let mut fluxion_stream = merged.into_fluxion_stream();

    // Drop sender immediately to end stream
    drop(tx);

    // Assert: Stream should end without errors
    assert_stream_ended(&mut fluxion_stream, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_merge_with_single_stream_interleaved_emissions() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<i32>>(0).merge_with(stream, |value, state| {
        *state += value;
        *state
    });

    // Act & Assert: Send and verify one at a time
    tx.send(Sequenced::with_timestamp(5, 1))?;
    assert_eq!(unwrap_stream(&mut merged, 100).await.into_inner(), 5);

    tx.send(Sequenced::with_timestamp(10, 2))?;
    assert_eq!(unwrap_stream(&mut merged, 100).await.into_inner(), 15);

    tx.send(Sequenced::with_timestamp(7, 3))?;
    assert_eq!(unwrap_stream(&mut merged, 100).await.into_inner(), 22);
    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_state_mutation_complex() -> anyhow::Result<()> {
    // Arrange
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
    struct ComplexState {
        sum: i32,
        count: usize,
        last_value: Option<i32>,
    }

    let (tx, stream) = test_channel::<Sequenced<i32>>();

    let initial_state = ComplexState {
        sum: 0,
        count: 0,
        last_value: None,
    };

    let mut merged = MergedStream::seed::<Sequenced<ComplexState>>(initial_state).merge_with(
        stream,
        |value, state| {
            state.sum += value;
            state.count += 1;
            state.last_value = Some(value);
            ComplexState {
                sum: state.sum,
                count: state.count,
                last_value: state.last_value,
            }
        },
    );

    // Act & Assert
    tx.send(Sequenced::with_timestamp(10, 1))?;
    assert_eq!(
        unwrap_stream(&mut merged, 100).await.into_inner(),
        ComplexState {
            sum: 10,
            count: 1,
            last_value: Some(10),
        }
    );

    tx.send(Sequenced::with_timestamp(20, 2))?;
    assert_eq!(
        unwrap_stream(&mut merged, 100).await.into_inner(),
        ComplexState {
            sum: 30,
            count: 2,
            last_value: Some(20),
        }
    );

    tx.send(Sequenced::with_timestamp(5, 3))?;
    assert_eq!(
        unwrap_stream(&mut merged, 100).await.into_inner(),
        ComplexState {
            sum: 35,
            count: 3,
            last_value: Some(5),
        }
    );

    drop(tx);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_timestamp_ordering_preserved() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel::<Sequenced<i32>>();

    let mut merged = MergedStream::seed::<Sequenced<Vec<(i32, u64)>>>(Vec::new())
        .merge_with(stream1, |value, state| {
            state.push((value, 1));
            state.clone()
        })
        .merge_with(stream2, |value, state| {
            state.push((value, 2));
            state.clone()
        });

    // Act: Send with specific timestamps
    tx1.send(Sequenced::with_timestamp(10, 1))?;
    tx2.send(Sequenced::with_timestamp(20, 2))?;
    tx1.send(Sequenced::with_timestamp(30, 3))?;
    tx2.send(Sequenced::with_timestamp(40, 4))?;

    // Assert: Items should be processed in timestamp order
    let r1 = unwrap_stream(&mut merged, 100).await;
    assert_eq!(r1.clone().into_inner(), vec![(10, 1)]);
    assert_eq!(r1.timestamp(), 1);

    let r2 = unwrap_stream(&mut merged, 100).await;
    assert_eq!(r2.clone().into_inner(), vec![(10, 1), (20, 2)]);
    assert_eq!(r2.timestamp(), 2);

    let r3 = unwrap_stream(&mut merged, 100).await;
    assert_eq!(r3.clone().into_inner(), vec![(10, 1), (20, 2), (30, 1)]);
    assert_eq!(r3.timestamp(), 3);

    let r4 = unwrap_stream(&mut merged, 100).await;
    assert_eq!(
        r4.clone().into_inner(),
        vec![(10, 1), (20, 2), (30, 1), (40, 2)]
    );
    assert_eq!(r4.timestamp(), 4);

    drop(tx1);
    drop(tx2);

    Ok(())
}

#[tokio::test]
async fn test_merge_with_clone_closure() -> anyhow::Result<()> {
    // Arrange: Test that closure cloning works correctly
    let (tx1, stream1) = test_channel::<Sequenced<i32>>();
    let (tx2, stream2) = test_channel::<Sequenced<i32>>();

    let multiplier = 2;
    let mut merged = MergedStream::seed::<Sequenced<i32>>(0)
        .merge_with(stream1, move |value, state| {
            *state += value * multiplier;
            *state
        })
        .merge_with(stream2, move |value, state| {
            *state += value * multiplier;
            *state
        });

    // Act
    tx1.send(Sequenced::with_timestamp(5, 1))?;
    assert_eq!(unwrap_stream(&mut merged, 100).await.into_inner(), 10); // 5 * 2

    tx2.send(Sequenced::with_timestamp(7, 2))?;
    assert_eq!(unwrap_stream(&mut merged, 100).await.into_inner(), 24); // 10 + (7 * 2)

    drop(tx1);
    drop(tx2);

    Ok(())
}
