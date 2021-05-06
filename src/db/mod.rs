pub mod mood_entries;
pub mod mood_fields;
pub mod users;
pub mod user_sessions;

/*
example db query
    match client.query("select * from entries", &[]).await {
        Ok(rows) => {
            println!("total records: {}", rows.len());

            for (index, record) in rows.iter().enumerate() {
                let mut print_str = String::new();

                write!(&mut print_str, "record: {}\n", index);

                for (index, col) in record.columns().iter().enumerate() {
                    write!(&mut print_str, "    {}:", col.name());
                    let col_type = col.type_().to_string();

                    if col_type == "integer" || col_type == "int4" {
                        let v: i32 = record.get(index);
                        write!(&mut print_str, " {}\n", v);
                    } else if col_type == "date" {
                        let v: chrono::NaiveDate = record.get(index);
                        write!(&mut print_str, " {}\n", v);
                    } else {
                        println!("unknown column type: {}", col_type);
                    }
                }

                println!("{}", print_str);
            }
        }
        Err(e) => {
            println!("error when running query {}", e);
        }
    }
 */