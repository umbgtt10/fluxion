use fluxion_exec::subscribe_latest_async::SubscribeLatestAsyncExt;
use fluxion_stream::sequenced_channel::unbounded_channel;
use fluxion_test_utils::test_value::{
    animal_ant, animal_cat, animal_dog, animal_spider, person_alice, person_bob, person_charlie,
    person_dave, person_diane, plant_rose, push, TestValue,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tokio_stream::{StreamExt as _, wrappers::UnboundedReceiverStream};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn test_subscribe_latest_async_no_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|sequenced| sequenced.value);

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
    push(person_alice(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_diane(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_dave(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_cat(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_ant(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_spider(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(plant_rose(), &sender);
    sleep(Duration::from_millis(10)).await;

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_error_no_cancellation_token() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|sequenced| sequenced.value);

    let func = move |item: TestValue, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            // Error on every third person (Bob, Dave)
            if matches!(&item, TestValue::Person(p) if p.name == "Bob" || p.name == "Dave") {
                return Err(format!("Failed to process {:?}", item));
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
    push(person_alice(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender); // Error
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_dave(), &sender); // Error
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender);
    sleep(Duration::from_millis(10)).await;

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_no_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();
    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|sequenced| sequenced.value);

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
    push(person_alice(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_diane(), &sender);
    sleep(Duration::from_millis(10)).await;
    cancellation_token.cancel();

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Act - these should not be processed after cancellation
    push(person_dave(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender);
    sleep(Duration::from_millis(10)).await;

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

#[tokio::test]
async fn test_subscribe_latest_async_with_cancellation_and_errors() {
    // Arrange
    let collected_items = Arc::new(Mutex::new(Vec::new()));
    let collected_items_clone = collected_items.clone();

    let (sender, receiver) = unbounded_channel();
    let stream =
        UnboundedReceiverStream::new(receiver.into_inner()).map(|sequenced| sequenced.value);

    let func = move |item: TestValue, _| {
        let collected_items = collected_items_clone.clone();
        async move {
            // Error on animals
            if matches!(&item, TestValue::Animal(_)) {
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
    push(person_alice(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(person_bob(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_dog(), &sender); // Error
    sleep(Duration::from_millis(10)).await;
    push(person_charlie(), &sender);
    sleep(Duration::from_millis(10)).await;
    cancellation_token.cancel();

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Act - these should not be processed after cancellation
    push(person_diane(), &sender);
    sleep(Duration::from_millis(10)).await;
    push(animal_cat(), &sender);
    sleep(Duration::from_millis(10)).await;

    // Assert
    assert_subset_items_processed(&collected_items).await;

    // Cleanup
    drop(sender);
    task_handle.await.unwrap();
}

async fn assert_subset_items_processed(processed_items: &Arc<Mutex<Vec<TestValue>>>) {
    let processed_items = processed_items.lock().await;
    assert!(
        !processed_items.is_empty(),
        "Expected some items to be processed, but found none"
    );
}
