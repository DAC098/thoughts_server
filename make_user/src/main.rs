use postgres::{Client, NoTls};
use tlib::{config, cli, db};

fn main() {
    std::process::exit(
        match app() {
            Ok(code) => code,
            Err(err) => {
                let (code, msg) = err.into_tuple();
                println!("{}", msg);

                code
            }
        }
    )
}

pub fn app() -> cli::error::Result<i32> {
    let mut level: db::users::Level = db::users::Level::User;
    let mut email_set = false;
    let mut password = "password".to_owned();
    let mut email = "admin@example.com".to_owned();
    let mut username_arg: Option<String> = None;
    let mut config_files: Vec<std::path::PathBuf> = Vec::new();
    let mut args = std::env::args();
    args.next();

    loop {
        let arg = match args.next() {
            Some(a) => a,
            None => break
        };

        if let Some(arg_substring) = cli::get_cli_option(&arg)? {
            if arg_substring == "username" {
                let next_arg = cli::get_cli_option_value(&mut args, "username")?;

                if !email_set {
                    email = format!("{}@example.com", next_arg);
                }

                username_arg = Some(next_arg);
            } else if arg_substring == "password" {
                password = cli::get_cli_option_value(&mut args, "password")?;
            } else if arg_substring == "email" {
                email = cli::get_cli_option_value(&mut args, "email")?;
                email_set = true;
            } else if arg_substring == "level" {
                let next_arg = cli::get_cli_option_value(&mut args, "level")?;

                if next_arg.as_str() == "admin" {
                    level = db::users::Level::Admin;
                } else if next_arg.as_str() == "manager" {
                    level = db::users::Level::Manager;
                } else if next_arg.as_str() == "user" {
                    level = db::users::Level::User;
                } else {
                    return Err(cli::error::Error::InvalidArg(
                        format!("level can be admin, manager, or user. given: \"{}\"", next_arg)
                    ));
                }
            } else if arg_substring == "config" {
                let filename = cli::get_cli_option_value(&mut args, "config")?;
                config_files.push(cli::file_from_arg(&filename)?);
            } else if arg_substring == "log-debug" {
                std::env::set_var("RUST_LOG", "debug");
            } else if arg_substring == "log-info" {
                std::env::set_var("RUST_LOG", "info");
            } else if arg_substring == "backtrace" {
                std::env::set_var("RUST_BACKTRACE", "full");
            } else {
                return Err(cli::error::Error::UnknownArg(arg_substring.to_owned()))
            }
        } else {
            return Err(cli::error::Error::UnknownArg(arg));
        }
    }

    if username_arg.is_none() {
        return Err(cli::error::Error::MissingArg("username".to_owned()));
    }

    let username = username_arg.unwrap();

    let server_config = config::load_server_config(config_files).map_err(|err| cli::error::Error::General(err.get_msg()))?;
    config::validate_server_config(&server_config).map_err(|err| cli::error::Error::General(err.get_msg()))?;

    let mut db_config = Client::configure();
    db_config.user(server_config.db.username.as_ref());
    db_config.password(server_config.db.password);
    db_config.host(server_config.db.hostname.as_ref());
    db_config.port(server_config.db.port);
    db_config.dbname(server_config.db.database.as_ref());

    let mut client = db_config.connect(NoTls).map_err(
        |err| cli::error::Error::General(format!("failed to connect to database server\n{:?}", err))
    )?;

    let result = client.execute(
        "select id from users where username = $1", 
        &[&username]
    ).map_err(
        |err| cli::error::Error::General(format!("failed to determine if user exists in database\n{:?}", err))
    )?;

    if result == 0 {
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

        openssl::rand::rand_bytes(&mut openssl_salt).map_err(
            |err| cli::error::Error::General(format!("error generating hash salt\n{:?}", err))
        )?;

        let hash = argon2::hash_encoded(&password.as_bytes(), &openssl_salt, &argon2_config).map_err(
            |err| cli::error::Error::General(format!("failed to generate hash for default password\n{:?}", err))
        )?;

        let _insert_result = client.execute(
            "insert into users (level, username, hash, email) values ($1, $2, $3, $4)",
            &[&(level as i32), &username, &hash, &email]
        ).map_err(
            |err| cli::error::Error::General(format!("failed to insert admin record into database\n{:?}", err))
        )?;

        println!("inserted user record into database");
    } else {
        println!("user account already exists");
    }

    Ok(0)
}