// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::SubjectError;
use std::error::Error;

#[test]
fn test_subject_error_closed_display() {
    // Arrange & Act
    let error = SubjectError::Closed;

    // Assert
    assert_eq!(error.to_string(), "Subject is closed");
}

#[test]
fn test_subject_error_clone() {
    // Arrange
    let error = SubjectError::Closed;

    // Act
    let cloned = error.clone();

    // Assert
    assert_eq!(error, cloned);
}

#[test]
fn test_subject_error_partial_eq() {
    // Arrange & Act
    let error1 = SubjectError::Closed;
    let error2 = SubjectError::Closed;

    // Assert
    assert_eq!(error1, error2);
}

#[test]
fn test_subject_error_debug() {
    // Arrange
    let error = SubjectError::Closed;

    // Act
    let debug_str = format!("{:?}", error);

    // Assert
    assert_eq!(debug_str, "Closed");
}

#[test]
fn test_subject_error_implements_error_trait() {
    // Arrange & Act
    let error = SubjectError::Closed;

    // Assert
    let _: &dyn Error = &error;
}

#[test]
fn test_subject_error_source_is_none() {
    // Arrange & Act
    let error = SubjectError::Closed;

    // Assert
    assert!(error.source().is_none());
}

#[test]
fn test_subject_error_is_send() {
    // Arrange & Act & Assert
    fn assert_send<T: Send>() {}
    assert_send::<SubjectError>();
}

#[test]
fn test_subject_error_is_sync() {
    // Arrange & Act & Assert
    fn assert_sync<T: Sync>() {}
    assert_sync::<SubjectError>();
}

#[test]
fn test_subject_error_eq_trait() {
    // Arrange & Act
    let error = SubjectError::Closed;

    // Assert
    assert_eq!(error, SubjectError::Closed);
}

#[test]
fn test_subject_error_boxed_error() {
    // Arrange & Act
    let error: Box<dyn Error> = Box::new(SubjectError::Closed);

    // Assert
    assert_eq!(error.to_string(), "Subject is closed");
}

#[test]
fn test_subject_error_in_result() {
    // Arrange & Act
    let result: Result<(), SubjectError> = Err(SubjectError::Closed);

    // Assert
    assert!(result.is_err());
    assert_eq!(result, Err(SubjectError::Closed));
}

#[test]
fn test_subject_error_error_chain() {
    // Arrange & Act
    let error = SubjectError::Closed;
    let err_ref: &dyn Error = &error;

    // Assert
    assert!(error.source().is_none());
    assert_eq!(err_ref.to_string(), "Subject is closed");
}
