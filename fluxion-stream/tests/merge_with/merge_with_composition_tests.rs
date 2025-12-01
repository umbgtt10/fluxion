// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::Timestamped;
use fluxion_stream::{FluxionStream, MergedStream};
use fluxion_test_utils::{person::Person, test_channel, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_merge_with_chaining_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx1, stream1) = test_channel::<Sequenced<Person>>();
    let (tx2, stream2) = test_channel::<Sequenced<Person>>();

    // Act: Chain map_ordered before merge_with
    // We map each person to have age 2, then sum ages in merge_with
    let mapped_stream1 = FluxionStream::new(stream1).map_ordered(|seq| {
        let mut p = seq.into_inner();
        p.age = 2;
        Sequenced::new(p)
    });

    let mapped_stream2 = FluxionStream::new(stream2).map_ordered(|seq| {
        let mut p = seq.into_inner();
        p.age = 3;
        Sequenced::new(p)
    });

    let mut result = MergedStream::seed::<Sequenced<Person>>(Person::new("State".to_string(), 0))
        .merge_with(mapped_stream1, |item: Person, state: &mut Person| {
            state.age += item.age;
            state.clone()
        })
        .merge_with(mapped_stream2, |item: Person, state: &mut Person| {
            state.age += item.age;
            state.clone()
        });

    // Act & Assert
    tx1.send(Sequenced::new(Person::new("Alice".to_string(), 10)))?;
    let res = unwrap_stream(&mut result, 500).await.into_inner();
    assert_eq!(res.age, 2, "First emission: 0+2 = 2");

    tx2.send(Sequenced::new(Person::new("Bob".to_string(), 20)))?;
    let res = unwrap_stream(&mut result, 500).await.into_inner();
    assert_eq!(res.age, 5, "Second emission: 2+3 = 5");

    Ok(())
}

#[tokio::test]
async fn test_merge_with_chaining_multiple_operators_map_ordered() -> anyhow::Result<()> {
    // Arrange
    let (tx, stream) = test_channel::<Sequenced<Person>>();

    // Act: Chain map, filter, and map before merge_with
    // 1. Map to set age = name length
    // 2. Filter age > 3
    // 3. Map age + 10
    // 4. Merge (Sum ages)
    let processed_stream = FluxionStream::new(stream)
        .map_ordered(|seq| {
            let mut p = seq.into_inner();
            p.age = p.name.len() as u32;
            Sequenced::new(p)
        })
        .filter_ordered(|p| p.age > 3)
        .map_ordered(|seq| {
            let mut p = seq.into_inner();
            p.age += 10;
            Sequenced::new(p)
        });

    let mut result = MergedStream::seed::<Sequenced<Person>>(Person::new("State".to_string(), 0))
        .merge_with(processed_stream, |item: Person, state: &mut Person| {
            state.age += item.age;
            state.clone()
        });

    // Act & Assert
    // Alice: len 5. 5 > 3. 5+10=15. State 0+15=15.
    tx.send(Sequenced::new(Person::new("Alice".to_string(), 0)))?;
    let res = unwrap_stream(&mut result, 500).await.into_inner();
    assert_eq!(res.age, 15, "Alice: 5 > 3, 5+10=15, State=15");

    // Bob: len 3. 3 > 3 is False. Filtered.
    tx.send(Sequenced::new(Person::new("Bob".to_string(), 0)))?;

    // Charlie: len 7. 7 > 3. 7+10=17. State 15+17=32.
    tx.send(Sequenced::new(Person::new("Charlie".to_string(), 0)))?;
    let res = unwrap_stream(&mut result, 500).await.into_inner();
    assert_eq!(res.age, 32, "Charlie: 7 > 3, 7+10=17, State=15+17=32");

    // Dave: len 4. 4 > 3. 4+10=14. State 32+14=46.
    tx.send(Sequenced::new(Person::new("Dave".to_string(), 0)))?;
    let res = unwrap_stream(&mut result, 500).await.into_inner();
    assert_eq!(res.age, 46, "Dave: 4 > 3, 4+10=14, State=32+14=46");

    Ok(())
}
