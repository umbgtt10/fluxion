use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawSensorValue {
    pub sensor_id: u8,
    pub value: f64,
}

impl Eq for RawSensorValue {}

impl PartialOrd for RawSensorValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RawSensorValue {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by sensor_id
        self.sensor_id.cmp(&other.sensor_id)
    }
}
