// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::time::Duration;

use fluxion_rx::prelude::*;
use fluxion_test_utils::{assert_no_element_emitted, helpers::unwrap_stream};
use futures::StreamExt;
use tokio::{
    spawn,
    sync::mpsc::{self, unbounded_channel},
    time::sleep,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SensorReading {
    timestamp: u64,
    temperature: i32,
}

impl Ordered for SensorReading {
    type Inner = Self;
    fn order(&self) -> u64 {
        self.timestamp
    }
    fn get(&self) -> &Self::Inner {
        self
    }
    fn with_order(value: Self, _order: u64) -> Self {
        value
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DataEvent {
    Sensor(SensorReading),
}

impl Ordered for DataEvent {
    type Inner = Self;
    fn order(&self) -> u64 {
        match self {
            DataEvent::Sensor(s) => s.timestamp,
        }
    }
    fn get(&self) -> &Self::Inner {
        self
    }
    fn with_order(value: Self, _order: u64) -> Self {
        value
    }
}

#[tokio::test]
async fn test_into_fluxion_stream_basic_transformation() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();

    let reading1 = SensorReading {
        timestamp: 100,
        temperature: 20,
    };
    let reading2 = SensorReading {
        timestamp: 200,
        temperature: 25,
    };

    let mut stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    // Act
    tx.send(reading1.clone())?;
    tx.send(reading2.clone())?;

    // Assert
    let result1 = unwrap_stream(&mut stream, 500).await;
    assert_eq!(result1.get(), &DataEvent::Sensor(reading1));

    let result2 = unwrap_stream(&mut stream, 500).await;
    assert_eq!(result2.get(), &DataEvent::Sensor(reading2));

    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_empty_channel() -> anyhow::Result<()> {
    // Arrange
    let (_tx, rx) = mpsc::unbounded_channel::<SensorReading>();

    let mut stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    // Act
    drop(_tx);

    // Assert
    assert!(stream.next().await.is_none());

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_preserves_order() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = mpsc::unbounded_channel::<SensorReading>();

    let readings: Vec<SensorReading> = (0..10)
        .map(|i| SensorReading {
            timestamp: i * 100,
            temperature: (i * 5) as i32,
        })
        .collect();

    // Act
    let mut stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    for reading in &readings {
        tx.send(reading.clone())?;
    }

    // Assert
    for expected in &readings {
        let result = unwrap_stream(&mut stream, 500).await;
        assert_eq!(result.get(), &DataEvent::Sensor(expected.clone()));
    }

    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_multiple_items() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel::<SensorReading>();

    let mut stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    // Act
    for i in 0..5 {
        tx.send(SensorReading {
            timestamp: i * 100,
            temperature: (i * 10) as i32,
        })?;
    }

    // Assert
    for i in 0..5 {
        let result = unwrap_stream(&mut stream, 500).await;
        let expected = SensorReading {
            timestamp: i as u64 * 100,
            temperature: (i * 10),
        };
        assert_eq!(result.get(), &DataEvent::Sensor(expected));
    }

    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_transformation_logic() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel::<SensorReading>();
    let expected = SensorReading {
        timestamp: 200,
        temperature: 20,
    };

    let mut stream = rx.into_fluxion_stream(|s| {
        let mut reading = s.clone();
        reading.timestamp *= 2;
        DataEvent::Sensor(reading)
    });

    let original = SensorReading {
        timestamp: 100,
        temperature: 20,
    };

    // Act
    tx.send(original.clone())?;

    // Assert
    let result = unwrap_stream(&mut stream, 500).await;
    assert_eq!(result.get(), &DataEvent::Sensor(expected));

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_can_combine_with_other_streams() -> anyhow::Result<()> {
    // Arrange
    let (tx1, rx1) = unbounded_channel::<SensorReading>();
    let (tx2, rx2) = unbounded_channel::<SensorReading>();

    let stream1 = rx1.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));
    let stream2 = rx2.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    let mut merged = stream1.ordered_merge(vec![stream2]);

    // Act - merge two transformed streams
    tx1.send(SensorReading {
        timestamp: 100,
        temperature: 20,
    })?;

    tx2.send(SensorReading {
        timestamp: 200,
        temperature: 25,
    })?;

    // Assert - both items should be in the merged stream
    let result1 = unwrap_stream(&mut merged, 500).await;
    assert_eq!(result1.order(), 100);

    let result2 = unwrap_stream(&mut merged, 500).await;
    assert_eq!(result2.order(), 200);

    assert_no_element_emitted(&mut merged, 100).await;

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_late_sends() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel::<SensorReading>();

    let mut stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    // Act - send items after creating stream with delays
    spawn(async move {
        sleep(Duration::from_millis(10)).await;
        tx.send(SensorReading {
            timestamp: 100,
            temperature: 20,
        })
        .unwrap();

        sleep(Duration::from_millis(10)).await;
        tx.send(SensorReading {
            timestamp: 200,
            temperature: 25,
        })
        .unwrap();
    });

    // Assert
    let result1 = unwrap_stream(&mut stream, 500).await;
    assert_eq!(result1.order(), 100);

    let result2 = unwrap_stream(&mut stream, 500).await;
    assert_eq!(result2.order(), 200);

    Ok(())
}

#[tokio::test]
async fn test_into_fluxion_stream_high_volume() -> anyhow::Result<()> {
    // Arrange
    let (tx, rx) = unbounded_channel::<SensorReading>();

    let mut stream = rx.into_fluxion_stream(|s| DataEvent::Sensor(s.clone()));

    const COUNT: usize = 1000;

    // Act
    for i in 0..COUNT {
        tx.send(SensorReading {
            timestamp: i as u64,
            temperature: (i % 100) as i32,
        })?;
    }

    // Assert
    for i in 0..COUNT {
        let result = unwrap_stream(&mut stream, 500).await;
        assert_eq!(result.order(), i as u64);
    }

    assert_no_element_emitted(&mut stream, 100).await;

    Ok(())
}
