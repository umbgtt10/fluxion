use stream_processing::wrapped_stream::wrapped_stream_unbounded_tokio::WrappedStreamUnboundedTokio;
use tokio_stream::StreamExt;

use crate::infra::infrastructure::assert_no_element_emitted;

#[tokio::test]
async fn test_unbounded_wrapped_stream() {
    // Arrange
    let mut stream = WrappedStreamUnboundedTokio::default();

    let mut subscription1 = stream
        .copy()
        .await
        .filter_map(|item| if item == "1" { Some(item) } else { None });
    let mut subscription2 = stream
        .copy()
        .await
        .filter_map(|item| if item == "2" { Some(item) } else { None });

    // Act
    stream.on_next("1".to_string()).await;

    // Assert
    assert_eq!(subscription1.next().await, Some("1".to_string()));
    assert_no_element_emitted(&mut subscription2, 100).await;

    // Act
    stream.on_next("2".to_string()).await;

    // Assert
    assert_no_element_emitted(&mut subscription1, 100).await;
    assert_eq!(subscription2.next().await, Some("2".to_string()));
}
