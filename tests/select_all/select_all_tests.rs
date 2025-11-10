use futures::StreamExt;
use futures::stream::select_all;
use tokio::sync::mpsc::unbounded_channel;
use tokio_stream::wrappers::UnboundedReceiverStream;

async fn select_all_template_test(channel1: usize, channel2: usize, channel3: usize) {
    // Arrange
    let (tx1, rx1) = unbounded_channel();
    let (tx2, rx2) = unbounded_channel();
    let (tx3, rx3) = unbounded_channel();
    let senders = [tx1, tx2, tx3];

    let stream1 = UnboundedReceiverStream::new(rx1);
    let stream2 = UnboundedReceiverStream::new(rx2);
    let stream3 = UnboundedReceiverStream::new(rx3);

    let mut merged = select_all(vec![stream1, stream2, stream3]);

    // Act
    senders[channel1].send("First message").unwrap();
    senders[channel2].send("Second message").unwrap();
    senders[channel3].send("Third message").unwrap();

    // Assert
    assert_eq!(merged.next().await.unwrap(), "First message");
    assert_eq!(merged.next().await.unwrap(), "Second message");
    assert_eq!(merged.next().await.unwrap(), "Third message");
}

#[should_panic]
#[tokio::test]
async fn test_select_all_all_combinations() {
    /*

       These tests fail becuase select_all does not guarantee order of messages
       across multiple streams. Use select_all_ordered for that functionality (and pay the price!)

    */

    select_all_template_test(0, 1, 2).await; // channel 0, 1, 2
    select_all_template_test(0, 2, 1).await; // channel 0, 2, 1
    select_all_template_test(2, 1, 0).await; // channel 2, 1, 0
    select_all_template_test(2, 0, 1).await; // channel 2, 0, 1
    select_all_template_test(1, 0, 2).await; // channel 1, 0, 2
    select_all_template_test(1, 2, 0).await; // channel 1, 2, 0
}
