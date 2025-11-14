// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

//! # Architecture: Clean Separation of Concerns
//!
//! This example demonstrates the clean architectural separation between:
//! - **FluxionStream**: Pure, functional stream operations (immutable)
//! - **TestChannel**: Test infrastructure with mutation (imperative)
//!
//! ## The Fundamental Problem We Solved
//!
//! We had a conflict between two ownership models:
//! 1. **Consuming operations**: Stream extensions like `combine_latest()` take `self`
//! 2. **Mutation operations**: Test helpers like `push()` need `&self`
//!
//! Trying to support both in one type was impossible. The solution is architectural
//! separation: different types for different use cases.

fn main() {
    println!("This example demonstrates architectural concepts.");
    println!("Run with: cargo test --example architecture");
}

#[cfg(test)]
mod tests {
    use fluxion_test_utils::{Sequenced, TestChannel};
    use futures::StreamExt;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Person {
        id: u32,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Animal {
        id: u32,
        species: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Plant {
        id: u32,
        name: String,
    }

    /// Example 1: Test infrastructure with push capabilities
    ///
    /// In tests, we use `TestChannel` which allows imperative setup:
    /// 1. Create channels
    /// 2. Push values imperatively
    /// 3. Convert to functional streams
    /// 4. Use consuming operations
    #[tokio::test]
    async fn example_test_with_push() {
        // Step 1: Create test channels (mutable, imperative)
        let person_channel = TestChannel::new();
        let animal_channel = TestChannel::new();

        // Step 2: Push values imperatively (test setup)
        person_channel
            .push(Person {
                id: 1,
                name: "Alice".to_string(),
            })
            .unwrap();
        person_channel
            .push(Person {
                id: 2,
                name: "Bob".to_string(),
            })
            .unwrap();

        animal_channel
            .push(Animal {
                id: 3,
                species: "Cat".to_string(),
            })
            .unwrap();

        // Step 3: Convert to functional streams (consumption)
        let mut person_stream = person_channel.into_stream();
        let mut animal_stream = animal_channel.into_stream();

        // Step 4: Now we can't push anymore, only consume
        // This is correct! Tests set up data first, then test stream operations.

        // Verify we can consume the streams
        let p1 = person_stream.next().await.unwrap();
        assert_eq!(p1.value.name, "Alice");

        let a1 = animal_stream.next().await.unwrap();
        assert_eq!(a1.value.species, "Cat");
    }

    /// Example 2: Production code pattern
    ///
    /// In production, we receive channels from other components and wrap them
    /// in FluxionStream for functional composition. No push needed.
    #[tokio::test]
    async fn example_production_pattern() {
        use fluxion::FluxionStream;
        use tokio::sync::mpsc;
        use tokio_stream::wrappers::UnboundedReceiverStream;

        // Production code receives channels from other components
        let (tx, rx) = mpsc::unbounded_channel();

        // Wrap in FluxionStream for functional composition
        let stream: FluxionStream<UnboundedReceiverStream<Sequenced<Person>>> =
            FluxionStream::from_unbounded_receiver(rx);

        // Now we have all the fluxion extension methods available
        // let result = stream.combine_latest(others, filter);
        // let filtered = stream.take_while_with(filter_stream, predicate);
        // etc.

        // Simulate producer sending data
        tx.send(Sequenced::new(Person {
            id: 1,
            name: "Charlie".to_string(),
        }))
        .unwrap();
        drop(tx);

        // Consume the stream
        let items: Vec<_> = stream.collect().await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].value.name, "Charlie");
    }

    /// Example 3: The conflict we avoided
    ///
    /// This shows why we can't have both push and consuming operations on the same type.
    #[tokio::test]
    async fn example_why_separation_is_needed() {
        let channel = TestChannel::new();

        // We can push because channel is &self
        channel
            .push(Person {
                id: 1,
                name: "Dave".to_string(),
            })
            .unwrap();

        // Converting to stream CONSUMES the channel (takes self)
        let _stream = channel.into_stream();

        // Now we can't push anymore - channel is moved!
        // This is the correct behavior:
        // - Setup phase: use TestChannel, push values
        // - Test phase: convert to stream, use functional operations

        // If we tried to make push work after into_stream(), we'd need:
        // - Either interior mutability (RefCell) - rejected as too complex
        // - Or dual channels - rejected as confusing
        // - Or separate types - ACCEPTED! This is what we implemented.
    }

    /// Example 4: Multiple channels pattern
    ///
    /// Shows creating multiple test channels individually.
    #[tokio::test]
    async fn example_multiple_channels() {
        // Create individual channels for different types
        let person: TestChannel<Person> = TestChannel::new();
        let animal: TestChannel<Animal> = TestChannel::new();
        let plant: TestChannel<Plant> = TestChannel::new();

        person
            .push(Person {
                id: 1,
                name: "Eve".to_string(),
            })
            .unwrap();
        animal
            .push(Animal {
                id: 1,
                species: "Dog".to_string(),
            })
            .unwrap();
        plant
            .push(Plant {
                id: 1,
                name: "Rose".to_string(),
            })
            .unwrap();

        // Convert all to streams
        let mut p_stream = person.into_stream();
        let mut a_stream = animal.into_stream();
        let mut pl_stream = plant.into_stream();

        // Verify
        assert!(p_stream.next().await.is_some());
        assert!(a_stream.next().await.is_some());
        assert!(pl_stream.next().await.is_some());
    }
}
