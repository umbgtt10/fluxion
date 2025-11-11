use std::sync::Arc;
use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;
use tokio::sync::{Mutex, mpsc};
use tokio::time::{Duration, sleep};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

const BUFFER_SIZE: usize = 10;

#[tokio::test]
async fn test_subscribe_latest_async_no_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            {
                let mut items = collected_items.lock().await;
                items.push(item);
                sleep(Duration::from_millis(50)).await;
            }
            Ok(())
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act
    for i in 1..=BUFFER_SIZE {
        sender.send(i).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }

    // Assert
    assert_subset_items_processed_sequentially(&collected_items, BUFFER_SIZE).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            if item % 3 == 0 {
                return Err(format!("item {} could not be processed", item));
            }

            let mut items = collected_items.lock().await;
            items.push(item);

            sleep(Duration::from_millis(50)).await;

            Ok(())
        }
    };

    let error_callback = |err: String| {
        eprintln!("Error occurred: {:?}", err);
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), None)
                .await;
        }
    });

    // Act
    for i in 1..=BUFFER_SIZE {
        sender.send(i).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }

    // Assert
    assert_subset_items_processed_sequentially(&collected_items, BUFFER_SIZE).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_no_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);

    let func = move |item, ctx: CancellationToken| {
        let collected_items = collected_items_clone.clone();
        async move {
            if ctx.is_cancelled() {
                return Ok(());
            }

            let mut items = collected_items.lock().await;
            items.push(item);

            sleep(Duration::from_millis(50)).await;

            Ok(())
        }
    };

    let error_callback = |_err: ()| {
        eprintln!("Error occurred.");
    };

    let cancellation_token = CancellationToken::new();
    let task_handle = tokio::spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), Some(cancellation_token))
                .await;
        }
    });

    // Act
    for i in 1..=4 {
        sender.send(i).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }
    cancellation_token.cancel();

    // Assert
    assert_subset_items_processed_sequentially(&collected_items, BUFFER_SIZE).await;

    // Act
    for i in 5..=BUFFER_SIZE {
        sender.send(i).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }

    // Assert
    assert_subset_items_processed_sequentially(&collected_items, BUFFER_SIZE).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_and_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);

    let func = move |item, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            if item % 3 == 0 {
                return Err(());
            }

            let mut items = collected_items.lock().await;
            items.push(item);

            sleep(Duration::from_millis(50)).await;

            Ok(())
        }
    };

    let error_callback = |err: ()| {
        eprintln!("Error occurred: {:?}", err);
    };

    let cancellation_token = CancellationToken::new();
    let task_handle = tokio::spawn({
        let cancellation_token = cancellation_token.clone();

        async move {
            stream
                .subscribe_latest_async(func, Some(error_callback), Some(cancellation_token))
                .await;
        }
    });

    // Act
    for i in 1..=4 {
        sender.send(i).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }
    cancellation_token.cancel();

    // Assert
    assert_subset_items_processed_sequentially(&collected_items, BUFFER_SIZE).await;

    // Act
    for i in 5..=BUFFER_SIZE {
        sender.send(i).await.unwrap();
        sleep(Duration::from_millis(10)).await;
    }

    // Assert
    assert_subset_items_processed_sequentially(&collected_items, BUFFER_SIZE).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

async fn assert_subset_items_processed_sequentially(
    processed_items: &Arc<Mutex<Vec<usize>>>,
    number_of_items: usize,
) {
    let processed_items = processed_items.lock().await;
    assert!(
        !processed_items.is_empty() && processed_items.len() < number_of_items,
        "Expected items must be processed, but found {}",
        processed_items.len()
    );

    for items in processed_items.windows(2) {
        assert!(
            items[0] < items[1],
            "Processed items should be in ascending order"
        );
    }
}
