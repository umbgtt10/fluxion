// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::take_latest_when::TakeLatestWhenExt;
use fluxion_test_utils::{
    helpers::{assert_no_element_emitted, assert_stream_ended, test_channel, unwrap_stream},
    sequenced::Sequenced,
    test_data::{
        animal, animal_ant, animal_cat, animal_dog, person, person_alice, person_bob,
        person_charlie, person_dave, TestData,
    },
};

#[tokio::test]
async fn test_take_latest_when_empty_streams() -> anyhow::Result<()> {
    // Arrange
    let filter_fn = |_: &TestData| -> bool { true };

    // Act
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
    drop(source_tx);
    drop(filter_tx);

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Assert
    assert_stream_ended(&mut result, 500).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_not_satisfied_does_not_emit() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_satisfied_emits() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => {
                panic!(
                    "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!"
                );
            }
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_satisfied() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!",
            ),
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    let first_item = unwrap_stream(&mut result, 500).await.unwrap();
    assert_eq!(first_item.value, person_alice());

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_emissions_filter_not_satisfied() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!",
            ),
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_charlie()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    source_tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_toggle_emissions() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => panic!(
                "Expected the filter stream to emit an Animal value. But it emitted: {filter_val:?} instead!",
            ),
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_alice()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_bob()
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_charlie()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_filter_stream_closes_no_further_emits() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_alice()
    );

    // Act
    drop(filter_tx);
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_source_publishes_before_filter() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_alice()
    );

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_bob()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_multiple_source_updates_while_filter_false() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_bob()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_charlie()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_dave()))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_dave()
    );
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_buffer_does_not_grow_unbounded() -> anyhow::Result<()> {
    // Arrange
    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let filter_fn = |filter_val: &TestData| -> bool {
        match filter_val {
            TestData::Animal(animal) => animal.legs > 5,
            _ => false,
        }
    };

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_cat()))?;
    for i in 0u32..10000u32 {
        source_tx.unbounded_send(Sequenced::new(person(format!("Person{i}"), i)))?;
    }

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person(String::from("Person9999"), 9999u32)
    );
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;
    for i in 10000u32..20000u32 {
        source_tx.unbounded_send(Sequenced::new(person(format!("Person{i}"), i)))?;
    }

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal_ant()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person(String::from("Person19999"), 19999u32)
    );
    assert_no_element_emitted(&mut result, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_boundary_empty_string_zero_values() -> anyhow::Result<()> {
    // Arrange
    let filter_fn: fn(&TestData) -> bool = |_: &TestData| true;

    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();

    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx.unbounded_send(Sequenced::new(person(String::new(), 0)))?;

    // Assert
    assert_no_element_emitted(&mut result, 100).await;

    // Act
    filter_tx.unbounded_send(Sequenced::new(animal(String::new(), 0)))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person(String::new(), 0)
    );

    // Act
    source_tx.unbounded_send(Sequenced::new(person_alice()))?;
    filter_tx.unbounded_send(Sequenced::new(animal_dog()))?;

    // Assert
    assert_eq!(
        unwrap_stream(&mut result, 500).await.unwrap().value,
        person_alice()
    );

    Ok(())
}

#[tokio::test]
async fn test_take_latest_when_boundary_maximum_concurrent_streams() -> anyhow::Result<()> {
    // Arrange
    let filter_fn: fn(&TestData) -> bool = |_: &TestData| true;

    let num_concurrent: u32 = 50;
    let mut handles = Vec::new();

    for i in 0..num_concurrent {
        let handle = tokio::spawn(async move {
            let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
            let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
            let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

            // Act
            source_tx
                .unbounded_send(Sequenced::new(person(format!("Person{i}"), i)))
                .unwrap();
            filter_tx
                .unbounded_send(Sequenced::new(animal(format!("Animal{i}"), i)))
                .unwrap();

            // Assert
            let item = unwrap_stream(&mut result, 500).await.unwrap();
            assert_eq!(&item.value, &person(format!("Person{i}"), i));

            // Act
            source_tx
                .unbounded_send(Sequenced::new(person_bob()))
                .unwrap();
            filter_tx
                .unbounded_send(Sequenced::new(animal_cat()))
                .unwrap();

            // Assert
            let result2 = unwrap_stream(&mut result, 500).await.unwrap();
            assert_eq!(&result2.value, &person_bob());
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}

#[tokio::test]
#[should_panic(expected = "Filter panicked")]
async fn test_take_latest_when_filter_panics() {
    // Arrange
    let filter_fn = |_: &TestData| -> bool {
        panic!("Filter panicked");
    };

    let (source_tx, source_stream) = test_channel::<Sequenced<TestData>>();
    let (filter_tx, filter_stream) = test_channel::<Sequenced<TestData>>();
    let mut result = source_stream.take_latest_when(filter_stream, filter_fn);

    // Act
    source_tx
        .unbounded_send(Sequenced::new(person_alice()))
        .unwrap();
    filter_tx
        .unbounded_send(Sequenced::new(animal_dog()))
        .unwrap();

    // Assert
    let _ = unwrap_stream(&mut result, 100).await;
}
