// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_ordered_merge::OrderedMergeExt;
use fluxion_test_utils::test_data::{
    animal_cat, animal_dog, animal_spider, person_alice, person_bob, person_charlie, plant_fern,
    plant_rose, plant_sunflower, TestData,
};
use fluxion_test_utils::Sequenced;
use futures::channel::mpsc;
use futures::StreamExt;

/// Generate all permutations of [0, 1, 2]
fn all_channel_permutations() -> Vec<[usize; 3]> {
    vec![
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ]
}

/// Generate send orders using a simplified pattern
/// We send 3 items from each channel, but in a structured way:
/// We have 3 "rounds", and in each round we pick one item from each of the 3 channels
/// This gives us 3! permutations per round, so (3!)^3 = 6^3 = 216 total patterns
fn generate_send_orders() -> Vec<[usize; 9]> {
    let mut orders = Vec::new();

    // Generate all permutations of [0, 1, 2] for each of 3 rounds
    let round_perms = vec![
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    for round1 in &round_perms {
        for round2 in &round_perms {
            for round3 in &round_perms {
                let order = [
                    round1[0], round1[1], round1[2], round2[0], round2[1], round2[2], round3[0],
                    round3[1], round3[2],
                ];
                orders.push(order);
            }
        }
    }

    orders
}

#[tokio::test]
async fn test_ordered_merge_all_permutations() -> anyhow::Result<()> {
    // Test that temporal ordering (sequence numbers) is preserved
    // regardless of channel order or send pattern
    // This tests 6 channel permutations x 216 send orders = 1296 test cases

    let channel_permutations = all_channel_permutations();
    let send_orders = generate_send_orders();

    assert_eq!(
        send_orders.len(),
        216,
        "Expected 216 send order patterns (6^3)"
    );
    assert_eq!(
        channel_permutations.len(),
        6,
        "Expected 6 channel permutations"
    );

    println!(
        "Testing {} channel permutations � {} send orders = {} total test cases",
        channel_permutations.len(),
        send_orders.len(),
        channel_permutations.len() * send_orders.len()
    );

    for channel_order in &channel_permutations {
        for send_order in &send_orders {
            // Arrange - create three channels
            let (person_tx, person_rx) = mpsc::unbounded::<Sequenced<TestData>>();
            let (animal_tx, animal_rx) = mpsc::unbounded::<Sequenced<TestData>>();
            let (plant_tx, plant_rx) = mpsc::unbounded::<Sequenced<TestData>>();

            let person_stream = person_rx;
            let animal_stream = animal_rx;
            let plant_stream = plant_rx;

            // Build streams vec based on permutation order
            let streams = match channel_order {
                [0, 1, 2] => vec![person_stream, animal_stream, plant_stream],
                [0, 2, 1] => vec![person_stream, plant_stream, animal_stream],
                [1, 0, 2] => vec![animal_stream, person_stream, plant_stream],
                [1, 2, 0] => vec![animal_stream, plant_stream, person_stream],
                [2, 0, 1] => vec![plant_stream, person_stream, animal_stream],
                [2, 1, 0] => vec![plant_stream, animal_stream, person_stream],
                _ => panic!("Invalid permutation"),
            };

            let mut results = Box::pin(streams.ordered_merge());

            // Act - send values according to send_order
            // Track which value to send next for each channel
            let mut person_idx = 0;
            let mut animal_idx = 0;
            let mut plant_idx = 0;

            let mut expected_order = Vec::new();

            for &channel in send_order {
                match channel {
                    0 => {
                        let value = match person_idx {
                            0 => person_alice(),
                            1 => person_bob(),
                            2 => person_charlie(),
                            _ => panic!("Too many person values"),
                        };
                        person_tx
                            .unbounded_send(Sequenced::new(value.clone()))
                            .unwrap();
                        expected_order.push(value);
                        person_idx += 1;
                    }
                    1 => {
                        let value = match animal_idx {
                            0 => animal_dog(),
                            1 => animal_spider(),
                            2 => animal_cat(),
                            _ => panic!("Too many animal values"),
                        };
                        animal_tx
                            .unbounded_send(Sequenced::new(value.clone()))
                            .unwrap();
                        expected_order.push(value);
                        animal_idx += 1;
                    }
                    2 => {
                        let value = match plant_idx {
                            0 => plant_rose(),
                            1 => plant_sunflower(),
                            2 => plant_fern(),
                            _ => panic!("Too many plant values"),
                        };
                        plant_tx
                            .unbounded_send(Sequenced::new(value.clone()))
                            .unwrap();
                        expected_order.push(value);
                        plant_idx += 1;
                    }
                    _ => panic!("Invalid channel index"),
                }
            }

            // Drop senders to signal end of streams
            drop(person_tx);
            drop(animal_tx);
            drop(plant_tx);

            // Yield to allow all messages to be processed
            tokio::task::yield_now().await;

            // Assert - all 9 values come out in send order
            for (i, expected_value) in expected_order.iter().enumerate() {
                let result = results.next().await;
                assert!(
                    result.is_some(),
                    "Expected value at position {} for channel_order={:?}, send_order={:?}",
                    i,
                    channel_order,
                    send_order
                );
                assert_eq!(
                    &result.unwrap().value,
                    expected_value,
                    "Value mismatch at position {} for channel_order={:?}, send_order={:?}",
                    i,
                    channel_order,
                    send_order
                );
            }
        }
    }

    Ok(())
}
