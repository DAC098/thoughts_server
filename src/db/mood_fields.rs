use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::mood_entries::MoodEntryType;
use crate::error;

pub struct MoodField {
    pub id: i32,
    pub name: String,
    pub owner: i32,
    pub config: MoodFieldType
}

pub async fn find_id(
    conn: &impl GenericClient,
    id: i32
) -> error::Result<Option<MoodField>> {
    let result = conn.query(
        r#"
        select id, 
               name, owner,
               config,
               comment
        from mood_fields 
        where id = $1
        "#,
        &[&id]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(MoodField {
            id: result[0].get(0),
            name: result[0].get(1),
            owner: result[0].get(2),
            config: serde_json::from_value(result[0].get(3)).unwrap()
        }))
    }
}

pub async fn get_via_id(
    conn: &impl GenericClient,
    id: i32,
    initiator_opt: Option<i32>,
) -> error::Result<MoodField> {
    if let Some(field) = find_id(conn, id).await? {
        if let Some(initiator) = initiator_opt {
            if field.owner != initiator {
                return Err(error::ResponseError::PermissionDenied(
                    format!("you do not haver permission to create a mood entry using this field id: {}", field.owner)
                ))
            }
        }

        Ok(field)
    } else {
        Err(error::ResponseError::MoodFieldNotFound(id))
    }
}

pub async fn get_via_mood_entry(
    conn: &impl GenericClient,
    mood_entry_id: i32,
    initiator_opt: Option<i32>
) -> error::Result<MoodField> {
    let result = conn.query(
        r#"
        select mood_entries.field,
               entries.owner 
        from mood_entries 
        join entries on mood_entries.entry = entries.id
        where mood_entries.id = $1
        "#, 
        &[&mood_entry_id]
    ).await?;

    if result.is_empty() {
        Err(error::ResponseError::MoodEntryNotFound(mood_entry_id))
    } else {
        if let Some(initiator) = initiator_opt {
            if initiator != result[0].get::<usize, i32>(1) {
                return Err(error::ResponseError::PermissionDenied(
                    format!("you do not own this mood entry. mood entry: {}", mood_entry_id)
                ));
            }
        }

        let field_id: i32 = result[0].get(0);

        if let Some(field) = find_id(conn, field_id).await? {
            Ok(field)
        } else {
            Err(error::ResponseError::MoodFieldNotFound(field_id))
        }
    }
}

fn default_time_range_show_diff() -> bool {
    false
}

fn default_as_12hr() -> bool {
    false
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MoodFieldType {
    Integer {
        minimum: Option<i32>,
        maximum: Option<i32>
    },
    IntegerRange {
        minimum: Option<i32>,
        maximum: Option<i32>
    },

    Float {
        minimum: Option<f32>,
        maximum: Option<f32>
    },
    FloatRange {
        minimum: Option<f32>,
        maximum: Option<f32>
    },

    Time {
        #[serde(default = "default_as_12hr")]
        as_12hr: bool
    },
    TimeRange {
        #[serde(default = "default_time_range_show_diff")]
        show_diff: bool,

        #[serde(default = "default_as_12hr")]
        as_12hr: bool
    },
}

fn verify_range<T>(value: &T, minimum: &Option<T>, maximum: &Option<T>) -> error::Result<()>
where
    T: PartialOrd + std::fmt::Display
{
    if let Some(min) = minimum {
        if value < min {
            return Err(error::ResponseError::Validation(
                format!("given value is less than the minimum specified. value[{}] minimum[{}]", value, min)
            ));
        }
    }

    if let Some(max) = maximum {
        if value > max {
            return Err(error::ResponseError::Validation(
                format!("given value is higher than the minimum specified. value[{}] maximum[{}]", value, max)
            ));
        }
    }

    return Ok(())
}

fn verify_range_bound<T>(low: &T, high: &T, minimum: &Option<T>, maximum: &Option<T>) -> error::Result<()>
where
    T: PartialOrd + std::fmt::Display
{
    if low > high {
        return Err(error::ResponseError::Validation(
            format!("given low is greater than the high. low[{}] high[{}]", low, high)
        ));
    }

    if let Some(min) = minimum {
        if low < min {
            return Err(error::ResponseError::Validation(
                format!("given low is less than the minimum specified. low[{}] minimum[{}]", low, min)
            ));
        }
    }

    if let Some(max) = maximum {
        if high > max {
            return Err(error::ResponseError::Validation(
                format!("given high is greater than the minimum specified. high[{}] maximum[{}]", high, max)
            ));
        }
    }

    return Ok(())
}

pub fn verifiy(config: &MoodFieldType, value: &MoodEntryType) -> error::Result<()> {
    match config {
        MoodFieldType::Integer {minimum, maximum} => {
            match value {
                MoodEntryType::Integer {value} => {
                    verify_range(value, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "Integer mood field can only validate a Integer mood entry".to_owned()
                ))
            }
        },
        MoodFieldType::IntegerRange {minimum, maximum} => {
            match value {
                MoodEntryType::IntegerRange {low, high} => {
                    verify_range_bound(low, high, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "IntegerRange mood field can only validate a IntergerRange mood entry".to_owned()
                ))
            }
        },
        MoodFieldType::Float {minimum, maximum} => {
            match value {
                MoodEntryType::Float {value} => {
                    verify_range(value, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "Float mood field can only validate a Float mood entry".to_owned()
                ))
            }
        },
        MoodFieldType::FloatRange {minimum, maximum} => {
            match value {
                MoodEntryType::FloatRange {low, high} => {
                    verify_range_bound(low, high, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "FloatRange mood field can only validate a FloatRange mood entry".to_owned()
                ))
            }
        },
        MoodFieldType::Time {as_12hr: _} => {
            match value {
                MoodEntryType::Time {value: _} => Ok(()),
                _ => Err(error::ResponseError::Validation(
                    "Time mood field can only validate a Time mood entry".to_owned()
                ))
            }
        },
        MoodFieldType::TimeRange {show_diff: _, as_12hr: _} => {
            match value {
                MoodEntryType::TimeRange {low, high} => {
                    let none_opt = None;
                    verify_range_bound(low, high, &none_opt, &none_opt)
                },
                _ => Err(error::ResponseError::Validation(
                    "TimeRange mood field can only validate a TimeRange mood entry".to_owned()
                ))
            }
        }
    }
}

