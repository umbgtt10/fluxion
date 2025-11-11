use fluxion_stream::{merge_with::MergedStream, timestamped_channel::unbounded_channel};
use fluxion_test_utils::{
    animal::Animal,
    person::Person,
    plant::Plant,
    push,
    test_data::{
        TestData, animal_bird, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
        person_dave, plant_fern, plant_oak,
    },
};
use futures::{StreamExt, stream};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_merge_with_empty_streams() {
    // Arrange
    let empty_stream1 = stream::empty::<i32>();
    let empty_stream2 = stream::empty::<i32>();

    // Act
    let result_stream = MergedStream::seed(0)
        .merge_with(empty_stream1, |num, state| {
            *state += num;
            *state
        })
        .merge_with(empty_stream2, |num, state| {
            *state += num;
            *state
        });

    // Assert
    let result: Vec<i32> = result_stream.collect().await;
    assert_eq!(result, vec![]);
}

#[tokio::test]
async fn test_merge_with_mixed_empty_and_non_empty_streams() {
    // Arrange
    let (non_empty_sender, non_empty_receiver) = unbounded_channel::<TestData>();
    let (_, empty_receiver) = unbounded_channel::<TestData>();

    let non_empty_stream =
        UnboundedReceiverStream::new(non_empty_receiver.into_inner()).map(|ts| ts.value);
    let empty_stream = UnboundedReceiverStream::new(empty_receiver.into_inner()).map(|ts| ts.value);

    // Use a simple counter state to verify emissions from the non-empty stream
    let merged_stream = MergedStream::seed(0usize)
        .merge_with(non_empty_stream, |_: TestData, state: &mut usize| {
            *state += 1;
            *state
        })
        .merge_with(empty_stream, |_: TestData, state: &mut usize| {
            *state += 1; // will never run in this test
            *state
        });

    let mut merged_stream = Box::pin(merged_stream);

    // Act
    push(person_alice(), &non_empty_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 1);

    // Act
    push(person_bob(), &non_empty_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 2);

    // Act
    push(person_charlie(), &non_empty_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 3);
}

#[tokio::test]
async fn test_merge_with_similar_streams_emits() {
    // Arrange
    let (sender1, receiver1) = unbounded_channel::<TestData>();
    let (sender2, receiver2) = unbounded_channel::<TestData>();
    let stream1 = UnboundedReceiverStream::new(receiver1.into_inner()).map(|ts| ts.value);
    let stream2 = UnboundedReceiverStream::new(receiver2.into_inner()).map(|ts| ts.value);

    let merged_stream = MergedStream::seed(Repository::new())
        .merge_with(
            stream1,
            |item: TestData, state: &mut Repository| match item {
                TestData::Person(p) => state.from_person(p),
                TestData::Animal(a) => state.from_animal(a),
                TestData::Plant(pl) => state.from_plant(pl),
            },
        )
        .merge_with(
            stream2,
            |item: TestData, state: &mut Repository| match item {
                TestData::Person(p) => state.from_person(p),
                TestData::Animal(a) => state.from_animal(a),
                TestData::Plant(pl) => state.from_plant(pl),
            },
        );

    let mut merged_stream = Box::pin(merged_stream);

    // Act
    push(person_alice(), &sender1);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state.person_name, Some("Alice".to_string()));

    // Act
    push(person_bob(), &sender2);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state.person_name, Some("Bob".to_string()));

    // Act
    push(person_charlie(), &sender1);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state.person_name, Some("Charlie".to_string()));

    // Act
    push(person_dave(), &sender2);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state.person_name, Some("Dave".to_string()));
}

#[tokio::test]
async fn test_merge_with_parallel_processing() {
    // Arrange
    let (sender1, receiver1) = unbounded_channel::<TestData>();
    let (sender2, receiver2) = unbounded_channel::<TestData>();

    let stream1 = UnboundedReceiverStream::new(receiver1.into_inner()).map(|ts| ts.value);
    let stream2 = UnboundedReceiverStream::new(receiver2.into_inner()).map(|ts| ts.value);

    let result_stream = MergedStream::seed(Repository::new())
        .merge_with(
            stream1,
            |item: TestData, state: &mut Repository| match item {
                TestData::Person(p) => state.from_person(p),
                TestData::Animal(a) => state.from_animal(a),
                TestData::Plant(pl) => state.from_plant(pl),
            },
        )
        .merge_with(
            stream2,
            |item: TestData, state: &mut Repository| match item {
                TestData::Person(p) => state.from_person(p),
                TestData::Animal(a) => state.from_animal(a),
                TestData::Plant(pl) => state.from_plant(pl),
            },
        );

    // Act
    tokio::spawn(async move {
        push(person_alice(), &sender1);
        push(person_bob(), &sender1);
        push(person_charlie(), &sender1);
    });

    tokio::spawn(async move {
        push(animal_dog(), &sender2);
        push(animal_spider(), &sender2);
    });

    // Assert
    let result: Vec<Repository> = result_stream.collect().await;
    let last = result.last().expect("at least one state");
    assert_eq!(last.person_name, Some("Charlie".to_string()));
    assert_eq!(last.animal_species, Some("Spider".to_string()));
}

#[tokio::test]
async fn test_merge_with_large_streams_emits() {
    // Arrange
    let large_stream1 = stream::iter(0..10000);
    let large_stream2 = stream::iter(10000..20000);

    // Act
    let result_stream = MergedStream::seed(0)
        .merge_with(large_stream1, |num, state| {
            *state += num;
            *state
        })
        .merge_with(large_stream2, |num, state| {
            *state += num;
            *state
        });

    // Assert
    let result: Vec<i32> = result_stream.collect().await;
    assert_eq!(result.len(), 20000);
    assert_eq!(result.last(), Some(&199990000));
}

#[tokio::test]
async fn test_merge_with_hybrid_using_repository_emits() {
    // Arrange
    let (animal_sender, animal_receiver) = unbounded_channel::<TestData>();
    let (person_sender, person_receiver) = unbounded_channel::<TestData>();
    let (plant_sender, plant_receiver) = unbounded_channel::<TestData>();

    let animal_stream =
        UnboundedReceiverStream::new(animal_receiver.into_inner()).map(|ts| ts.value);
    let person_stream =
        UnboundedReceiverStream::new(person_receiver.into_inner()).map(|ts| ts.value);
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner()).map(|ts| ts.value);

    let merged_stream = MergedStream::seed(Repository::new())
        .merge_with(animal_stream, |item: TestData, state| match item {
            TestData::Animal(a) => state.from_animal(a),
            TestData::Person(p) => state.from_person(p),
            TestData::Plant(pl) => state.from_plant(pl),
        })
        .merge_with(person_stream, |item: TestData, state| match item {
            TestData::Animal(a) => state.from_animal(a),
            TestData::Person(p) => state.from_person(p),
            TestData::Plant(pl) => state.from_plant(pl),
        })
        .merge_with(plant_stream, |item: TestData, state| match item {
            TestData::Animal(a) => state.from_animal(a),
            TestData::Person(p) => state.from_person(p),
            TestData::Plant(pl) => state.from_plant(pl),
        });

    let mut merged_stream = Box::pin(merged_stream);

    // Act
    push(animal_dog(), &animal_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
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
    push(animal_bird(), &animal_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
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
    push(person_alice(), &person_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
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
    push(person_bob(), &person_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
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
    push(plant_fern(), &plant_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
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
    push(plant_oak(), &plant_sender);

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
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

#[derive(Debug, PartialEq, Default)]
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
}

#[tokio::test]
#[should_panic(expected = "User closure panicked on purpose")]
async fn test_merge_with_user_closure_panics() {
    // Arrange
    let (sender, receiver) = unbounded_channel::<TestData>();
    let stream = UnboundedReceiverStream::new(receiver.into_inner()).map(|ts| ts.value);

    // Create a merge_with stream where the closure panics on the second emission
    let merged_stream =
        MergedStream::seed(0usize).merge_with(stream, |_data, state: &mut usize| {
            *state += 1;
            if *state == 2 {
                panic!("User closure panicked on purpose");
            }
            *state
        });

    let mut merged_stream = Box::pin(merged_stream);

    // Act: First emission should succeed
    push(person_alice(), &sender);
    let first = merged_stream.next().await.unwrap();
    assert_eq!(first, 1);

    // Act: Second emission triggers panic
    push(person_bob(), &sender);
    let _second = merged_stream.next().await; // This will panic
}
