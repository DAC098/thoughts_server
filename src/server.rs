use std::path::Path;
use actix_web::{web, App, HttpServer};
use actix_web::middleware::{Logger};
use actix_session::{CookieSession};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tokio_postgres::{Config as PGConfig, NoTls};
use bb8_postgres::{PostgresConnectionManager, bb8};

mod cli;
mod config;
mod error;
mod security;
mod state;
mod db;
mod handler;
mod response;
mod request;
mod json;
mod parsing;
mod util;
mod email;

use error::{AppError, Result};

fn main() {
    std::process::exit(match app_runner() {
        Ok(code) => code,
        Err(err) => {
            println!("{}", err);

            err.get_code()
        }
    });
}

fn app_runner() -> Result {
    let config = config::load_server_config(cli::init_from_cli()?)?;

    config::validate_server_config(&config)?;

    let system_runner = actix_web::rt::System::new();
    
    let result = system_runner.block_on(server_runner(config));

    log::info!("server shutdown");

    result
}

async fn server_runner(config: config::ServerConfig) -> Result {
    let db_config = {
        let mut rtn = PGConfig::new();
        rtn.user(config.db.username.as_ref());
        rtn.password(config.db.password);
        rtn.host(config.db.hostname.as_ref());
        rtn.port(config.db.port);
        rtn.dbname(config.db.database.as_ref());
        rtn
    };

    let session_domain = config.session.domain;
    let ssl_config = config.ssl;
    let bind_config = config.bind;

    let static_dir = {
        let mut rtn = std::env::current_dir()?;
        rtn.push("static");
        rtn
    };
    let app_state = state::AppState::new(
        bb8::Pool::builder().build(
            PostgresConnectionManager::new(db_config, NoTls)
        ).await?,
        config.email,
        config.info
    );

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(handler::handle_json_error)
            )
            .data(app_state.clone())
            .wrap(Logger::new("%a XF-%{X-Forwarded-For}i:%{X-Forwarded-Port}i %t \"%r\" %s %b \"%{Referer}i\" %T"))
            .wrap(
                CookieSession::signed(&[0; 32])
                    .secure(true)
                    .domain(session_domain.clone())
                    .name("thoughts_session")
                    .path("/")
            )
            .route("/", web::get().to(handler::handle_get))
            .route("/account", web::get().to(handler::account::handle_get))
            .route("/account", web::put().to(handler::account::handle_put))
            .route("/admin/users", web::get().to(handler::admin::users::handle_get))
            .route("/admin/users", web::post().to(handler::admin::users::handle_post))
            .route("/admin/users/{user_id}", web::get().to(handler::admin::users::user_id::handle_get))
            .route("/admin/users/{user_id}", web::put().to(handler::admin::users::user_id::handle_put))
            .route("/admin/users/{user_id}", web::delete().to(handler::admin::users::user_id::handle_delete))
            .route("/auth/login", web::get().to(handler::auth::login::handle_get))
            .route("/auth/login", web::post().to(handler::auth::login::handle_post))
            .route("/auth/logout", web::post().to(handler::auth::logout::handle_post))
            .route("/auth/change", web::post().to(handler::auth::change::handle_post))
            .route("/auth/verify_email", web::get().to(handler::auth::verify_email::handle_get))
            .route("/backup", web::get().to(handler::backup::handle_get))
            .route("/backup", web::post().to(handler::backup::handle_post))
            .route("/custom_fields", web::get().to(handler::custom_fields::handle_get))
            .route("/custom_fields", web::post().to(handler::custom_fields::handle_post))
            .route("/custom_fields/{field_id}", web::get().to(handler::custom_fields::field_id::handle_get))
            .route("/custom_fields/{field_id}", web::put().to(handler::custom_fields::field_id::handle_put))
            .route("/custom_fields/{field_id}", web::delete().to(handler::custom_fields::field_id::handle_delete))
            .route("/email", web::get().to(handler::email::handle_get))
            .route("/entries", web::get().to(handler::entries::handle_get))
            .route("/entries", web::post().to(handler::entries::handle_post))
            .route("/entries/{entry_id}", web::get().to(handler::entries::entry_id::handle_get))
            .route("/entries/{entry_id}", web::put().to(handler::entries::entry_id::handle_put))
            .route("/entries/{entry_id}", web::delete().to(handler::entries::entry_id::handle_delete))
            .route("/settings", web::get().to(handler::okay))
            .route("/settings", web::put().to(handler::okay))
            .route("/tags", web::get().to(handler::tags::handle_get))
            .route("/tags", web::post().to(handler::tags::handle_post))
            .route("/tags/{tag_id}", web::get().to(handler::tags::tag_id::handle_get))
            .route("/tags/{tag_id}", web::put().to(handler::tags::tag_id::handle_put))
            .route("/tags/{tag_id}", web::delete().to(handler::tags::tag_id::handle_delete))
            .route("/users", web::get().to(handler::users::handle_get))
            .route("/users/{user_id}", web::get().to(handler::users::user_id::handle_get))
            .route("/users/{user_id}", web::put().to(handler::okay))
            .route("/users/{user_id}/entries", web::get().to(handler::users::user_id::entries::handle_get))
            .route("/users/{user_id}/entries/{entry_id}", web::get().to(handler::users::user_id::entries::entry_id::handle_get))
            .route("/users/{user_id}/custom_fields", web::get().to(handler::users::user_id::custom_fields::handle_get))
            .route("/users/{user_id}/custom_fields/{field_id}", web::get().to(handler::users::user_id::custom_fields::field_id::handle_get))
            .route("/users/{user_id}/tags", web::get().to(handler::users::user_id::tags::handle_get))
            .service(
                actix_files::Files::new("/static", &static_dir)
                    .show_files_listing()
                    .redirect_to_slash_directory()
            )
            .default_service(
                web::route().to(handler::handle_not_found)
            )
    })
        .backlog(config.backlog)
        .max_connections(config.max_connections)
        .max_connection_rate(config.max_connection_rate);

    let bind_iter = bind_config.iter().map(
        |interface| format!("{}:{}", interface.host, interface.port)
    );

    if ssl_config.enable {
        let cert_file = ssl_config.cert.ok_or(
            AppError::SslError(format!("cert file not given"))
        )?;
        let key_file = ssl_config.key.ok_or(
            AppError::SslError(format!("key file not given"))
        )?;

        let key_path = Path::new(&key_file);
        let cert_path = Path::new(&cert_file);

        if !key_path.exists() {
            return Err(AppError::SslError(
                format!("key file given does not exist: {}", key_file)
            ));
        }

        if !cert_path.exists() {
            return Err(AppError::SslError(
                format!("cert file given does not exist: {}", cert_file)
            ));
        }

        for bind_value in bind_iter {
            let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            ssl_builder.set_private_key_file(key_file.clone(), SslFiletype::PEM).unwrap();
            ssl_builder.set_certificate_chain_file(cert_file.clone()).unwrap();
            server = server.bind_openssl(bind_value, ssl_builder)?;
        }
    } else {
        for bind_value in bind_iter {
            server = server.bind(bind_value)?;
        }
    }

    log::info!("server listening for requests");

    server.workers(config.threads).run().await?;

    Ok(0)
}
