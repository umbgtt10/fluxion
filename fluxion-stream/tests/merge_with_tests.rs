use fluxion_stream::{FluxionChannel, merge_with::MergedStream, timestamped::Timestamped};
use fluxion_test_utils::{
    TestChannels,
    animal::Animal,
    person::Person,
    plant::Plant,
    push,
    test_data::{
        TestData, animal_bird, animal_dog, animal_spider, person, person_alice, person_bob,
        person_charlie, person_dave, plant_fern, plant_oak,
    },
};
use futures::StreamExt;

#[tokio::test]
async fn test_merge_with_empty_streams() {
    // Arrange
    let empty_channel1 = FluxionChannel::<TestData>::empty();
    let empty_channel2 = FluxionChannel::<TestData>::empty();
    let empty_stream1 = empty_channel1.stream;
    let empty_stream2 = empty_channel2.stream;

    // Act
    let result_stream = MergedStream::seed(0)
        .merge_with(
            empty_stream1,
            |ts_new_item: Timestamped<TestData>, state: &mut i32| {
                let seq = ts_new_item.sequence();
                let _ = ts_new_item.into_inner();
                let out = *state;
                Timestamped::with_sequence(out, seq)
            },
        )
        .merge_with(
            empty_stream2,
            |ts_new_item: Timestamped<TestData>, state: &mut i32| {
                let seq = ts_new_item.sequence();
                let _ = ts_new_item.into_inner();
                let out = *state;
                Timestamped::with_sequence(out, seq)
            },
        );

    // Assert
    let result: Vec<Timestamped<i32>> = result_stream.collect().await;
    assert_eq!(result, vec![], "Empty streams should produce empty result");
}

#[tokio::test]
async fn test_merge_with_mixed_empty_and_non_empty_streams() {
    // Arrange
    let (non_empty, empty) = TestChannels::two::<TestData>();
    drop(empty.sender);

    // Keep the Timestamped wrapper for deterministic ordering
    let non_empty_stream = non_empty.stream;
    let empty_stream = empty.stream;

    // Use a simple counter state to verify emissions from the non-empty stream
    let merged_stream = MergedStream::seed(0usize)
        .merge_with(
            non_empty_stream,
            |ts_new_item: Timestamped<TestData>, state: &mut usize| {
                let seq = ts_new_item.sequence();
                let _inner = ts_new_item.into_inner();
                *state += 1;
                let out = *state;
                Timestamped::with_sequence(out, seq)
            },
        )
        .merge_with(
            empty_stream,
            |ts_new_item: Timestamped<TestData>, state: &mut usize| {
                let seq = ts_new_item.sequence();
                let _inner = ts_new_item.into_inner();
                *state += 1; // will never run in this test
                let out = *state;
                Timestamped::with_sequence(out, seq)
            },
        );

    // Act
    push(person_alice(), &non_empty.sender);

    // Assert
    let mut merged_stream = Box::pin(merged_stream);
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.into_inner(),
        1,
        "First emission should increment counter to 1"
    );

    // Act
    push(person_bob(), &non_empty.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.into_inner(),
        2,
        "Second emission should increment counter to 2"
    );

    // Act
    push(person_charlie(), &non_empty.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.into_inner(),
        3,
        "Third emission should increment counter to 3"
    );
}

#[tokio::test]
async fn test_merge_with_similar_streams_emits() {
    // Arrange
    let (channel1, channel2) = TestChannels::two::<TestData>();
    let stream1 = channel1.stream;
    let stream2 = channel2.stream;

    let merged_stream = MergedStream::seed(Repository::new())
        .merge_with(
            stream1,
            |ts_new_item: Timestamped<TestData>, state: &mut Repository| {
                state.from_testdata_timestamped(ts_new_item)
            },
        )
        .merge_with(
            stream2,
            |ts_new_item: Timestamped<TestData>, state: &mut Repository| {
                state.from_testdata_timestamped(ts_new_item)
            },
        );

    // Act
    push(person_alice(), &channel1.sender);

    // Assert
    let mut merged_stream = Box::pin(merged_stream);
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.person_name,
        Some("Alice".to_string()),
        "Repository should contain Alice after first emission"
    );

    // Act
    push(person_bob(), &channel2.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.person_name,
        Some("Bob".to_string()),
        "Repository should contain Bob after second emission"
    );

    // Act
    push(person_charlie(), &channel1.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.person_name,
        Some("Charlie".to_string()),
        "Repository should contain Charlie after third emission"
    );

    // Act
    push(person_dave(), &channel2.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state.person_name,
        Some("Dave".to_string()),
        "Repository should contain Dave after fourth emission"
    );
}

#[tokio::test]
async fn test_merge_with_parallel_processing() {
    // Arrange
    let (channel1, channel2) = TestChannels::two::<TestData>();

    let stream1 = channel1.stream;
    let stream2 = channel2.stream;

    let result_stream = MergedStream::seed(Repository::new())
        .merge_with(
            stream1,
            |ts_new_item: Timestamped<TestData>, state: &mut Repository| {
                state.from_testdata_timestamped(ts_new_item)
            },
        )
        .merge_with(
            stream2,
            |ts_new_item: Timestamped<TestData>, state: &mut Repository| {
                state.from_testdata_timestamped(ts_new_item)
            },
        );

    // Act
    let sender1 = channel1.sender;
    tokio::spawn(async move {
        push(person_alice(), &sender1);
        push(person_bob(), &sender1);
        push(person_charlie(), &sender1);
    });

    let sender2 = channel2.sender;
    tokio::spawn(async move {
        push(animal_dog(), &sender2);
        push(animal_spider(), &sender2);
    });

    // Assert
    let result: Vec<Timestamped<Repository>> = result_stream.collect().await;
    let last = result.last().expect("at least one state").get();
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
}

#[tokio::test]
async fn test_merge_with_large_streams_emits() {
    // Arrange
    let channel1 = FluxionChannel::<TestData>::new();
    let channel2 = FluxionChannel::<TestData>::new();
    let large_stream1 = channel1.stream;
    let large_stream2 = channel2.stream;

    let merged_stream = MergedStream::seed(0)
        .merge_with(
            large_stream1,
            |ts_new_item: Timestamped<TestData>, state: &mut i32| {
                let seq = ts_new_item.sequence();
                let item = ts_new_item.into_inner();
                let num: i32 = match item {
                    TestData::Person(p) => p.age as i32,
                    TestData::Animal(a) => a.legs as i32,
                    TestData::Plant(pl) => pl.height as i32,
                };
                *state += num;
                Timestamped::with_sequence(*state, seq)
            },
        )
        .merge_with(
            large_stream2,
            |ts_new_item: Timestamped<TestData>, state: &mut i32| {
                let seq = ts_new_item.sequence();
                let item = ts_new_item.into_inner();
                let num: i32 = match item {
                    TestData::Person(p) => p.age as i32,
                    TestData::Animal(a) => a.legs as i32,
                    TestData::Plant(pl) => pl.height as i32,
                };
                *state += num;
                Timestamped::with_sequence(*state, seq)
            },
        );

    // Act
    for i in 0..10000 {
        push(person(i.to_string(), i as u32), &channel1.sender);
    }
    for i in 10000..20000 {
        push(person(i.to_string(), i as u32), &channel2.sender);
    }

    // Assert
    let mut merged_stream = Box::pin(merged_stream);
    let mut count = 0;
    let mut final_state = None;
    for _ in 0..20000 {
        let state = merged_stream.next().await.unwrap();
        count += 1;
        final_state = Some(state);
    }
    assert_eq!(count, 20000, "Should have processed all 20000 emissions");
    assert_eq!(
        final_state.as_ref().map(|ts| *ts.get()),
        Some(199990000),
        "Final accumulated sum should be correct"
    );
}

#[tokio::test]
async fn test_merge_with_hybrid_using_repository_emits() {
    // Arrange
    let (animal, person, plant) = TestChannels::three::<TestData>();

    let animal_stream = animal.stream;
    let person_stream = person.stream;
    let plant_stream = plant.stream;

    let merged_stream = MergedStream::seed(Repository::new())
        .merge_with(
            animal_stream,
            |ts_new_item: Timestamped<TestData>, state| {
                state.from_testdata_timestamped(ts_new_item)
            },
        )
        .merge_with(
            person_stream,
            |ts_new_item: Timestamped<TestData>, state| {
                state.from_testdata_timestamped(ts_new_item)
            },
        )
        .merge_with(plant_stream, |ts_new_item: Timestamped<TestData>, state| {
            state.from_testdata_timestamped(ts_new_item)
        });

    // Act
    push(animal_dog(), &animal.sender);

    // Assert
    let mut merged_stream = Box::pin(merged_stream);
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        *state.get(),
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
    push(animal_bird(), &animal.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        *state.get(),
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
    push(person_alice(), &person.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        *state.get(),
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
    push(person_bob(), &person.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        *state.get(),
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
    push(plant_fern(), &plant.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        *state.get(),
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
    push(plant_oak(), &plant.sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        *state.get(),
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

    /// Accept a timestamped TestData and return a timestamped Repository where
    /// the output preserves the incoming sequence. This centralizes timestamp
    /// handling inside the repository helper instead of in every caller.
    pub fn from_testdata_timestamped(&mut self, ts: Timestamped<TestData>) -> Timestamped<Self> {
        let seq = ts.sequence();
        let out = match ts.into_inner() {
            TestData::Person(p) => self.from_person(p),
            TestData::Animal(a) => self.from_animal(a),
            TestData::Plant(pl) => self.from_plant(pl),
        };
        Timestamped::with_sequence(out, seq)
    }
}

#[tokio::test]
#[should_panic(expected = "User closure panicked on purpose")]
async fn test_merge_with_user_closure_panics() {
    // Arrange
    let channel = FluxionChannel::<TestData>::new();
    let stream = channel.stream;

    // Create a merge_with stream where the closure panics on the second emission
    let merged_stream = MergedStream::seed(0usize).merge_with(
        stream,
        |ts_new_item: Timestamped<TestData>, state: &mut usize| {
            let seq = ts_new_item.sequence();
            let _inner = ts_new_item.into_inner();
            *state += 1;
            if *state == 2 {
                panic!("User closure panicked on purpose");
            }
            let out = *state;
            Timestamped::with_sequence(out, seq)
        },
    );

    let mut merged_stream = Box::pin(merged_stream);

    // Act: First emission should succeed
    push(person_alice(), &channel.sender);
    let first = merged_stream.next().await.unwrap();
    assert_eq!(
        first.into_inner(),
        1,
        "First emission should increment state to 1"
    );

    // Act: Second emission triggers panic
    push(person_bob(), &channel.sender);
    let _second = merged_stream.next().await; // This will panic
}
