// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// SPDX-License-Identifier: MIT OR Apache-2.0

use fluxion_stream::MergedStream;
use fluxion_test_utils::{test_channel, unwrap_stream, Sequenced};

#[tokio::test]
async fn test_merge_with_repository_pattern() -> anyhow::Result<()> {
    // Define domain events
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum Event {
        UserCreated { id: u32, name: String },
        OrderPlaced { user_id: u32, amount: u32 },
        PaymentReceived { user_id: u32, amount: u32 },
    }

    // Repository state tracking users and orders
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
    struct Repository {
        total_users: u32,
        total_orders: u32,
        total_revenue: u32,
    }

    // Create event streams
    let (tx_users, user_stream) = test_channel::<Sequenced<Event>>();
    let (tx_orders, order_stream) = test_channel::<Sequenced<Event>>();
    let (tx_payments, payment_stream) = test_channel::<Sequenced<Event>>();

    let user_created1 = Sequenced::with_timestamp(
        Event::UserCreated {
            id: 1,
            name: "Alice".into(),
        },
        1,
    );
    let user_created2 = Sequenced::with_timestamp(
        Event::UserCreated {
            id: 2,
            name: "Bob".into(),
        },
        2,
    );
    let order_placed1 = Sequenced::with_timestamp(
        Event::OrderPlaced {
            user_id: 1,
            amount: 100,
        },
        3,
    );
    let payment_received1 = Sequenced::with_timestamp(
        Event::PaymentReceived {
            user_id: 1,
            amount: 100,
        },
        4,
    );
    let order_placed2 = Sequenced::with_timestamp(
        Event::OrderPlaced {
            user_id: 2,
            amount: 200,
        },
        5,
    );

    // Merge all event streams into a single repository state
    let mut repository_stream = MergedStream::seed::<Sequenced<Repository>>(Repository {
        total_users: 0,
        total_orders: 0,
        total_revenue: 0,
    })
    .merge_with(user_stream, |event: Event, repo: &mut Repository| {
        if let Event::UserCreated { .. } = event {
            repo.total_users += 1;
        }
        repo.clone()
    })
    .merge_with(order_stream, |event: Event, repo: &mut Repository| {
        if let Event::OrderPlaced { .. } = event {
            repo.total_orders += 1;
        }
        repo.clone()
    })
    .merge_with(payment_stream, |event: Event, repo: &mut Repository| {
        if let Event::PaymentReceived { amount, .. } = event {
            repo.total_revenue += amount;
        }
        repo.clone()
    });

    // Emit events in temporal order
    tx_users.unbounded_send(user_created1)?;
    tx_users.unbounded_send(user_created2)?;
    tx_orders.unbounded_send(order_placed1)?;
    tx_payments.unbounded_send(payment_received1)?;
    tx_orders.unbounded_send(order_placed2)?;

    // Verify repository state updates
    let state1 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state1.value.total_users, 1);
    assert_eq!(state1.value.total_orders, 0);
    assert_eq!(state1.value.total_revenue, 0);

    let state2 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state2.value.total_users, 2);

    let state3 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state3.value.total_orders, 1);

    let state4 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state4.value.total_revenue, 100);

    let state5 = unwrap_stream(&mut repository_stream, 500).await.unwrap();
    assert_eq!(state5.value.total_orders, 2);

    Ok(())
}
