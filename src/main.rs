use std::path::Path;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::{Logger};
use actix_session::{CookieSession};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tokio_postgres::{Config as PGConfig, NoTls};
use bb8_postgres::{PostgresConnectionManager, bb8};

mod security;
mod error;
mod time;
mod config;
mod state;
mod db;
mod handler;
mod response;
mod request;
mod json;
mod parsing;
mod util;
mod email;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut config_files: Vec<std::path::PathBuf> = vec!();
    let mut args = std::env::args();
    args.next();

    while let Some(arg) = args.next() {
        if arg.starts_with("--") {
            if arg.len() <= 2 {
                println!("incomplete argument given");
                return Ok(());
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
                return Ok(());
            }
        } else {
            if let Ok(canonical_path) = std::fs::canonicalize(arg.clone()) {
                if !canonical_path.is_file() {
                    println!("specified configuration file is not a file. {:?}", canonical_path.into_os_string());
                    return Ok(());
                }
    
                config_files.push(canonical_path);
            } else {
                println!("failed to locate given file. {}", arg);
                return Ok(());
            }
        }
    }

    env_logger::init();

    let config_result = config::load_server_config(config_files);

    if config_result.is_err() {
        println!("failed to load server configuration\n{:?}", config_result.unwrap_err());
        return Ok(());
    }

    let config = config_result.unwrap();

    if config.bind.len() == 0 {
        println!("no bind interfaces specified");
        return Ok(());
    }

    let mut db_config = PGConfig::new();
    db_config.user(config.db.username.as_ref());
    db_config.password(config.db.password);
    db_config.host(config.db.hostname.as_ref());
    db_config.port(config.db.port);
    db_config.dbname(config.db.database.as_ref());

    if config.email.enable {
        if config.email.username.is_none() || config.email.password.is_none() {
            println!("username and password must be given if email is enabled");
            return Ok(());
        }

        if config.email.from.is_none() {
            println!("from email address must be given if email is enabled");
            return Ok(());
        } else {
            if !email::valid_email_address(config.email.from.as_ref().unwrap()) {
                println!("from email address is invalid");
                return Ok(());
            }
        }

        if config.email.relay.is_none() {
            println!("relay must be given if email is enabled");
            return Ok(());
        }
    }

    let info_config = config.info;
    let email_config = config.email;
    let session_domain = config.session.domain;
    let manager = PostgresConnectionManager::new(db_config, NoTls);
    let pool = match bb8::Pool::builder().build(manager).await {
        Ok(p) => p,
        Err(e) => panic!("failed to create database connection pool. error: {}", e)
    };
    let mut static_dir = std::env::current_dir()?;
    static_dir.push("static");

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(
                web::JsonConfig::default()
                .error_handler(handler::handle_json_error)
            )
            .data(state::AppState::new(&pool, &email_config, &info_config))
            .wrap(Logger::new("%a XF-%{X-Forwarded-For}i:%{X-Forwarded-Port}i %t \"%r\" %s %b \"%{Referer}i\" %T"))
            .wrap(
                CookieSession::signed(&[0; 32])
                    .secure(true)
                    .domain(&session_domain.clone())
                    .name("thoughts_session")
                    .path("/")
            )
            .route("/", web::get().to(handler::handle_get))
            .route("/email", web::get().to(handler::email::handle_get))
            .route("/auth/login", web::get().to(handler::auth::login::handle_get))
            .route("/auth/login", web::post().to(handler::auth::login::handle_post))
            .route("/auth/logout", web::post().to(handler::auth::logout::handle_post))
            .route("/auth/change", web::post().to(handler::auth::change::handle_post))
            .route("/auth/verify_email", web::get().to(handler::auth::verify_email::handle_get))
            .route("/admin/users", web::get().to(handler::admin::users::handle_get))
            .route("/admin/users", web::post().to(handler::admin::users::handle_post))
            .route("/admin/users/{user_id}", web::get().to(handler::admin::users::user_id::handle_get))
            .route("/admin/users/{user_id}", web::put().to(handler::admin::users::user_id::handle_put))
            .route("/admin/users/{user_id}", web::delete().to(handler::admin::users::user_id::handle_delete))
            .route("/entries", web::get().to(handler::entries::handle_get))
            .route("/entries", web::post().to(handler::entries::handle_post))
            .route("/entries/{entry_id}", web::get().to(handler::entries::entry_id::handle_get))
            .route("/entries/{entry_id}", web::put().to(handler::entries::entry_id::handle_put))
            .route("/entries/{entry_id}", web::delete().to(handler::entries::entry_id::handle_delete))
            .route("/custom_fields", web::get().to(handler::custom_fields::handle_get))
            .route("/custom_fields", web::post().to(handler::custom_fields::handle_post))
            .route("/custom_fields/{field_id}", web::get().to(handler::custom_fields::field_id::handle_get))
            .route("/custom_fields/{field_id}", web::put().to(handler::custom_fields::field_id::handle_put))
            .route("/custom_fields/{field_id}", web::delete().to(handler::custom_fields::field_id::handle_delete))
            .route("/tags", web::get().to(handler::tags::handle_get))
            .route("/tags", web::post().to(handler::tags::handle_post))
            .route("/tags/{tag_id}", web::get().to(handler::tags::tag_id::handle_get))
            .route("/tags/{tag_id}", web::put().to(handler::tags::tag_id::handle_put))
            .route("/tags/{tag_id}", web::delete().to(handler::tags::tag_id::handle_delete))
            .route("/users", web::get().to(handler::users::handle_get))
            .route(
                "/users/{user_id}",
                web::get().to(handler::users::user_id::handle_get)
            )
            .route(
                "/users/{user_id}",
                web::put().to(handler::okay)
            )
            .route(
                "/users/{user_id}/entries",
                web::get().to(handler::users::user_id::entries::handle_get)
            )
            .route(
                "/users/{user_id}/entries/{entry_id}",
                web::get().to(handler::users::user_id::entries::entry_id::handle_get)
            )
            .route(
                "/users/{user_id}/custom_fields",
                web::get().to(handler::users::user_id::custom_fields::handle_get)
            )
            .route(
                "/users/{user_id}/custom_fields/{field_id}",
                web::get().to(handler::users::user_id::custom_fields::field_id::handle_get)
            )
            .route(
                "/users/{user_id}/tags",
                web::get().to(handler::users::user_id::tags::handle_get)
            )
            .route("/account", web::get().to(handler::account::handle_get))
            .route("/account", web::put().to(handler::account::handle_put))
            .route("/settings", web::get().to(handler::okay))
            .route("/settings", web::put().to(handler::okay))
            .route("/backup", web::get().to(handler::backup::handle_get))
            .route("/backup", web::post().to(handler::backup::handle_post))
            .service(
                actix_files::Files::new("/static", &static_dir)
                    .show_files_listing()
                    .redirect_to_slash_directory()
            )
    })
        .backlog(config.backlog)
        .max_connections(config.max_connections)
        .max_connection_rate(config.max_connection_rate);

    let mut run_ssl = false;
    let mut cert_file = String::from("");
    let mut key_file = String::from("");

    if config.key.is_some() && config.cert.is_some() {
        cert_file = config.cert.unwrap();
        key_file = config.key.unwrap();

        let key_path = Path::new(&key_file);
        let cert_path = Path::new(&cert_file);

        if !key_path.exists() {
            panic!("key file given does not exist: {}", key_file);
        }

        if !cert_path.exists() {
            panic!("cert file given does not exist: {}", cert_file);
        }

        run_ssl = true;
    }

    for interface in config.bind.iter() {
        let bind_value = format!("{}:{}", interface.host, interface.port);
        let bind_check;

        if run_ssl {
            let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            ssl_builder.set_private_key_file(key_file.clone(), SslFiletype::PEM).unwrap();
            ssl_builder.set_certificate_chain_file(cert_file.clone()).unwrap();
            bind_check = server.bind_openssl(bind_value.clone(), ssl_builder);
        } else {
            bind_check = server.bind(bind_value.clone());
        }

        server = match bind_check {
            Ok(s) => s,
            Err(e) => panic!("failed to bind interface: {}\n{:?}", bind_value, e)
        };
    }

    log::info!("server listening for requests");

    if let Err(e) = server.workers(config.threads).run().await {
        log::error!("server error: {}", e);
    }

    Ok(())
}
