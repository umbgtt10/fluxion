// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::lock_utilities::lock_or_recover;
use std::sync::{Arc, Mutex};

#[test]
fn test_lock_or_recover_success() {
    let mutex = Arc::new(Mutex::new(42));
    let guard = lock_or_recover(&mutex, "test");
    assert_eq!(*guard, 42);
    drop(guard);
}

#[test]
fn test_lock_or_recover_normal_operation() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));

    {
        let guard = lock_or_recover(&mutex, "normal operation");
        assert_eq!(*guard, vec![1, 2, 3]);
        drop(guard);
    }

    let mut guard = lock_or_recover(&mutex, "second lock");
    guard.push(4);
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}

#[test]
fn test_lock_or_recover_recovers_from_poison() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    // Poison the mutex by panicking while holding the lock
    let _ = std::panic::catch_unwind(|| {
        // inline the single-use lock to avoid holding the guard across the closure
        mutex_clone.lock().unwrap().push(4);
        panic!("Intentional panic to poison mutex");
    });

    // lock_or_recover should recover and return the data (with the mutation from before panic)
    let guard = lock_or_recover(&mutex, "poisoned test");
    assert_eq!(*guard, vec![1, 2, 3, 4]); // Data includes the push(4) before panic
    drop(guard);
}

#[test]
fn test_lock_or_recover_with_string_data() {
    let mutex = Arc::new(Mutex::new(String::from("hello")));

    {
        let mut guard = lock_or_recover(&mutex, "string operation");
        guard.push_str(" world");
        assert_eq!(*guard, "hello world");
        drop(guard);
    }
}

#[test]
fn test_lock_or_recover_with_complex_type() {
    #[derive(Debug, PartialEq, Clone)]
    struct Data {
        id: u32,
        name: String,
    }

    let mutex = Arc::new(Mutex::new(Data {
        id: 1,
        name: "test".to_string(),
    }));

    {
        let mut guard = lock_or_recover(&mutex, "complex type");
        guard.id = 2;
        guard.name = "updated".to_string();
        drop(guard);
    }

    let guard = lock_or_recover(&mutex, "verify update");
    assert_eq!(guard.id, 2);
    assert_eq!(guard.name, "updated");
    drop(guard);
}

#[tokio::test]
async fn test_lock_or_recover_in_async_context() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    let handle = tokio::spawn(async move {
        let mut guard = lock_or_recover(&mutex_clone, "async task");
        guard.push(4);
        guard.len()
    });

    let len = handle.await.unwrap();
    assert_eq!(len, 4);

    let guard = lock_or_recover(&mutex, "main task");
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}

#[test]
fn test_lock_or_recover_with_poisoned_mutex() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    // Poison the mutex
    let _ = std::panic::catch_unwind(|| {
        mutex_clone.lock().unwrap().push(4);
        panic!("Intentional panic to poison mutex");
    });

    // lock_or_recover should recover from poisoned mutex
    let guard = lock_or_recover(&mutex, "poisoned recovery");
    assert_eq!(*guard, vec![1, 2, 3, 4]); // Recovered data includes mutation before panic
    drop(guard);
}

#[test]
fn test_multiple_recoveries_from_same_poison() {
    let mutex = Arc::new(Mutex::new(100));
    let mutex_clone = Arc::clone(&mutex);

    // Poison the mutex
    let _ = std::panic::catch_unwind(|| {
        *mutex_clone.lock().unwrap() = 200;
        panic!("Intentional panic");
    });

    // All attempts should successfully recover
    let guard1 = lock_or_recover(&mutex, "first recovery");
    assert_eq!(*guard1, 200); // Data from before panic
    drop(guard1);

    let guard2 = lock_or_recover(&mutex, "second recovery");
    assert_eq!(*guard2, 200);
    drop(guard2);

    let guard3 = lock_or_recover(&mutex, "third recovery");
    assert_eq!(*guard3, 200);
    drop(guard3);

    // Can still modify after recovery
    let mut guard4 = lock_or_recover(&mutex, "modify after recovery");
    *guard4 = 300;
    assert_eq!(*guard4, 300);
    drop(guard4);

    // Verify modification persisted
    let guard5 = lock_or_recover(&mutex, "verify mutation");
    assert_eq!(*guard5, 300);
    drop(guard5);
}

#[test]
fn test_concurrent_poison_recovery() {
    use std::thread;

    let mutex = Arc::new(Mutex::new(0));
    let mutex_clone1 = Arc::clone(&mutex);
    let mutex_clone2 = Arc::clone(&mutex);
    let mutex_clone3 = Arc::clone(&mutex);

    // Poison the mutex in a thread
    let poisoner = thread::spawn(move || {
        let _ = std::panic::catch_unwind(|| {
            *mutex_clone1.lock().unwrap() = 42;
            panic!("Poisoning the mutex");
        });
    });

    poisoner.join().ok();

    // Multiple threads should all successfully recover from poisoned mutex
    let handle1 = thread::spawn(move || {
        let guard = lock_or_recover(&mutex_clone2, "thread 1");
        *guard
    });

    let handle2 = thread::spawn(move || {
        let guard = lock_or_recover(&mutex_clone3, "thread 2");
        *guard
    });

    let value1 = handle1.join().unwrap();
    let value2 = handle2.join().unwrap();

    // Both should get the poisoned data (42)
    assert_eq!(value1, 42);
    assert_eq!(value2, 42);

    // Main thread should also recover successfully
    let guard = lock_or_recover(&mutex, "main thread");
    assert_eq!(*guard, 42);
    drop(guard);
}

#[test]
fn test_poison_with_partial_mutation() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    // Poison after partial mutation
    let _ = std::panic::catch_unwind(|| {
        let mut guard = mutex_clone.lock().unwrap();
        guard.push(4);
        guard.push(5);
        // Panic before releasing lock
        panic!("Panic with partial state");
    });

    // lock_or_recover should recover and include partial mutations
    let guard = lock_or_recover(&mutex, "recover partial");
    assert_eq!(*guard, vec![1, 2, 3, 4, 5]); // All mutations before panic are preserved
    drop(guard);
}

#[tokio::test]
async fn test_async_concurrent_poison_recovery() {
    let mutex = Arc::new(Mutex::new(String::from("initial")));
    let mutex_clone1 = Arc::clone(&mutex);
    let mutex_clone2 = Arc::clone(&mutex);
    let mutex_clone3 = Arc::clone(&mutex);

    // Poison in a spawned task
    let _ = tokio::spawn(async move {
        let _ = std::panic::catch_unwind(|| {
            let mut guard = mutex_clone1.lock().unwrap();
            guard.push_str("_poisoned");
            panic!("Async poison");
        });
    })
    .await;

    // Multiple async tasks should recover from poisoned mutex
    let handle1 = tokio::spawn(async move {
        let guard = lock_or_recover(&mutex_clone2, "async task 1");
        guard.clone()
    });

    let handle2 = tokio::spawn(async move {
        let guard = lock_or_recover(&mutex_clone3, "async task 2");
        guard.clone()
    });

    let value1 = handle1.await.unwrap();
    let value2 = handle2.await.unwrap();

    assert_eq!(value1, "initial_poisoned");
    assert_eq!(value2, "initial_poisoned");

    // Main async context should also recover
    let guard = lock_or_recover(&mutex, "main async");
    assert_eq!(*guard, "initial_poisoned");
    drop(guard);
}

#[test]
fn test_repeated_poison_and_recovery_cycle() {
    let mutex = Arc::new(Mutex::new(0));

    // Poison the mutex once
    let mutex_clone = Arc::clone(&mutex);
    let _ = std::panic::catch_unwind(move || {
        *mutex_clone.lock().unwrap() = 10;
        panic!("Poison iteration");
    });

    // All subsequent attempts should successfully recover
    for i in 1..=5 {
        let guard = lock_or_recover(&mutex, &format!("attempt {}", i));
        assert_eq!(*guard, 10); // All recoveries get the poisoned data
        drop(guard);
    }
}
