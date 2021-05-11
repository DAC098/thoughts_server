use postgres::{Client, NoTls};

#[path = "../config/mod.rs"]
mod config;
#[path = "../time.rs"]
mod time;
#[path = "../db/mod.rs"]
mod db;
#[path = "../response/mod.rs"]
mod response;
#[path = "../error/mod.rs"]
mod error;
#[path = "../security/mod.rs"]
mod security;

fn main() {
    let mut config_files: Vec::<&std::path::Path> = Vec::with_capacity(2);
    let config_file = std::path::Path::new("./server_config.json");
    let config_override_file = std::path::Path::new("./server_config.override.json");
    
    if config_file.exists() {
        config_files.push(config_file);
    }

    if config_override_file.exists() {
        config_files.push(config_override_file);
    }

    let config_check = config::load_server_config(config_files);

    if config_check.is_err() {
        panic!("failed to load config file\n{:?}", config_check.unwrap_err());
    }

    let server_config = config_check.unwrap();

    let mut db_config = Client::configure();
    db_config.user(server_config.db.username.as_ref());
    db_config.password(server_config.db.password);
    db_config.host(server_config.db.hostname.as_ref());
    db_config.port(server_config.db.port);
    db_config.dbname(server_config.db.database.as_ref());

    let mut client = match db_config.connect(NoTls) {
        Ok(con) => con,
        Err(e) => panic!("failed to connect to database\n{:?}", e)
    };

    let result = match client.execute("select id from users where username = 'admin'", &[]) {
        Ok(r) => r,
        Err(e) => panic!("failed to determine if admin user exists in database\n{:?}", e)
    };

    if result == 0 {
        let level: i32 = 1;
        let username = "admin".to_owned();
        let password = "password".to_owned();
        let email = "admin@example.com".to_owned();
        let hash = match security::generate_new_hash(&password) {
            Ok(h) => h,
            Err(e) => panic!("failed to generate hash for default password\n{:?}", e)
        };

        let _insert_result = match client.execute(
            "insert into users (level, username, hash, email) values ($1, $2, $3, $4)",
            &[&level, &username, &hash, &email]
        ) {
            Ok(r) => r,
            Err(e) => panic!("failed to insert admin record into database\n{:?}", e)
        };

        println!("inserted admin record into database. admin password \"{}\"", password);
    } else {
        println!("admin user account already exists");
    }
}