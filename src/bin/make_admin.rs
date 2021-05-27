use postgres::{Client, NoTls};

#[path = "../config/mod.rs"]
mod config;

fn main() {
    let mut config_files: Vec<std::path::PathBuf> = vec!();
    let mut args = std::env::args();
    args.next();

    while let Some(arg) = args.next() {
        if arg.starts_with("--") {
            if arg.len() <= 2 {
                println!("incomplete argument given");
                return;
            }

            let (_, arg_substring) = arg.split_at(2);

            if arg_substring == "debug" {
                std::env::set_var("RUST_LOG", "debug");
            } else if arg_substring == "backtrace" {
                std::env::set_var("RUST_BACKTRACE", "full");
            } else if arg_substring == "info" {
                std::env::set_var("RUST_LOG", "info");
            } else {
                println!("unknown argument given. {}", arg_substring);
                return;
            }
        } else {
            if let Ok(canonical_path) = std::fs::canonicalize(arg.clone()) {
                if !canonical_path.is_file() {
                    println!("specified configuration file is not a file. {:?}", canonical_path.into_os_string());
                    return;
                }
    
                config_files.push(canonical_path);
            } else {
                println!("failed to locate given file. {}", arg);
                return;
            }
        }
    }

    env_logger::init();

    let config_check = config::load_server_config(config_files);

    if config_check.is_err() {
        panic!("failed to load server configuration\n{:?}", config_check.unwrap_err());
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
        let argon2_config = argon2::Config {
            variant: argon2::Variant::Argon2i,
            version: argon2::Version::Version13,
            mem_cost: 65536,
            time_cost: 10,
            lanes: 4,
            thread_mode: argon2::ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: 32
        };
        let mut openssl_salt: [u8; 64] = [0; 64];

        if let Err(e) = openssl::rand::rand_bytes(&mut openssl_salt) {
            panic!("error generating hash salt\n{:?}", e);
        };

        let hash = match argon2::hash_encoded(&password.as_bytes(), &openssl_salt, &argon2_config) {
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