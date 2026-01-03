// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_core::SubjectError;
use std::error::Error;

#[test]
fn test_subject_error_closed_display() {
    let error = SubjectError::Closed;
    assert_eq!(error.to_string(), "Subject is closed");
}

#[test]
fn test_subject_error_clone() {
    let error = SubjectError::Closed;
    let cloned = error.clone();
    assert_eq!(error, cloned);
}

#[test]
fn test_subject_error_partial_eq() {
    let error1 = SubjectError::Closed;
    let error2 = SubjectError::Closed;
    assert_eq!(error1, error2);
}

#[test]
fn test_subject_error_debug() {
    let error = SubjectError::Closed;
    let debug_str = format!("{:?}", error);
    assert_eq!(debug_str, "Closed");
}

#[test]
fn test_subject_error_implements_error_trait() {
    let error = SubjectError::Closed;
    let _: &dyn Error = &error;
}

#[test]
fn test_subject_error_source_is_none() {
    let error = SubjectError::Closed;
    assert!(error.source().is_none());
}

#[test]
fn test_subject_error_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<SubjectError>();
}

#[test]
fn test_subject_error_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<SubjectError>();
}

#[test]
fn test_subject_error_eq_trait() {
    let error = SubjectError::Closed;
    assert_eq!(error, SubjectError::Closed);
}

#[test]
fn test_subject_error_boxed_error() {
    let error: Box<dyn Error> = Box::new(SubjectError::Closed);
    assert_eq!(error.to_string(), "Subject is closed");
}

#[test]
fn test_subject_error_in_result() {
    let result: Result<(), SubjectError> = Err(SubjectError::Closed);
    assert!(result.is_err());
    assert_eq!(result, Err(SubjectError::Closed));
}

#[test]
fn test_subject_error_error_chain() {
    let error = SubjectError::Closed;

    // Error has no source
    assert!(error.source().is_none());

    // Can be treated as a dyn Error
    let err_ref: &dyn Error = &error;
    assert_eq!(err_ref.to_string(), "Subject is closed");
}
