use fluxion_exec::subscribe_async::SubscribeAsyncExt;
use std::{sync::Arc, sync::Mutex as StdMutex, time::Duration};
use tokio::{
    sync::{Mutex as TokioMutex, mpsc},
    time::sleep,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

const BUFFER_SIZE: usize = 10;

#[tokio::test]
async fn test_subscribe_async_sequential_processing() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let results = Arc::new(TokioMutex::new(Vec::new()));

    let func = {
        let results = results.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            async move {
                results.lock().await.push(item);
                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await
        }
    });

    // Act
    sender.send(1).await.unwrap();
    sleep(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1]);

    // Repeat for other items.
    sender.send(2).await.unwrap();
    sleep(Duration::from_millis(50)).await;
    assert_eq!(*results.lock().await, vec![1, 2]);

    sender.send(3).await.unwrap();
    sleep(Duration::from_millis(50)).await;
    assert_eq!(*results.lock().await, vec![1, 2, 3]);

    sender.send(4).await.unwrap();
    sleep(Duration::from_millis(50)).await;
    assert_eq!(*results.lock().await, vec![1, 2, 3, 4]);

    sender.send(5).await.unwrap();
    sleep(Duration::from_millis(50)).await;
    assert_eq!(*results.lock().await, vec![1, 2, 3, 4, 5]);

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_with_errors() {
    // Arrange
    let (sender, receiver) = mpsc::channel(BUFFER_SIZE);
    let stream = ReceiverStream::new(receiver);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));

    let func = {
        let results = results.clone();
        move |item, _ctx: CancellationToken| {
            let results = results.clone();
            async move {
                if item % 2 == 0 {
                    return Err(format!("Error processing item: {}", item));
                }
                results.lock().await.push(item);
                Ok::<(), String>(())
            }
        }
    };

    let error_callback = {
        let errors = errors.clone();
        move |err| {
            errors.lock().unwrap().push(err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, None, Some(error_callback))
                .await;
        }
    });

    // Act
    sender.send(1).await.unwrap();
    sender.send(2).await.unwrap();
    sender.send(3).await.unwrap();
    sender.send(4).await.unwrap();
    sender.send(5).await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1, 3, 5]);
    assert_eq!(
        *errors.lock().unwrap(),
        vec![
            "Error processing item: 2".to_string(),
            "Error processing item: 4".to_string()
        ]
    );

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_triggered_cancellation_token() {
    // Arrange
    let (sender, receiver) = mpsc::channel(10);
    let stream = ReceiverStream::new(receiver);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();

    let func = {
        let results = results.clone();
        move |item: i32, ctx: CancellationToken| {
            let results = results.clone();
            async move {
                sleep(Duration::from_millis(20)).await;

                if ctx.is_cancelled() {
                    return Ok(());
                }

                results.lock().await.push(item);

                Ok::<(), ()>(())
            }
        }
    };

    let error_callback = {
        move |err| {
            panic!("Unexpected error while processing: {:?}", err);
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, Some(cancellation_token), Some(error_callback))
                .await;
        }
    });

    // Act
    sender.send(1).await.unwrap();
    sender.send(2).await.unwrap();
    sleep(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1, 2]);

    // Act
    cancellation_token_clone.cancel();
    sender.send(3).await.unwrap();
    sender.send(4).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1, 2]);

    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_async_errors_and_triggered_cancellation_token() {
    // Arrange
    let (sender, receiver) = mpsc::channel(10);
    let stream = ReceiverStream::new(receiver);
    let results = Arc::new(TokioMutex::new(Vec::new()));
    let errors = Arc::new(StdMutex::new(Vec::new()));
    let cancellation_token = CancellationToken::new();
    let cancellation_token_clone = cancellation_token.clone();

    #[derive(Debug, PartialEq, Eq)]
    enum ProcessingError {
        Cancelled(String),
        Other(String),
    }

    let func = {
        let results = results.clone();
        move |item: i32, ctx: CancellationToken| {
            let results = results.clone();
            async move {
                sleep(Duration::from_millis(20)).await;

                if ctx.is_cancelled() {
                    return Err(ProcessingError::Cancelled(format!(
                        "Cancelled during processing of item: {}",
                        item
                    )));
                }

                results.lock().await.push(item);

                if item == 3 {
                    Err(ProcessingError::Other(
                        "Failed to process item 3".to_string(),
                    ))
                } else {
                    Ok::<(), ProcessingError>(())
                }
            }
        }
    };

    let error_callback = {
        let errors = errors.clone();
        move |err| {
            if let ProcessingError::Other(_) = err {
                errors.lock().unwrap().push(err);
            }
        }
    };

    let task_handle = tokio::spawn({
        async move {
            stream
                .subscribe_async(func, Some(cancellation_token), Some(error_callback))
                .await;
        }
    });

    // Act
    sender.send(1).await.unwrap();
    sender.send(2).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1, 2]);
    assert!(errors.lock().unwrap().is_empty());

    // Act
    sender.send(3).await.unwrap();
    sleep(Duration::from_millis(50)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1, 2, 3]);
    assert_eq!(
        *errors.lock().unwrap(),
        vec![ProcessingError::Other(
            "Failed to process item 3".to_string()
        )]
    );

    // Act
    cancellation_token_clone.cancel();
    sender.send(4).await.unwrap();
    sender.send(5).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Assert
    assert_eq!(*results.lock().await, vec![1, 2, 3]);
    assert_eq!(
        *errors.lock().unwrap(),
        vec![ProcessingError::Other(
            "Failed to process item 3".to_string()
        )]
    );

    drop(sender);
    task_handle.await.unwrap();
}
