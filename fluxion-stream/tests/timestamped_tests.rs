use fluxion_stream::timestamped::Timestamped;

#[test]
fn test_timestamped_ordering() {
    let first = Timestamped::new("first");
    let second = Timestamped::new("second");
    let third = Timestamped::new("third");

    assert!(first < second);
    assert!(second < third);
    assert!(first < third);
}

#[test]
fn test_timestamped_deref() {
    let seq = Timestamped::new("hello");
    assert_eq!(seq.len(), 5); // Derefs to &str
}
