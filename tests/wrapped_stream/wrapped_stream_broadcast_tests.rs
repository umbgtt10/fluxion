use stream_processing::wrapped_stream::wrapped_stream_broadcast::WrappedStreamBroadcast;
use tokio_stream::StreamExt;

use crate::infra::infrastructure::assert_no_element_emitted;

#[tokio::test]
async fn main() {
    // Arrange
    let stream = WrappedStreamBroadcast::default();

    let mut subscription1 = stream.copy().filter_map(|item| {
        let item = item.unwrap();
        if item == "1" { Some(item) } else { None }
    });
    let mut subscription2 = stream.copy().filter_map(|item| {
        let item = item.unwrap();
        if item == "2" { Some(item) } else { None }
    });

    // Act
    stream.on_next("1".to_string());

    // Assert
    assert_eq!(subscription1.next().await, Some("1".to_string()));
    assert_no_element_emitted(&mut subscription2, 100).await;

    // Act
    stream.on_next("2".to_string());

    // Assert
    assert_no_element_emitted(&mut subscription1, 100).await;
    assert_eq!(subscription2.next().await, Some("2".to_string()));
}
