use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::mood_entries::MoodEntryType;
use crate::error;

pub struct MoodField {
    id: i32,
    name: String,
    owner: i32,
    config: MoodFieldType
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
        from mood_fields where id = $1
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
            config: serde_json::from_value(result[0].get(3))?
        }))
    }
}

impl MoodField {

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn get_owner(&self) -> i32 {
        self.owner
    }

    pub fn get_config(&self) -> MoodFieldType {
        self.config.clone()
    }
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

    Time {},
    TimeRange {},
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
        MoodFieldType::Time {} => {
            match value {
                MoodEntryType::Time {value: _} => Ok(()),
                _ => Err(error::ResponseError::Validation(
                    "Time mood field can only validate a Time mood entry".to_owned()
                ))
            }
        },
        MoodFieldType::TimeRange {} => {
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

