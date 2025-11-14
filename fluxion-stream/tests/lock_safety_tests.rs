// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_stream::util::safe_lock;
use fluxion_stream::util::try_lock;
use std::sync::{Arc, Mutex};

#[test]
fn test_safe_lock_normal_operation() {
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));

    {
        let guard = safe_lock(&mutex, "normal operation").unwrap();
        assert_eq!(*guard, vec![1, 2, 3]);
        drop(guard);
    }

    let mut guard = safe_lock(&mutex, "second lock").unwrap();
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

    // safe_lock should recover from the poison
    let guard = safe_lock(&mutex, "poisoned test").unwrap();

    // The data should still be accessible (the 4 was added before panic)
    assert_eq!(guard.len(), 4);
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}

#[test]
fn test_safe_lock_with_string_data() {
    let mutex = Arc::new(Mutex::new(String::from("hello")));

    {
        let mut guard = safe_lock(&mutex, "string operation").unwrap();
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
        let mut guard = safe_lock(&mutex, "complex type").unwrap();
        guard.id = 2;
        guard.name = "updated".to_string();
        drop(guard);
    }

    let guard = safe_lock(&mutex, "verify update").unwrap();
    assert_eq!(guard.id, 2);
    assert_eq!(guard.name, "updated");
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
        let mut guard = safe_lock(&mutex_clone, "async task").unwrap();
        guard.push(4);
        guard.len()
    });

    let len = handle.await.unwrap();
    assert_eq!(len, 4);

    let guard = safe_lock(&mutex, "main task").unwrap();
    assert_eq!(*guard, vec![1, 2, 3, 4]);
    drop(guard);
}
