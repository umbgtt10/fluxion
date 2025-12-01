// Copyright 2025 Umberto Gotti <umberto.gotti@umberto.gotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use fluxion_rx::ReceiverStreamExt;
use fluxion_test_utils::assert_stream_ended;
use fluxion_test_utils::helpers::unwrap_stream;
use fluxion_test_utils::test_data::{person_alice, person_bob};
use fluxion_test_utils::unwrap_value;
use tokio::sync::mpsc::unbounded_channel;

#[tokio::test]
async fn test_functional_from_unbounded_receiver() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel();
    let mut stream = rx.to_fluxion_stream();

    // Act
    tx.send(person_alice())?;
    tx.send(person_bob())?;
    drop(tx);

    // Assert
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        person_alice()
    );
    assert_eq!(
        unwrap_value(Some(unwrap_stream(&mut stream, 500).await)),
        person_bob()
    );
    assert_stream_ended(&mut stream, 500).await;

    Ok(())
}
