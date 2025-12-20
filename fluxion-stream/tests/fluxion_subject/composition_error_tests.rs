// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, FluxionSubject, HasTimestamp, StreamItem};
use fluxion_stream::{
    CombineLatestExt, CombineWithPreviousExt, FilterOrderedExt, MapOrderedExt, OnErrorExt,
    OrderedStreamExt, TakeLatestWhenExt,
};
use fluxion_test_utils::person::Person;
use fluxion_test_utils::test_data::{
    animal_dog, animal_spider, person_alice, person_bob, person_charlie, plant_rose, TestData,
};
use fluxion_test_utils::{test_channel, test_channel_with_errors, unwrap_stream, Sequenced};

#[tokio::test]
async fn subject_at_start_complex_chain_propagates_error() -> anyhow::Result<()> {
    // subject feeds a map -> filter -> combine_with_previous -> map chain
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut stream = subject
        .subscribe()
        .unwrap()
        .map_ordered(|item| {
            let ts = item.timestamp();
            let mapped = match item.into_inner() {
                TestData::Person(person) => {
                    TestData::Person(Person::new(person.name, person.age + 1))
                }
                other => other,
            };
            Sequenced::with_timestamp(mapped, ts)
        })
        .filter_ordered(|data| matches!(data, TestData::Person(_)))
        .combine_with_previous()
        .map_ordered(|with_prev| {
            let ts = with_prev.current.timestamp();
            let current_age = match with_prev.current.into_inner() {
                TestData::Person(p) => p.age,
                _ => unreachable!(),
            };
            let prev_age = with_prev
                .previous
                .map(|p| match p.into_inner() {
                    TestData::Person(pp) => pp.age,
                    _ => 0,
                })
                .unwrap_or(0);
            Sequenced::with_timestamp(current_age + prev_age, ts)
        });

    subject.send(StreamItem::Value(Sequenced::new(person_charlie())))?;
    assert_eq!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        36
    );

    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert_eq!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        62
    );

    subject.send(StreamItem::Error(FluxionError::stream_error("chain fail")))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context } ) if context == "chain fail"
    ));
    Ok(())
}

#[tokio::test]
async fn subject_in_middle_gate_error_terminates_stream() -> anyhow::Result<()> {
    // source -> take_latest_when(gate subject) -> assertions
    let (tx, rx) = test_channel::<Sequenced<TestData>>();
    let gate: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut stream = rx.take_latest_when(
        gate.subscribe().unwrap(),
        |data| matches!(data, TestData::Animal(animal) if animal.legs >= 4),
    );

    tx.unbounded_send(Sequenced::new(animal_dog()))?;
    gate.send(StreamItem::Value(Sequenced::new(animal_spider())))?; // initializes gate
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await.unwrap().into_inner(),
        TestData::Animal(ref a) if a.species == "Dog"
    ));

    gate.send(StreamItem::Error(FluxionError::stream_error("gate boom")))?;
    assert!(matches!(
        unwrap_stream(&mut stream, 200).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "gate boom"
    ));
    Ok(())
}

#[tokio::test]
async fn subject_at_end_forwarding_chain_propagates_error() -> anyhow::Result<()> {
    // simple combine_latest: one stream + one subject; subject injects error after first successful pair
    let (tx, rx) = test_channel_with_errors::<Sequenced<TestData>>();
    let subject: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut combined = rx.combine_latest(vec![subject.subscribe().unwrap()], |_| true);

    // First, provide both sides so combine_latest can emit a value
    tx.unbounded_send(StreamItem::Value(Sequenced::new(plant_rose())))?;
    subject.send(StreamItem::Value(Sequenced::new(person_alice())))?;
    assert!(matches!(
        unwrap_stream(&mut combined, 200).await,
        StreamItem::Value(ref combined_state) if {
            let values = combined_state.values();
            values.len() == 2 &&
                matches!(values[0], TestData::Plant(ref p) if p.species == "Rose") &&
                matches!(values[1], TestData::Person(ref p) if p.name == "Alice")
        }
    ));

    // Then inject an error from the subject side
    subject.send(StreamItem::Error(FluxionError::stream_error("sink fail")))?;
    assert!(matches!(
        unwrap_stream(&mut combined, 200).await,
        StreamItem::Error(FluxionError::StreamProcessingError { context }) if context == "sink fail"
    ));

    Ok(())
}

#[tokio::test]
async fn subject_on_error_with_ordered_merge_skips_transient() -> anyhow::Result<()> {
    let (tx, err_stream) = test_channel_with_errors::<Sequenced<TestData>>();
    let gate: FluxionSubject<Sequenced<TestData>> = FluxionSubject::new();

    let mut merged = err_stream
        .on_error(|err| err.to_string().contains("transient"))
        .ordered_merge(vec![gate.subscribe().unwrap()]);

    tx.unbounded_send(StreamItem::Error(FluxionError::stream_error(
        "transient failure",
    )))?;
    gate.send(StreamItem::Value(Sequenced::with_timestamp(
        plant_rose(),
        1,
    )))?;
    tx.unbounded_send(StreamItem::Value(Sequenced::with_timestamp(
        person_bob(),
        2,
    )))?;

    let first = unwrap_stream(&mut merged, 200).await.unwrap().into_inner();
    assert!(matches!(first, TestData::Plant(ref p) if p.species == "Rose"));

    let second = unwrap_stream(&mut merged, 200).await.unwrap().into_inner();
    assert!(matches!(second, TestData::Person(ref p) if p.name == "Bob"));
    Ok(())
}
