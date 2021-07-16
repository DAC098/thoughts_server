use std::fmt::{Write};
use std::collections::{HashSet};

use postgres::{Client, Transaction, NoTls};
use chrono::{DateTime, Local, Utc, Duration};
use rand::{Rng, SeedableRng};
use rand::rngs::{SmallRng};
use lipsum;

mod cli;
mod config;
mod db;
mod error;

use error::{Result, AppError};
use db::custom_fields::{CustomFieldType};
use db::custom_field_entries::{CustomFieldEntryType};

type RtnResult<T> = std::result::Result<T, AppError>;

fn main() {
    std::process::exit(match make_test_data() {
        Ok(code) => code,
        Err(err) => {
            println!("{}", err);

            err.get_code()
        }
    })
}

fn make_test_data() -> Result {
    let server_config = config::load_server_config(cli::init_from_cli()?)?;
    config::validate_server_config(&server_config)?;

    let mut db_config = Client::configure();
    db_config.user(server_config.db.username.as_ref());
    db_config.password(server_config.db.password);
    db_config.host(server_config.db.hostname.as_ref());
    db_config.port(server_config.db.port);
    db_config.dbname(server_config.db.database.as_ref());

    let mut client = db_config.connect(NoTls)?;
    let mut transaction = client.transaction()?;
    let search = transaction.query(
        r#"select id from users where username = 'admin'"#,
        &[]
    )?;

    if search.is_empty() {
        println!("no admin user found");
        transaction.rollback()?;
    } else {
        let total_days: i64 = 365 * 10;
        let mut small_rng = SmallRng::from_entropy();
        let start_date = Local::today().and_hms(0, 0, 0) - Duration::days(total_days);
        let owner = search[0].get(0);

        delete_current_data(&mut transaction, &owner)?;
        
        let tags = make_tags(&mut transaction, &mut small_rng, &owner)?;
        let fields = make_custom_fields(&mut transaction, &owner)?;

        make_entries(&mut transaction, &mut small_rng, &owner, total_days, &start_date, &tags, &fields)?;

        transaction.commit()?;
    }

    Ok(0)
}

fn delete_current_data(
    conn: &mut Transaction,
    owner: &i32
) -> RtnResult<()> {
    conn.execute(
        r#"
        delete from text_entries where entry in (
            select id from entries where owner = $1
        )
        "#,
        &[&owner]
    )?;
    conn.execute(
        r#"
        delete from custom_field_entries where entry in (
            select id from entries where owner = $1
        )
        "#, 
        &[&owner]
    )?;
    conn.execute(
        r#"
        delete from entries2tags where entry in (
            select id from entries where owner = $1
        )
        "#, 
        &[&owner]
    )?;
    conn.execute(
        r#"
        delete from entry_markers where entry in (
            select id from entries where owner = $1
        )
        "#, 
        &[&owner]
    )?;
    conn.execute(
        r#"
        delete from entries where owner = $1
        "#, 
        &[&owner]
    )?;
    conn.execute(
        r#"
        delete from tags where owner = $1
        "#, 
        &[&owner]
    )?;
    conn.execute(
        r#"
        delete from custom_fields where owner = $1
        "#, 
        &[&owner]
    )?;

    Ok(())
}

fn make_custom_fields(
    conn: &mut Transaction, 
    owner: &i32
) -> RtnResult<Vec<db::custom_fields::CustomField>> {
    let mut rtn: Vec<db::custom_fields::CustomField> = Vec::with_capacity(6);
    
    // make integer field
    {
        let value = serde_json::to_value(
            db::custom_fields::CustomFieldType::Integer {
                minimum: None,
                maximum: None
            }
        ).unwrap();

        let result = conn.query_one(
            r#"
            insert into custom_fields (name, owner, config) values
            ('integer', $1, $2)
            returning id
            "#, 
            &[&owner, &value]
        )?;

        rtn.push(db::custom_fields::CustomField {
            id: result.get(0),
            name: "integer".to_owned(),
            owner: owner.clone(),
            config: db::custom_fields::CustomFieldType::Integer {
                minimum: None,
                maximum: None
            }
        });
    }

    // make integer range field
    {
        let value = serde_json::to_value(
            db::custom_fields::CustomFieldType::IntegerRange {
                minimum: Some(1),
                maximum: Some(10)
            }
        ).unwrap();

        let result = conn.query_one(
            r#"
            insert into custom_fields (name, owner, config) values
            ('integer_range', $1, $2)
            returning id
            "#, 
            &[&owner, &value]
        )?;

        rtn.push(db::custom_fields::CustomField {
            id: result.get(0),
            name: "integer_range".to_owned(),
            owner: owner.clone(),
            config: db::custom_fields::CustomFieldType::IntegerRange {
                minimum: Some(1),
                maximum: Some(10)
            }
        });
    }

    // make float field
    {
        let value = serde_json::to_value(
            db::custom_fields::CustomFieldType::Float {
                minimum: Some(0.0),
                maximum: Some(50.0),
                step: 1.25,
                precision: 2
            }
        ).unwrap();

        let result = conn.query_one(
            r#"
            insert into custom_fields (name, owner, config) values
            ('float', $1, $2)
            returning id
            "#, 
            &[&owner, &value]
        )?;

        rtn.push(db::custom_fields::CustomField {
            id: result.get(0),
            name: "float".to_owned(),
            owner: owner.clone(),
            config: db::custom_fields::CustomFieldType::Float {
                minimum: Some(0.0),
                maximum: Some(50.0),
                step: 1.25,
                precision: 2
            }
        });
    }

    // make float range field
    {
        let value = serde_json::to_value(
            db::custom_fields::CustomFieldType::FloatRange {
                minimum: Some(0.0),
                maximum: None,
                step: 1.125,
                precision: 3
            }
        ).unwrap();

        let result = conn.query_one(
            r#"
            insert into custom_fields (name, owner, config) values
            ('float_range', $1, $2)
            returning id
            "#, 
            &[&owner, &value]
        )?;

        rtn.push(db::custom_fields::CustomField {
            id: result.get(0),
            name: "float_range".to_owned(),
            owner: owner.clone(),
            config: db::custom_fields::CustomFieldType::FloatRange {
                minimum: Some(0.0),
                maximum: None,
                step: 1.125,
                precision: 3
            }
        })
    }

    // make timee field
    {
        let value = serde_json::to_value(
            db::custom_fields::CustomFieldType::Time {
                as_12hr: false
            }
        ).unwrap();

        let result = conn.query_one(
            r#"
            insert into custom_fields (name, owner, config) values
            ('time', $1, $2)
            returning id
            "#, 
            &[&owner, &value]
        )?;

        rtn.push(db::custom_fields::CustomField {
            id: result.get(0),
            name: "time".to_owned(),
            owner: owner.clone(),
            config: db::custom_fields::CustomFieldType::Time {
                as_12hr: false
            }
        });
    }

    // make time range field
    {
        let value = serde_json::to_value(
            db::custom_fields::CustomFieldType::TimeRange {
                show_diff: true,
                as_12hr: false
            }
        ).unwrap();

        let result = conn.query_one(
            r#"
            insert into custom_fields (name, owner, config) values
            ('time_rnage', $1, $2)
            returning id
            "#,
            &[&owner, &value]
        )?;

        rtn.push(db::custom_fields::CustomField {
            id: result.get(0),
            name: "time_range".to_owned(),
            owner: owner.clone(),
            config: db::custom_fields::CustomFieldType::TimeRange {
                show_diff: true,
                as_12hr: false
            }
        });
    }

    Ok(rtn)
}

fn make_tags(conn: &mut Transaction, rng: &mut SmallRng, owner: &i32) -> RtnResult<Vec<i32>> {
    let words = vec!(
        "happy", "sad", "party", 
        "bday", "appointment", "hike", 
        "read", "board_games", "video_games",
        "sex", "pub", "family", 
        "night_out", "date",
        "movies", "theatre", "concert"
    );
    let mut rtn: Vec<i32> = Vec::with_capacity(words.len());

    // make tags with random colors
    for word in words {
        let mut values = [0u8; 3];
        rng.fill(&mut values);

        let mut color = String::with_capacity(7);
        write!(color, "#").unwrap();

        for v in &values {
            write!(color, "{:02x}", v).unwrap();
        }

        let result = conn.query_one(
            r#"
            insert into tags (title, owner, color) values
            ($1, $2, $3)
            returning id
            "#,
            &[&word, owner, &color]
        )?;

        rtn.push(result.get(0));
    }

    Ok(rtn)
}

fn make_entries(
    conn: &mut Transaction,
    rng: &mut SmallRng,
    owner: &i32, 
    amount: i64, 
    start_day: &DateTime<Local>, 
    tags: &Vec<i32>, 
    fields: &Vec<db::custom_fields::CustomField>
) -> RtnResult<()> {
    for n in 0..amount {
        let day = *start_day + Duration::days(n);
        let result = conn.query_one(
            r#"
            insert into entries (day, owner) values
            ($1, $2)
            returning id
            "#,
            &[&day, owner]
        )?;

        let entry_id: i32 = result.get(0);

        make_custom_field_entries(conn, rng, &entry_id, &day, fields)?;
        make_text_entries(conn, rng, &entry_id)?;
        make_tag_entries(conn, rng, &entry_id, tags)?;
    }

    Ok(())
}

fn make_custom_field_entry_value(
    config: &CustomFieldType,
    rng: &mut SmallRng,
    day: &DateTime<Local>
) -> db::custom_field_entries::CustomFieldEntryType {
    match &*config {
        CustomFieldType::Integer{minimum, maximum} => {
            CustomFieldEntryType::Integer {
                value: rng.gen_range(minimum.unwrap_or(0)..maximum.unwrap_or(100))
            }
        },
        CustomFieldType::IntegerRange{minimum, maximum} => {
            let min = minimum.unwrap_or(0);
            let max = maximum.unwrap_or(100);
            let mid: i32 = (max - min) / 2;

            CustomFieldEntryType::IntegerRange {
                low: rng.gen_range(min..mid),
                high: rng.gen_range(mid..max)
            }
        },
        CustomFieldType::Float{minimum, maximum, step: _, precision: _} => {
            CustomFieldEntryType::Float {
                value: rng.gen_range(minimum.unwrap_or(0.0)..maximum.unwrap_or(100.0))
            }
        },
        CustomFieldType::FloatRange{minimum, maximum, step: _, precision: _} => {
            let min = minimum.unwrap_or(0.0);
            let max = maximum.unwrap_or(100.0);
            let mid: f32 = (max - min) / 2.0f32;

            CustomFieldEntryType::FloatRange {
                low: rng.gen_range(min..mid),
                high: rng.gen_range(mid..max)
            }
        },
        CustomFieldType::Time{as_12hr: _} => {
            let value = DateTime::from(
                day.date().and_hms(
                    rng.gen_range(0..23), 
                    rng.gen_range(0..60),
                    rng.gen_range(0..60)
                )
            );

            CustomFieldEntryType::Time {value}
        },
        CustomFieldType::TimeRange{as_12hr: _, show_diff: _} => {
            let low: DateTime<Utc> = DateTime::from(
                day.date().and_hms(
                    rng.gen_range(0..21), 
                    rng.gen_range(0..60),
                    rng.gen_range(0..60)
                )
            );
            let hours = Duration::hours(rng.gen_range(4..10));
            let minutes = Duration::minutes(rng.gen_range(0..60));
            let seconds = Duration::seconds(rng.gen_range(0..60));
            let high: DateTime<Utc> = low + hours + minutes + seconds;

            CustomFieldEntryType::TimeRange {low, high}
        }
    }
}

fn make_custom_field_entries(
    conn: &mut Transaction,
    rng: &mut SmallRng,
    entry_id: &i32,
    day: &DateTime<Local>,
    fields: &Vec<db::custom_fields::CustomField>
) -> RtnResult<()> {
    for field in fields {
        let value = serde_json::to_value(
            make_custom_field_entry_value(&field.config, rng, day)
        ).unwrap();

        conn.execute(
            r#"
            insert into custom_field_entries (field, value, entry) values
            ($1, $2, $3)
            "#,
            &[&field.id, &value, entry_id]
        )?;
    }

    Ok(())
}

fn make_text_entries(
    conn: &mut Transaction, 
    rng: &mut SmallRng,
    entry_id: &i32
) -> RtnResult<()> {
    let total: u32 = rng.gen_range(0..3);

    for _ in 0..total {
        let text = lipsum::lipsum(rng.gen_range(1..100));
        let private = rng.gen_bool(1.0 / 5.0);

        conn.execute(
            r#"
            insert into text_entries (thought, private, entry) values
            ($1, $2, $3)
            "#,
            &[&text, &private, entry_id]
        )?;
    }

    Ok(())
}

fn make_tag_entries(
    conn: &mut Transaction, 
    rng: &mut SmallRng,
    entry_id: &i32, 
    tags: &Vec<i32>
) -> RtnResult<()> {
    let total: usize = rng.gen_range(0..3);
    let mut assigned: HashSet<i32> = HashSet::with_capacity(total);

    for _ in 0..total {
        let index: usize = rng.gen_range(0..tags.len());

        if !assigned.insert(tags[index]) {
            continue;
        }

        conn.execute(
            r#"
            insert into entries2tags (tag, entry) values
            ($1, $2)
            "#, 
            &[&tags[index], entry_id]
        )?;
    }

    Ok(())
}