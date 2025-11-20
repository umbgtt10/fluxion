// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::lock_utilities::{lock_or_error, try_lock};
use std::sync::{Arc, Mutex};

#[test]
fn test_safe_lock_success() {
    let mutex = Arc::new(Mutex::new(42));
    let guard = lock_or_error(&mutex, "test").unwrap();
    assert_eq!(*guard, 42);
    drop(guard);
}

#[test]
fn test_safe_lock_normal_operation() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));

    {
        let guard = lock_or_error(&mutex, "normal operation").unwrap();
        assert_eq!(*guard, vec![1, 2, 3]);
        drop(guard);
    }

    let mut guard = lock_or_error(&mutex, "second lock").unwrap();
    guard.push(4);
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}

#[test]
fn test_safe_lock_recovers_from_poison() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    // Poison the mutex by panicking while holding the lock
    let _ = std::panic::catch_unwind(|| {
        // inline the single-use lock to avoid holding the guard across the closure
        mutex_clone.lock().unwrap().push(4);
        panic!("Intentional panic to poison mutex");
    });

    // lock_or_error should recover from the poison
    let guard = lock_or_error(&mutex, "poisoned test").unwrap();

    // The data should still be accessible (the 4 was added before panic)
    assert_eq!(guard.len(), 4);
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}

#[test]
fn test_safe_lock_with_string_data() {
    let mutex = Arc::new(Mutex::new(String::from("hello")));

    {
        let mut guard = lock_or_error(&mutex, "string operation").unwrap();
        guard.push_str(" world");
        assert_eq!(*guard, "hello world");
        drop(guard);
    }
}

#[test]
fn test_safe_lock_with_complex_type() {
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
        let mut guard = lock_or_error(&mutex, "complex type").unwrap();
        guard.id = 2;
        guard.name = "updated".to_string();
        drop(guard);
    }

    let guard = lock_or_error(&mutex, "verify update").unwrap();
    assert_eq!(guard.id, 2);
    assert_eq!(guard.name, "updated");
    drop(guard);
}

#[test]
fn test_try_lock() {
    let mutex = Arc::new(Mutex::new("test data"));
    let guard = try_lock(&mutex, "reading test data").unwrap();
    assert_eq!(*guard, "test data");
    drop(guard);
}

#[test]
fn test_try_lock_alias() {
    let mutex = Arc::new(Mutex::new(42));
    let guard = try_lock(&mutex, "testing try_lock").unwrap();
    assert_eq!(*guard, 42);
    drop(guard);
}

#[tokio::test]
async fn test_safe_lock_in_async_context() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    let handle = tokio::spawn(async move {
        let mut guard = lock_or_error(&mutex_clone, "async task").unwrap();
        guard.push(4);
        guard.len()
    });

    let len = handle.await.unwrap();
    assert_eq!(len, 4);

    let guard = lock_or_error(&mutex, "main task").unwrap();
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}

#[test]
fn test_try_lock_with_poisoned_mutex() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = Arc::clone(&mutex);

    // Poison the mutex
    let _ = std::panic::catch_unwind(|| {
        mutex_clone.lock().unwrap().push(4);
        panic!("Intentional panic to poison mutex");
    });

    // try_lock should recover from poison
    let guard = try_lock(&mutex, "poisoned try_lock").unwrap();
    assert_eq!(guard.len(), 4);
    assert_eq!(*guard, vec![1, 2, 3, 4]);
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

    // First recovery
    let guard1 = lock_or_error(&mutex, "first recovery").unwrap();
    assert_eq!(*guard1, 200);
    drop(guard1);

    // Second recovery - should still work
    let guard2 = lock_or_error(&mutex, "second recovery").unwrap();
    assert_eq!(*guard2, 200);
    drop(guard2);

    // Third recovery with mutation
    let mut guard3 = lock_or_error(&mutex, "third recovery").unwrap();
    *guard3 = 300;
    drop(guard3);

    // Verify the mutation persisted
    let guard4 = lock_or_error(&mutex, "verify mutation").unwrap();
    assert_eq!(*guard4, 300);
    drop(guard4);
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

    // Multiple threads should all be able to recover
    let handle1 = thread::spawn(move || {
        let guard = lock_or_error(&mutex_clone2, "thread 1").unwrap();
        *guard
    });

    let handle2 = thread::spawn(move || {
        let guard = lock_or_error(&mutex_clone3, "thread 2").unwrap();
        *guard
    });

    let value1 = handle1.join().unwrap();
    let value2 = handle2.join().unwrap();

    // Both should have recovered the same poisoned data
    assert_eq!(value1, 42);
    assert_eq!(value2, 42);

    // Main thread should also recover
    let guard = lock_or_error(&mutex, "main thread").unwrap();
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

    // Recovery should preserve the partial mutation
    let guard = lock_or_error(&mutex, "recover partial").unwrap();
    assert_eq!(*guard, vec![1, 2, 3, 4, 5]);
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

    // Multiple async tasks should recover
    let handle1 = tokio::spawn(async move {
        let guard = lock_or_error(&mutex_clone2, "async task 1").unwrap();
        guard.clone()
    });

    let handle2 = tokio::spawn(async move {
        let guard = lock_or_error(&mutex_clone3, "async task 2").unwrap();
        guard.clone()
    });

    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();

    assert_eq!(result1, "initial_poisoned");
    assert_eq!(result2, "initial_poisoned");

    // Main async context should also recover
    let guard = lock_or_error(&mutex, "main async").unwrap();
    assert_eq!(*guard, "initial_poisoned");
    drop(guard);
}

#[test]
fn test_repeated_poison_and_recovery_cycle() {
    let mutex = Arc::new(Mutex::new(0));

    for i in 1..=5 {
        let mutex_clone = Arc::clone(&mutex);

        // Poison the mutex
        let _ = std::panic::catch_unwind(move || {
            *mutex_clone.lock().unwrap() = i * 10;
            panic!("Poison iteration {}", i);
        });

        // Recover and verify
        let guard = lock_or_error(&mutex, &format!("recovery {}", i)).unwrap();
        assert_eq!(*guard, i * 10);
        drop(guard);

        // Successful operation after recovery
        let mut guard = lock_or_error(&mutex, &format!("update {}", i)).unwrap();
        *guard += 1;
        drop(guard);

        // Verify update
        let guard = lock_or_error(&mutex, &format!("verify {}", i)).unwrap();
        assert_eq!(*guard, i * 10 + 1);
        drop(guard);
    }
}
