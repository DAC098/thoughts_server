use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum CustomFieldEntryType {
    Integer {
        value: i32
    },
    IntegerRange {
        low: i32,
        high: i32
    },

    Float {
        value: f32
    },
    FloatRange {
        low: f32,
        high: f32
    },

    Time {
        value: chrono::DateTime<chrono::Utc>
    },
    TimeRange {
        low: chrono::DateTime<chrono::Utc>,
        high: chrono::DateTime<chrono::Utc>
    },
}