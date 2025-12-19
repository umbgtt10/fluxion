// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{HasTimestamp, Timestamped};
use fluxion_stream_time::runtimes::TokioTimer;
use fluxion_stream_time::timer::Timer;
use fluxion_stream_time::InstantTimestamped;
use std::cmp::Ordering;
use std::f64::consts::PI;

#[test]
fn test_instant_timestamped_new() {
    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant);

    assert_eq!(item.value, 42);
    assert_eq!(item.timestamp, instant);
}

#[test]
fn test_instant_timestamped_has_timestamp() {
    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant);

    assert_eq!(item.timestamp(), instant);
}

#[test]
fn test_instant_timestamped_into_inner() {
    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<String, TokioTimer> =
        InstantTimestamped::new("test".to_string(), instant);

    let inner = item.into_inner();
    assert_eq!(inner, "test");
}

#[test]
fn test_instant_timestamped_with_timestamp() {
    let timer = TokioTimer;
    let instant1 = timer.now();
    let instant2 = timer.now();

    let item: InstantTimestamped<i32, TokioTimer> =
        InstantTimestamped::with_timestamp(42, instant1);

    assert_eq!(item.value, 42);
    assert_eq!(item.timestamp, instant1);

    // Test replacing timestamp
    let item2: InstantTimestamped<i32, TokioTimer> =
        InstantTimestamped::with_timestamp(100, instant2);
    assert_eq!(item2.value, 100);
    assert_eq!(item2.timestamp, instant2);
}

#[test]
fn test_instant_timestamped_clone() {
    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant);

    let cloned = item.clone();
    assert_eq!(cloned.value, item.value);
    assert_eq!(cloned.timestamp, item.timestamp);
}

#[test]
fn test_instant_timestamped_debug() {
    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant);

    let debug_str = format!("{:?}", item);
    assert!(debug_str.contains("InstantTimestamped"));
    assert!(debug_str.contains("value"));
    assert!(debug_str.contains("42"));
}

#[test]
fn test_instant_timestamped_equality() {
    let timer = TokioTimer;
    let instant1 = timer.now();
    let instant2 = timer.now();

    let item1: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant1);
    let item2: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant1);
    let item3: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant2);
    let item4: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(100, instant1);

    // Same value and timestamp
    assert_eq!(item1, item2);

    // Different timestamp
    assert_ne!(item1, item3);

    // Different value
    assert_ne!(item1, item4);
}

#[test]
fn test_instant_timestamped_ordering() {
    let timer = TokioTimer;
    let instant1 = timer.now();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let instant2 = timer.now();

    let item1: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(100, instant1);
    let item2: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant2);

    // Ordering based on timestamp, not value
    assert_eq!(item1.partial_cmp(&item2), Some(Ordering::Less));
    assert_eq!(item2.partial_cmp(&item1), Some(Ordering::Greater));
    assert_eq!(item1.cmp(&item2), Ordering::Less);
    assert_eq!(item2.cmp(&item1), Ordering::Greater);

    assert!(item1 < item2);
    assert!(item2 > item1);
}

#[test]
fn test_instant_timestamped_ordering_same_timestamp() {
    let timer = TokioTimer;
    let instant = timer.now();

    let item1: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(42, instant);
    let item2: InstantTimestamped<i32, TokioTimer> = InstantTimestamped::new(100, instant);

    // Same timestamp = equal ordering
    assert_eq!(item1.partial_cmp(&item2), Some(Ordering::Equal));
    assert_eq!(item1.cmp(&item2), Ordering::Equal);
}

#[test]
fn test_instant_timestamped_deref() {
    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<String, TokioTimer> =
        InstantTimestamped::new("hello".to_string(), instant);

    // Test Deref - can call String methods directly
    assert_eq!(item.len(), 5);
    assert_eq!(item.as_str(), "hello");
    assert!(item.starts_with("hel"));
}

#[test]
fn test_instant_timestamped_deref_with_complex_type() {
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct Person {
        name: String,
        age: u32,
    }

    impl Person {
        fn is_adult(&self) -> bool {
            self.age >= 18
        }
    }

    let timer = TokioTimer;
    let instant = timer.now();
    let person = Person {
        name: "Alice".to_string(),
        age: 25,
    };

    let item: InstantTimestamped<Person, TokioTimer> = InstantTimestamped::new(person, instant);

    // Can call Person methods through Deref
    assert!(item.is_adult());
    assert_eq!(item.age, 25);
    assert_eq!(item.name, "Alice");
}

#[test]
fn test_instant_timestamped_with_option() {
    let timer = TokioTimer;
    let instant = timer.now();

    let item1: InstantTimestamped<Option<i32>, TokioTimer> =
        InstantTimestamped::new(Some(42), instant);
    let item2: InstantTimestamped<Option<i32>, TokioTimer> = InstantTimestamped::new(None, instant);

    assert_eq!(*item1, Some(42));
    assert_eq!(*item2, None);
}

#[test]
fn test_instant_timestamped_trait_bounds() {
    // Test that InstantTimestamped works with types that have different trait bounds

    // Type with all traits
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    struct FullTraits(i32);

    let timer = TokioTimer;
    let instant = timer.now();
    let item: InstantTimestamped<FullTraits, TokioTimer> =
        InstantTimestamped::new(FullTraits(42), instant);

    assert_eq!(item.0, 42);

    // Type without Ord (should still work for basic operations)
    #[derive(Debug, Clone, PartialEq)]
    struct PartialOnly(f64);

    let item2: InstantTimestamped<PartialOnly, TokioTimer> =
        InstantTimestamped::new(PartialOnly(PI), instant);

    assert_eq!(item2.0, PI);
}

#[test]
fn test_instant_timestamped_vector_sorting() {
    let timer = TokioTimer;
    let instant1 = timer.now();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let instant2 = timer.now();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let instant3 = timer.now();

    let mut items = [
        InstantTimestamped::<i32, TokioTimer>::new(3, instant3),
        InstantTimestamped::<i32, TokioTimer>::new(1, instant1),
        InstantTimestamped::<i32, TokioTimer>::new(2, instant2),
    ];

    items.sort();

    assert_eq!(items[0].value, 1);
    assert_eq!(items[1].value, 2);
    assert_eq!(items[2].value, 3);
}
