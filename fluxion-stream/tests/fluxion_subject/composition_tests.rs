// Composition tests for chaining FluxionStream with FluxionSubject using shared test data.

use fluxion_core::{FluxionSubject, HasTimestamp, StreamItem, Timestamped};
use fluxion_stream::FluxionStream;
use fluxion_test_utils::animal::Animal;
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{
    animal_bird, animal_cat, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
    plant_fern, plant_rose, plant_sunflower, TestData,
};
use fluxion_test_utils::{assert_no_element_emitted, test_channel, unwrap_stream, Sequenced};

fn person_age(data: &TestData) -> u32 {
    match data {
        TestData::Person(person) => person.age,
        other => panic!("expected person, got {other:?}"),
    }
}

fn animal_legs(data: &TestData) -> u32 {
    match data {
        TestData::Animal(animal) => animal.legs,
        other => panic!("expected animal, got {other:?}"),
    }
}

#[tokio::test]
async fn subject_at_start_map_and_filter() -> anyhow::Result<()> {
    // Arrange
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let (other_tx, other_rx) = test_channel::<Sequenced<TestData>>();

    let mut stream = FluxionStream::new(subject.subscribe())
        .combine_latest(vec![other_rx], |_| true)
        .map_ordered(|combined| {
            let person = combined.values()[0].clone();
            let updated = match person {
                TestData::Person(p) => TestData::Person(Person::new(p.name, p.age + 5)),
                other => other,
            };
            Sequenced::new(updated)
        })
        .filter_ordered(|data| matches!(data, TestData::Person(person) if person.age > 30));

    // Act
    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_bob())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    other_tx.send(Sequenced::new(person_alice()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Charlie" && person.age == 40
    ));

    // Assert
    other_tx.send(Sequenced::new(person_bob()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Charlie" && person.age == 40
    ));

    other_tx.send(Sequenced::new(person_charlie()))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Person(ref person) if person.name == "Charlie" && person.age == 40
    ));

    assert_no_element_emitted(&mut stream, 100).await;
    Ok(())
}

#[tokio::test]
async fn subject_combined_with_stream_via_combine_latest() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let gate_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut combined =
        FluxionStream::new(primary_rx).combine_latest(vec![gate_subject.subscribe()], |_| true);

    // Act
    gate_subject.send(StreamItem::Value(Sequenced::new(plant_sunflower())))?;
    primary_tx.send(Sequenced::new(person_alice()))?; // first combined output

    // Assert first emission
    let first = unwrap_stream(&mut combined, 200)
        .await
        .unwrap()
        .into_inner();
    let vals = first.values();
    assert!(
        matches!(vals[0], TestData::Person(ref p) if p.name == "Alice")
            && matches!(vals[1], TestData::Plant(ref plant) if plant.species == "Sunflower")
    );

    // Assert second emission
    primary_tx.send(Sequenced::new(person_bob()))?; // second combined output
    let second = unwrap_stream(&mut combined, 200)
        .await
        .unwrap()
        .into_inner();
    let vals = second.values();
    assert!(
        matches!(vals[0], TestData::Person(ref p) if p.name == "Bob")
            && matches!(vals[1], TestData::Plant(ref plant) if plant.species == "Sunflower")
    );
    Ok(())
}

#[tokio::test]
async fn subject_in_middle_take_latest_when() -> anyhow::Result<()> {
    // Arrange
    let (src_tx, src_rx) = test_channel::<Sequenced<TestData>>();
    let gate_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let source = FluxionStream::new(src_rx);
    let gate = FluxionStream::new(gate_subject.subscribe());

    let mut stream = source.take_latest_when(
        gate,
        |signal| matches!(signal, TestData::Animal(animal) if animal.legs >= 4),
    );

    // Act
    src_tx.send(Sequenced::new(person_alice()))?;
    gate_subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Alice")
    );

    // Act
    src_tx.send(Sequenced::new(person_bob()))?;
    gate_subject.send(StreamItem::Value(Sequenced::new(animal_bird())))?;
    assert_no_element_emitted(&mut stream, 100).await;

    gate_subject.send(StreamItem::Value(Sequenced::new(animal_dog())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Bob")
    );
    Ok(())
}

#[tokio::test]
async fn subject_combines_with_latest_from() -> anyhow::Result<()> {
    // Arrange
    let (primary_tx, primary_rx) = test_channel::<Sequenced<TestData>>();
    let secondary_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let primary = FluxionStream::new(primary_rx);
    let secondary = FluxionStream::new(secondary_subject.subscribe());

    let mut stream = primary.with_latest_from(secondary, |state| {
        let values = state.values();
        let age = person_age(&values[0]);
        let legs = animal_legs(&values[1]);
        Sequenced::new(age * legs)
    });

    // Act
    secondary_subject.send(StreamItem::Value(Sequenced::new(animal_cat())))?;
    primary_tx.send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        140
    );
    Ok(())
}

#[tokio::test]
async fn subject_as_filter_for_emit_when() -> anyhow::Result<()> {
    // Arrange
    let (src_tx, src_rx) = test_channel::<Sequenced<TestData>>();
    let filter_subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut stream = FluxionStream::new(src_rx).emit_when(
        filter_subject.subscribe(),
        |state| matches!(state.values()[1], TestData::Plant(ref plant) if plant.height >= 100),
    );

    // Act
    src_tx.send(Sequenced::new(person_alice()))?;
    filter_subject.send(StreamItem::Value(Sequenced::new(plant_rose())))?;
    assert_no_element_emitted(&mut stream, 100).await;

    filter_subject.send(StreamItem::Value(Sequenced::new(plant_sunflower())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Alice")
    );

    // Act
    src_tx.send(Sequenced::new(person_bob()))?;
    filter_subject.send(StreamItem::Value(Sequenced::new(plant_fern())))?;

    // Assert
    assert!(
        matches!(unwrap_stream(&mut stream, 200).await.unwrap().into_inner(), TestData::Person(ref person) if person.name == "Bob")
    );
    Ok(())
}

#[tokio::test]
async fn subject_chain_with_filter_and_map() -> anyhow::Result<()> {
    // Arrange
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();
    let mut stream = FluxionStream::new(subject.subscribe())
        .filter_ordered(|data| {
            matches!(
                data,
                TestData::Animal(animal) if animal.legs % 2 == 0 && animal.legs >= 4
            )
        })
        .map_ordered(|item| {
            let ts = item.timestamp();
            let mapped = match item.into_inner() {
                TestData::Animal(animal) => {
                    TestData::Animal(Animal::new(animal.species, animal.legs * 2))
                }
                other => other,
            };
            Sequenced::with_timestamp(mapped, ts)
        });

    // Act
    subject.send(StreamItem::Value(Sequenced::new(animal_spider())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_bird())))?;
    subject.send(StreamItem::Value(Sequenced::new(animal_cat())))?;

    // Assert
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Animal(ref animal) if animal.species == "Spider" && animal.legs == 16
    ));

    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Animal(ref animal) if animal.species == "Cat" && animal.legs == 8
    ));
    Ok(())
}
