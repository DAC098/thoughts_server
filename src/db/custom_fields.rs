use tokio_postgres::{GenericClient};
use serde::{Serialize, Deserialize};

use crate::db::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomField {
    pub id: i32,
    pub name: String,
    pub owner: i32,
    pub config: CustomFieldType,
    pub order: i32,
    pub comment: Option<String>,
    pub issued_by: Option<i32>,
}

fn default_time_range_show_diff() -> bool {
    false
}

fn default_as_12hr() -> bool {
    false
}

fn default_step() -> f64 {
    0.01
}

fn default_precision() -> i32 {
    2
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
        maximum: Option<f32>,
        #[serde(default = "default_step")]
        step: f64,
        #[serde(default = "default_precision")]
        precision: i32
    },
    FloatRange {
        minimum: Option<f32>,
        maximum: Option<f32>,
        #[serde(default = "default_step")]
        step: f64,
        #[serde(default = "default_precision")]
        precision: i32
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

pub async fn find_from_id(
    conn: &impl GenericClient,
    id: &i32
) -> error::Result<Option<CustomField>> {
    let result = conn.query(
        "\
        select id, \
               name, \
               owner, \
               config, \
               comment, \
               \"order\", \
               issued_by \
        from custom_fields \
        where id = $1",
        &[id]
    ).await?;

    if result.len() == 0 {
        Ok(None)
    } else {
        Ok(Some(CustomField {
            id: result[0].get(0),
            name: result[0].get(1),
            owner: result[0].get(2),
            config: serde_json::from_value(result[0].get(3)).unwrap(),
            comment: result[0].get(4),
            order: result[0].get(5),
            issued_by: result[0].get(6)
        }))
    }
}

pub async fn find_from_owner(
    conn: &impl GenericClient,
    owner: &i32
) -> error::Result<Vec<CustomField>> {
    Ok(
        conn.query(
            "\
            select id, \
                   name, \
                   owner, \
                   config, \
                   comment, \
                   \"order\", \
                   issued_by \
            from custom_fields \
            where owner = $1 \
            order by \"order\", name",
            &[owner]
        )
        .await?
        .iter()
        .map(|row| CustomField {
            id: row.get(0),
            name: row.get(1),
            owner: row.get(2),
            config: serde_json::from_value(row.get(3)).unwrap(),
            comment: row.get(4),
            order: row.get(5),
            issued_by: row.get(6)
        })
        .collect()
    )
}

