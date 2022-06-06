use std::{fmt};

use crate::db::{custom_fields, custom_field_entries, error};

use custom_fields::{CustomFieldType};
use custom_field_entries::{CustomFieldEntryType};

fn verify_range<T>(value: &T, minimum: &Option<T>, maximum: &Option<T>) -> error::Result<()>
where
    T: PartialOrd + fmt::Display
{
    if let Some(min) = minimum {
        if value < min {
            return Err(error::Error::Validation(
                format!("given value is less than the minimum specified. value[{}] minimum[{}]", value, min)
            ));
        }
    }

    if let Some(max) = maximum {
        if value > max {
            return Err(error::Error::Validation(
                format!("given value is higher than the minimum specified. value[{}] maximum[{}]", value, max)
            ));
        }
    }

    return Ok(())
}

fn verify_range_bound<T>(low: &T, high: &T, minimum: &Option<T>, maximum: &Option<T>) -> error::Result<()>
where
    T: PartialOrd + fmt::Display
{
    if low > high {
        return Err(error::Error::Validation(
            format!("given low is greater than the high. low[{}] high[{}]", low, high)
        ));
    }

    if let Some(min) = minimum {
        if low < min {
            return Err(error::Error::Validation(
                format!("given low is less than the minimum specified. low[{}] minimum[{}]", low, min)
            ));
        }
    }

    if let Some(max) = maximum {
        if high > max {
            return Err(error::Error::Validation(
                format!("given high is greater than the minimum specified. high[{}] maximum[{}]", high, max)
            ));
        }
    }

    return Ok(())
}

pub fn verifiy_custom_field_entry(config: &CustomFieldType, value: &CustomFieldEntryType) -> error::Result<()> {
    match config {
        CustomFieldType::Integer {minimum, maximum} => {
            match value {
                CustomFieldEntryType::Integer {value} => {
                    verify_range(value, minimum, maximum)
                },
                _ => Err(error::Error::Validation(
                    "Integer mood field can only validate a Integer mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::IntegerRange {minimum, maximum} => {
            match value {
                CustomFieldEntryType::IntegerRange {low, high} => {
                    verify_range_bound(low, high, minimum, maximum)
                },
                _ => Err(error::Error::Validation(
                    "IntegerRange mood field can only validate a IntergerRange mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::Float {minimum, maximum, step: _, precision: _} => {
            match value {
                CustomFieldEntryType::Float {value} => {
                    verify_range(value, minimum, maximum)
                },
                _ => Err(error::Error::Validation(
                    "Float mood field can only validate a Float mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::FloatRange {minimum, maximum, step: _, precision: _} => {
            match value {
                CustomFieldEntryType::FloatRange {low, high} => {
                    verify_range_bound(low, high, minimum, maximum)
                },
                _ => Err(error::Error::Validation(
                    "FloatRange mood field can only validate a FloatRange mood entry".to_owned()
                ))
            }
        },
        CustomFieldType::Time {as_12hr: _} => {
            match value {
                CustomFieldEntryType::Time {value: _} => Ok(()),
                _ => Err(error::Error::Validation(
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
                _ => Err(error::Error::Validation(
                    "TimeRange mood field can only validate a TimeRange mood entry".to_owned()
                ))
            }
        }
    }
}