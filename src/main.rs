use std::path::Path;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::{Logger};
use actix_session::{CookieSession};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tokio_postgres::{Config as PGConfig, NoTls};
use bb8_postgres::{PostgresConnectionManager, bb8};
use log::{warn, info, error};
use env_logger;

mod error;
mod time;
mod config;
mod state;
mod db;
mod handler;
mod response;
mod request;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "full");
    env_logger::init();

    let config_file = String::from("./server_config.json");
    let config_check = config::load_server_config(config_file);

    if config_check.is_err() {
        println!("failed to load config file\n{:?}", config_check.unwrap_err());
        return Ok(());
    }

    let config = config_check.unwrap();

    if config.host.len() == 0 {
        println!("no hosts specified");
        return Ok(());
    }

    let mut db_config = PGConfig::new();
    db_config.user(config.db.username.as_ref());
    db_config.password(config.db.password);
    db_config.host(config.db.hostname.as_ref());
    db_config.port(config.db.port);
    db_config.dbname("thoughts");

    let manager = PostgresConnectionManager::new(db_config, NoTls);
    let pool_result = bb8::Pool::builder().build(manager).await;

    if pool_result.is_err() {
        panic!("failed to create database connection pool. error: {}", pool_result.unwrap_err());
    }

    let pool = pool_result.unwrap();
    let session_domain = config.session_domain.unwrap_or("".to_owned());
    let mut static_dir = std::env::current_dir()?;
    static_dir.push("static");

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(
                web::JsonConfig::default()
                .error_handler(handler::handle_json_error)
            )
            .data(state::AppState::new(&pool))
            .wrap(Logger::new("%a %t \"%r\" %s %b \"%{Referer}i\" %T"))
            .wrap(
                CookieSession::signed(&[0; 32])
                    .secure(true)
                    .domain(&session_domain.clone())
                    .name("thoughts_session")
                    .path("/")
            )
            // .route("/static/{file_path:.*}", web::get().to(handler::handle_get_static))
            .route("/", web::get().to(handler::handle_get_root))
            .route("/auth/login", web::get().to(handler::auth::handle_get_auth_login))
            .route("/auth/login", web::post().to(handler::auth::handle_post_auth_login))
            .route("/auth/create", web::post().to(handler::auth::handle_post_auth_create))
            .route("/entries", web::get().to(handler::entries::handle_get_entries))
            .route("/entries", web::post().to(handler::entries::handle_post_entries))
            .route("/entries/{entry_id}", web::get().to(handler::entries::handle_get_entries_id))
            .route("/entries/{entry_id}", web::put().to(handler::entries::handle_put_entries_id))
            .route("/entries/{entry_id}", web::delete().to(handler::entries::handle_delete_entries_id))
            .route("/entries/{entry_id}/mood_entries", web::get().to(handler::entries::handle_get_entries_id_mood_entries))
            .route("/entries/{entry_id}/mood_entries", web::post().to(handler::entries::handle_post_entries_id_mood_entries))
            .route("/entries/{entry_id}/mood_entries/{mood_id}", web::put().to(handler::entries::handle_put_entries_id_mood_entries_id))
            .route("/entries/{entry_id}/mood_entries/{mood_id}", web::delete().to(handler::entries::handle_delete_entries_id_mood_entries_id))
            .route("/entries/{entry_id}/text_entries", web::get().to(handler::entries::handle_get_entries_id_text_entries))
            .route("/entries/{entry_id}/text_entries", web::post().to(handler::entries::handle_post_entries_id_text_entries))
            .route("/entries/{entry_id}/text_entries/{text_id}", web::put().to(handler::entries::handle_put_entries_id_text_entries_id))
            .route("/entries/{entry_id}/text_entries/{text_id}", web::delete().to(handler::entries::handle_delete_entries_id_text_entries_id))
            .route("/mood_entries", web::get().to(handler::mood_entries::handle_get_mood_entries))
            .route("/text_entries", web::get().to(handler::text_entries::handle_get_text_entries))
            .route("/mood_fields", web::get().to(handler::mood_fields::handle_get_mood_fields))
            .route("/mood_fields", web::post().to(handler::mood_fields::handle_post_mood_fields))
            .route("/mood_fields/{field_id}", web::put().to(handler::mood_fields::handle_put_mood_fields_id))
            .route("/mood_fields/{field_id}", web::delete().to(handler::mood_fields::handle_delete_mood_fields_id))
            .route("/users", web::get().to(handler::users::handle_get_users))
            .route("/users", web::post().to(handler::users::handle_post_users))
            .route("/users/{user_id}", web::get().to(handler::users::handle_get_users_id))
            .route("/users/{user_id}", web::put().to(handler::users::handle_put_users_id))
            .route("/users/{user_id}", web::delete().to(handler::users::handle_delete_users_id))
            .route("/dashboard", web::get().to(handler::handle_get_dashboard))
            .service(
                actix_files::Files::new("/static", &static_dir)
                .show_files_listing()
                .redirect_to_slash_directory()
            )
    });

    let mut run_ssl = false;
    let mut cert_file = String::from("");
    let mut key_file = String::from("");

    if config.key.is_some() && config.cert.is_some() {
        cert_file = config.cert.unwrap();
        key_file = config.key.unwrap();

        let key_path = Path::new(&key_file);
        let cert_path = Path::new(&cert_file);

        if !key_path.exists() {
            warn!("key file given does not exist: {}", key_file);
            return Ok(());
        }

        if !cert_path.exists() {
            warn!("cert file given does not exist: {}", cert_file);
            return Ok(());
        }

        run_ssl = true;
    }

    for host in config.host.iter() {
        let bind_value = format!("{}:{}", host, config.port);
        let bind_check;

        if run_ssl {
            let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            ssl_builder.set_private_key_file(key_file.clone(), SslFiletype::PEM).unwrap();
            ssl_builder.set_certificate_chain_file(cert_file.clone()).unwrap();
            bind_check = server.bind_openssl(bind_value.clone(), ssl_builder);
        } else {
            bind_check = server.bind(bind_value.clone());
        }

        if bind_check.is_err() {
            warn!("failed to bind interface: {}", bind_value);
            return Ok(());
        } else {
            info!("bound to interface: {}", bind_value);
            server = bind_check.unwrap();
        }
    }

    let runner = server.run();
    info!("server listening for requests");
    let run_result = runner.await;

    if run_result.is_err() {
        error!("server error: {}", run_result.unwrap_err());
    }

    return Ok(());
}
