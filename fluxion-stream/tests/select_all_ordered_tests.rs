use fluxion_stream::select_all_ordered::SelectAllExt;
use fluxion_stream::sequenced_channel::unbounded_channel;
use fluxion_test_utils::test_data::{DataVariant, expect_variant, send_variant};
use tokio_stream::wrappers::UnboundedReceiverStream;

#[tokio::test]
async fn test_select_all_ordered_all_permutations() {
    select_all_ordered_template_test(DataVariant::Person, DataVariant::Animal, DataVariant::Plant)
        .await;
    select_all_ordered_template_test(DataVariant::Person, DataVariant::Plant, DataVariant::Animal)
        .await;
    select_all_ordered_template_test(DataVariant::Plant, DataVariant::Animal, DataVariant::Person)
        .await;
    select_all_ordered_template_test(DataVariant::Plant, DataVariant::Person, DataVariant::Animal)
        .await;
    select_all_ordered_template_test(DataVariant::Animal, DataVariant::Person, DataVariant::Plant)
        .await;
    select_all_ordered_template_test(DataVariant::Animal, DataVariant::Plant, DataVariant::Person)
        .await;
}

/// Test template for `select_all_ordered` that verifies temporal Variant preservation in stream merging.
/// Sets up channels for Person/Animal/Plant, merges streams, sends in specified Variant, and asserts correct reception.
/// Crucial for deterministic async processing; ensures `select_all_ordered` avoids race conditions of `select_all`.
/// Called with different permutations to validate robustness across send sequences.
/// Validates sequence-based Varianting over poll-time randomness for reliable event handling.
async fn select_all_ordered_template_test(
    variant1: DataVariant,
    variant2: DataVariant,
    variant3: DataVariant,
) {
    // Arrange
    let (person_sender, person_receiver) = unbounded_channel();
    let (animal_sender, animal_receiver) = unbounded_channel();
    let (plant_sender, plant_receiver) = unbounded_channel();

    let person_stream = UnboundedReceiverStream::new(person_receiver.into_inner());
    let animal_stream = UnboundedReceiverStream::new(animal_receiver.into_inner());
    let plant_stream = UnboundedReceiverStream::new(plant_receiver.into_inner());

    let senders = vec![person_sender, animal_sender, plant_sender];
    let streams = vec![person_stream, animal_stream, plant_stream];

    let results = streams.select_all_ordered();

    // Act
    send_variant(&variant1, &senders);
    send_variant(&variant2, &senders);
    send_variant(&variant3, &senders);

    // Assert
    let mut results = Box::pin(results);

    expect_variant(&variant1, &mut results).await;
    expect_variant(&variant2, &mut results).await;
    expect_variant(&variant3, &mut results).await;
}
