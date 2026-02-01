// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_core::CancellationToken;
use futures::{channel::mpsc::UnboundedSender, FutureExt};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{sleep, Duration};

use crate::domain::models::Order;

pub struct LegacyMessageQueue {
    order_id_counter: AtomicU64,
}

impl LegacyMessageQueue {
    pub fn new() -> Self {
        Self {
            order_id_counter: AtomicU64::new(5000),
        }
    }

    pub async fn consume_orders(self, tx: UnboundedSender<Order>, cancel: CancellationToken) {
        println!("  📨 Legacy Message Queue: Consuming order events (every 2s)");

        loop {
            futures::select! {
                _ = cancel.cancelled().fuse() => {
                    println!("  📨 Legacy Message Queue: Shutting down");
                    break;
                }
                _ = sleep(Duration::from_secs(2)).fuse() => {
                    let order_id = self.order_id_counter.fetch_add(1, Ordering::SeqCst);

                    let order = Order {
                        id: order_id,
                        user_id: 1000 + fastrand::u64(0..5),
                        product_id: 100 + fastrand::u64(0..3),
                        quantity: fastrand::u32(1..10),
                        status: Default::default(),
                    };

                    let _xml = quick_xml::se::to_string(&order).unwrap();

                    if tx.unbounded_send(order).is_err() {
                        break;
                    }
                }
            }
        }
    }
}
