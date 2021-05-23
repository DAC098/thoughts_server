use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::custom_field_entries::CustomFieldEntryType;
use crate::error;

pub struct CustomField {
    pub id: i32,
    pub name: String,
    pub owner: i32,
    pub config: CustomFieldType
}

pub async fn find_id(
    conn: &impl GenericClient,
    id: i32
) -> error::Result<Option<CustomField>> {
    let result = conn.query(
        r#"
        select id, 
               name, owner,
               config,
               comment
        from custom_fields 
        where id = $1
        "#,
        &[&id]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(CustomField {
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
) -> error::Result<CustomField> {
    if let Some(field) = find_id(conn, id).await? {
        if let Some(initiator) = initiator_opt {
            if field.owner != initiator {
                return Err(error::ResponseError::PermissionDenied(
                    format!("you do not haver permission to create a custom field entry using this field id: {}", field.owner)
                ))
            }
        }

        Ok(field)
    } else {
        Err(error::ResponseError::CustomFieldNotFound(id))
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
pub enum CustomFieldType {
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

pub fn verifiy(config: &CustomFieldType, value: &CustomFieldEntryType) -> error::Result<()> {
    match config {
        CustomFieldType::Integer {minimum, maximum} => {
            match value {
                CustomFieldEntryType::Integer {value} => {
                    verify_range(value, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "Integer mood field can only validate a Integer mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::IntegerRange {minimum, maximum} => {
            match value {
                CustomFieldEntryType::IntegerRange {low, high} => {
                    verify_range_bound(low, high, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "IntegerRange mood field can only validate a IntergerRange mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::Float {minimum, maximum} => {
            match value {
                CustomFieldEntryType::Float {value} => {
                    verify_range(value, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "Float mood field can only validate a Float mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::FloatRange {minimum, maximum} => {
            match value {
                CustomFieldEntryType::FloatRange {low, high} => {
                    verify_range_bound(low, high, minimum, maximum)
                },
                _ => Err(error::ResponseError::Validation(
                    "FloatRange mood field can only validate a FloatRange mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::Time {as_12hr: _} => {
            match value {
                CustomFieldEntryType::Time {value: _} => Ok(()),
                _ => Err(error::ResponseError::Validation(
                    "Time mood field can only validate a Time mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::TimeRange {show_diff: _, as_12hr: _} => {
            match value {
                CustomFieldEntryType::TimeRange {low, high} => {
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

