use fluxion_stream::sequenced::Sequenced;

#[test]
fn test_sequenced_ordering() {
    let first = Sequenced::new("first");
    let second = Sequenced::new("second");
    let third = Sequenced::new("third");

    assert!(first < second);
    assert!(second < third);
    assert!(first < third);
}

#[test]
fn test_sequenced_deref() {
    let seq = Sequenced::new("hello");
    assert_eq!(seq.len(), 5); // Derefs to &str
}
