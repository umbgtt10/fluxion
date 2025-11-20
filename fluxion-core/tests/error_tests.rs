// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::{FluxionError, IntoFluxionError, Result, ResultExt};
use std::io;

#[test]
fn test_error_display() {
    let err = FluxionError::lock_error("test mutex");
    assert_eq!(err.to_string(), "Failed to acquire lock: test mutex");

    let err = FluxionError::stream_error("processing failed");
    assert_eq!(
        err.to_string(),
        "Stream processing error: processing failed"
    );
}

#[test]
fn test_error_constructors() {
    let err = FluxionError::lock_error("my lock");
    assert!(matches!(err, FluxionError::LockError { .. }));

    let err = FluxionError::stream_error("processing failed");
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
}

#[test]
fn test_is_recoverable() {
    assert!(FluxionError::lock_error("test").is_recoverable());
    assert!(!FluxionError::stream_error("test").is_recoverable());
    assert!(!FluxionError::user_error(io::Error::other("test")).is_recoverable());
}

#[test]
fn test_is_permanent() {
    assert!(FluxionError::stream_error("test").is_permanent());
    assert!(FluxionError::user_error(io::Error::other("test")).is_permanent());
    assert!(!FluxionError::lock_error("test").is_permanent());
}

#[test]
fn test_result_context() {
    // Create a result with a UserError which gets wrapped by context
    let result: Result<()> = Err(FluxionError::UserError("test error".into()));

    let err = result.context("operation failed").unwrap_err();
    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
    assert!(err.to_string().contains("operation failed"));
    assert!(err.to_string().contains("test error"));
}

#[test]
fn test_result_context_preserves_non_user_errors() {
    // Other error types are preserved, not wrapped
    let result: Result<()> = Err(FluxionError::LockError {
        context: "test lock error".to_string(),
    });

    let err = result.context("operation failed").unwrap_err();
    assert!(matches!(err, FluxionError::LockError { .. }));
}

#[test]
fn test_result_context_ok() {
    let result: Result<i32> = Ok(42);
    let value = result.context("operation failed").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_multiple_errors_creation() {
    let errors = vec![
        FluxionError::lock_error("lock1"),
        FluxionError::stream_error("stream1"),
        FluxionError::lock_error("lock2"),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    assert!(matches!(
        multi_err,
        FluxionError::MultipleErrors { count: 3, .. }
    ));
    assert!(multi_err.to_string().contains("3 errors"));
}

#[test]
fn test_from_user_errors() {
    #[derive(Debug, thiserror::Error)]
    #[error("Custom error: {msg}")]
    struct CustomError {
        msg: String,
    }

    let errors = vec![
        CustomError {
            msg: "first".to_string(),
        },
        CustomError {
            msg: "second".to_string(),
        },
        CustomError {
            msg: "third".to_string(),
        },
    ];

    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 3);
        assert_eq!(errors.len(), 3);
        assert!(errors
            .iter()
            .all(|e| matches!(e, FluxionError::UserError(_))));
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_from_user_errors_empty() {
    let errors: Vec<io::Error> = vec![];
    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 0);
        assert_eq!(errors.len(), 0);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_from_user_errors_single() {
    let errors = vec![io::Error::other("single error")];
    let result = FluxionError::from_user_errors(errors);

    if let FluxionError::MultipleErrors { count, errors } = result {
        assert_eq!(count, 1);
        assert_eq!(errors.len(), 1);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_multiple_errors_recoverable() {
    let errors = vec![
        FluxionError::lock_error("lock1"),
        FluxionError::lock_error("lock2"),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    // MultipleErrors is not considered recoverable
    assert!(!multi_err.is_recoverable());
}

#[test]
fn test_multiple_errors_permanent() {
    let errors = vec![
        FluxionError::stream_error("stream1"),
        FluxionError::user_error(io::Error::other("user1")),
    ];

    let multi_err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    // MultipleErrors is not considered permanent
    assert!(!multi_err.is_permanent());
}

#[test]
fn test_into_fluxion_error_trait() {
    let io_err = io::Error::other("test error");
    let fluxion_err = io_err.into_fluxion_error("operation failed");

    assert!(matches!(fluxion_err, FluxionError::UserError(_)));
}

#[test]
fn test_into_fluxion_method() {
    let io_err = io::Error::other("test error");
    let fluxion_err = io_err.into_fluxion();

    assert!(matches!(fluxion_err, FluxionError::UserError(_)));
    assert!(fluxion_err.to_string().contains("test error"));
}

#[test]
fn test_with_context_lazy() {
    let result: Result<()> = Err(FluxionError::UserError("test error".into()));

    let err = result
        .with_context(|| {
            // Simulate expensive context computation
            format!("computed context at {}", "runtime")
        })
        .unwrap_err();

    assert!(matches!(err, FluxionError::StreamProcessingError { .. }));
    assert!(err.to_string().contains("computed context"));
    assert!(err.to_string().contains("test error"));
}

#[test]
fn test_with_context_preserves_non_user_errors() {
    let result: Result<()> = Err(FluxionError::LockError {
        context: "lock failed".to_string(),
    });

    let err = result
        .with_context(|| "expensive context".to_string())
        .unwrap_err();
    assert!(matches!(err, FluxionError::LockError { .. }));
}

#[test]
fn test_with_context_ok() {
    let result: Result<i32> = Ok(42);
    let value = result
        .with_context(|| "should not be called".to_string())
        .unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_clone_lock_error() {
    let err = FluxionError::lock_error("test lock");
    let cloned = err.clone();

    assert!(matches!(cloned, FluxionError::LockError { .. }));
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_clone_stream_error() {
    let err = FluxionError::stream_error("test stream");
    let cloned = err.clone();

    assert!(matches!(cloned, FluxionError::StreamProcessingError { .. }));
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_clone_user_error() {
    let err = FluxionError::user_error(io::Error::other("test user error"));
    let cloned = err.clone();

    // UserError is converted to StreamProcessingError when cloned
    assert!(matches!(cloned, FluxionError::StreamProcessingError { .. }));
    assert!(cloned.to_string().contains("User error"));
    assert!(cloned.to_string().contains("test user error"));
}

#[test]
fn test_clone_multiple_errors() {
    let errors = vec![
        FluxionError::lock_error("lock1"),
        FluxionError::stream_error("stream1"),
    ];

    let err = FluxionError::MultipleErrors {
        count: errors.len(),
        errors,
    };

    let cloned = err.clone();

    if let FluxionError::MultipleErrors { count, errors } = cloned {
        assert_eq!(count, 2);
        assert_eq!(errors.len(), 2);
    } else {
        panic!("Expected MultipleErrors variant");
    }
}

#[test]
fn test_user_error_constructor() {
    let io_err = io::Error::other("test io error");
    let err = FluxionError::user_error(io_err);

    assert!(matches!(err, FluxionError::UserError(_)));
    assert!(err.to_string().contains("test io error"));
}

#[test]
fn test_error_variants_display() {
    let lock_err = FluxionError::lock_error("mutex_a");
    assert_eq!(lock_err.to_string(), "Failed to acquire lock: mutex_a");

    let stream_err = FluxionError::stream_error("processing failed");
    assert_eq!(
        stream_err.to_string(),
        "Stream processing error: processing failed"
    );

    let user_err = FluxionError::user_error(io::Error::other("custom error"));
    assert!(user_err.to_string().contains("User error"));
    assert!(user_err.to_string().contains("custom error"));

    let multi_err = FluxionError::MultipleErrors {
        count: 5,
        errors: vec![],
    };
    assert!(multi_err.to_string().contains("5 errors"));
}

#[test]
fn test_result_ext_multiple_contexts() {
    let result: Result<()> = Err(FluxionError::UserError("base error".into()));

    let err = result.context("first context").unwrap_err();

    assert!(err.to_string().contains("first context"));
    assert!(err.to_string().contains("base error"));
}

#[test]
fn test_nested_user_errors() {
    #[derive(Debug, thiserror::Error)]
    #[error("Outer: {0}")]
    struct OuterError(#[source] InnerError);

    #[derive(Debug, thiserror::Error)]
    #[error("Inner: {msg}")]
    struct InnerError {
        msg: String,
    }

    let inner = InnerError {
        msg: "root cause".to_string(),
    };
    let outer = OuterError(inner);
    let err = FluxionError::user_error(outer);

    assert!(matches!(err, FluxionError::UserError(_)));
    let display = err.to_string();
    assert!(display.contains("Outer"));
}

#[test]
fn test_error_type_sizes() {
    use std::mem::size_of;

    // Ensure error types are reasonably sized
    let error_size = size_of::<FluxionError>();
    // FluxionError should be reasonably sized (less than 128 bytes)
    assert!(
        error_size < 128,
        "FluxionError is too large: {error_size} bytes"
    );
}
