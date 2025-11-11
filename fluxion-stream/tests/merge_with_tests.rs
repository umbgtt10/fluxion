use fluxion_stream::merge_with::MergedStream;
use fluxion_test_utils::animal::Animal;
use fluxion_test_utils::person::Person;
use fluxion_test_utils::plant::Plant;
use futures::StreamExt;
use futures::stream;
use tokio::sync::mpsc;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;

const BUFFER_SIZE: usize = 10;

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
    let (non_empty_sender, non_empty_receiver) = mpsc::channel(BUFFER_SIZE);
    let (_, empty_receiver) = mpsc::channel::<i32>(BUFFER_SIZE);

    let non_empty_stream = ReceiverStream::new(non_empty_receiver);
    let empty_stream = ReceiverStream::new(empty_receiver);

    let merged_stream = MergedStream::seed(0)
        .merge_with(non_empty_stream, |num: i32, state: &mut i32| {
            *state += num;
            *state
        })
        .merge_with(empty_stream, |num: i32, state: &mut i32| {
            *state += num;
            *state
        });

    let mut merged_stream = Box::pin(merged_stream);

    // Act
    non_empty_sender.send(1).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 1);

    // Act
    non_empty_sender.send(3).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 4);

    // Act
    non_empty_sender.send(6).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 10);
}

#[tokio::test]
async fn test_merge_with_similar_streams_emits() {
    // Arrange
    let (sender1, receiver1) = mpsc::channel(BUFFER_SIZE);
    let (sender2, receiver2) = mpsc::channel(BUFFER_SIZE);
    let stream1 = ReceiverStream::new(receiver1);
    let stream2 = ReceiverStream::new(receiver2);

    let merged_stream = MergedStream::seed(0)
        .merge_with(stream1, |num: i32, state: &mut i32| {
            *state += num;
            *state
        })
        .merge_with(stream2, |num: i32, state: &mut i32| {
            *state += num;
            *state
        });

    let mut merged_stream = Box::pin(merged_stream);

    // Act
    sender1.send(1).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 1);

    // Act
    sender2.send(10).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 11);

    // Act
    sender1.send(2).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 13);

    // Act
    sender2.send(20).await.unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(state, 33);
}

#[tokio::test]
async fn test_merge_with_parallel_processing() {
    // Arrange
    let (sender1, receiver1) = mpsc::channel(BUFFER_SIZE);
    let (sender2, receiver2) = mpsc::channel(BUFFER_SIZE);

    let stream1 = ReceiverStream::new(receiver1);
    let stream2 = ReceiverStream::new(receiver2);

    let result_stream = MergedStream::seed(0)
        .merge_with(stream1, |num: i32, state: &mut i32| {
            *state += num;
            *state
        })
        .merge_with(stream2, |num: i32, state: &mut i32| {
            *state += num;
            *state
        });

    // Act
    tokio::spawn(async move {
        sender1.send(1).await.unwrap();
        time::sleep(time::Duration::from_millis(50)).await;
        sender1.send(2).await.unwrap();
        time::sleep(time::Duration::from_millis(50)).await;
        sender1.send(3).await.unwrap();
    });

    tokio::spawn(async move {
        sender2.send(10).await.unwrap();
        time::sleep(time::Duration::from_millis(50)).await;
        sender2.send(20).await.unwrap();
        time::sleep(time::Duration::from_millis(50)).await;
        sender2.send(30).await.unwrap();
        sender2.send(40).await.unwrap();
    });

    // Assert
    let result: Vec<i32> = result_stream.collect().await;
    assert_eq!(result.last(), Some(&106));
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
    let (animal_sender, animal_receiver) = mpsc::channel(BUFFER_SIZE);
    let (person_sender, person_receiver) = mpsc::channel(BUFFER_SIZE);
    let (plant_sender, plant_receiver) = mpsc::channel(BUFFER_SIZE);

    let animal_stream = ReceiverStream::new(animal_receiver);
    let person_stream = ReceiverStream::new(person_receiver);
    let plant_stream = ReceiverStream::new(plant_receiver);

    let merged_stream = MergedStream::seed(Repository::new())
        .merge_with(animal_stream, |animal, state| state.from_animal(animal))
        .merge_with(person_stream, |person, state| state.from_person(person))
        .merge_with(plant_stream, |plant, state| state.from_plant(plant));

    let mut merged_stream = Box::pin(merged_stream);

    // Act
    animal_sender
        .send(Animal::new("Dog".to_string(), 4))
        .await
        .unwrap();

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
    animal_sender
        .send(Animal::new("Bird".to_string(), 2))
        .await
        .unwrap();

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
    person_sender
        .send(Person::new("Alice".to_string(), 30))
        .await
        .unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Alice".to_string(), 30)),
            last_plant: None,
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Alice".to_string()),
            person_age: Some(30),
            plant_species: None,
            plant_height: None,
        }
    );

    // Act
    person_sender
        .send(Person::new("Bob".to_string(), 25))
        .await
        .unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Bob".to_string(), 25)),
            last_plant: None,
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Bob".to_string()),
            person_age: Some(25),
            plant_species: None,
            plant_height: None,
        }
    );

    // Act
    plant_sender
        .send(Plant::new("Fern".to_string(), 150))
        .await
        .unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Bob".to_string(), 25)),
            last_plant: Some(Plant::new("Fern".to_string(), 150)),
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Bob".to_string()),
            person_age: Some(25),
            plant_species: Some("Fern".to_string()),
            plant_height: Some(150),
        }
    );

    // Act
    plant_sender
        .send(Plant::new("Oak".to_string(), 1000))
        .await
        .unwrap();

    // Assert
    let state = merged_stream.next().await.unwrap();
    assert_eq!(
        state,
        Repository {
            last_animal: Some(Animal::new("Bird".to_string(), 2)),
            last_person: Some(Person::new("Bob".to_string(), 25)),
            last_plant: Some(Plant::new("Oak".to_string(), 1000)),
            animal_species: Some("Bird".to_string()),
            animal_legs: Some(2),
            person_name: Some("Bob".to_string()),
            person_age: Some(25),
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
