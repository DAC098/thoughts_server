use std::path::Path;

use actix_web::{web, App, HttpServer};
use actix_web::middleware::{Logger};
use actix_session::{CookieSession};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tokio_postgres::{Config as PGConfig, NoTls};
use bb8_postgres::{PostgresConnectionManager, bb8};

use tlib::{cli, config};

mod error;
mod security;
mod state;
mod handler;
mod response;
mod request;
mod json;
mod parsing;
mod util;
mod email;
mod getters;
mod template;

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

fn app_runner() -> Result<i32> {
    let mut conf_files: Vec<std::path::PathBuf> = Vec::new();
    let mut args = std::env::args();
    args.next();

    loop {
        let arg = match args.next() {
            Some(a) => a,
            None => break
        };

        if let Some(arg_substring) = cli::get_cli_option(&arg)? {
            if arg_substring == "log-debug" {
                std::env::set_var("RUST_LOG", "debug");
            } else if arg_substring == "log-info" {
                std::env::set_var("RUST_LOG", "info");
            } else if arg_substring == "backtrace" {
                std::env::set_var("RUST_BACKTRACE", "full");
            } else {
                return Err(cli::error::Error::UnknownArg(arg_substring.to_owned()).into());
            }
        } else {
            conf_files.push(cli::file_from_arg(&arg)?);
        }
    }

    env_logger::init();

    let conf = config::load_server_config(conf_files)?;

    config::validate_server_config(&conf)?;
    
    let result = actix_web::rt::System::new().block_on(server_runner(conf));

    log::info!("server shutdown");

    result
}

async fn server_runner(config: config::ServerConfig) -> Result<i32> {
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
    let file_serving_config = config.file_serving;

    let db_state_ref = web::Data::new(state::db::DBState::new(
        bb8::Pool::builder().build(
            PostgresConnectionManager::new(db_config, NoTls)
        ).await?
    ));
    let template_state_ref = web::Data::new(state::template::TemplateState::new(
        template::get_built_registry(config.template)?
    ));
    let email_state_ref = web::Data::new(state::email::EmailState::new(
        config.email
    ));
    let server_info_state_ref = web::Data::new(state::server_info::ServerInfoState::new(
        config.info
    ));
    let storage_state_ref = web::Data::new(state::storage::StorageState::new(
        config.storage
    )?);

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .error_handler(handler::handle_json_error)
            )
            .app_data(db_state_ref.clone())
            .app_data(template_state_ref.clone())
            .app_data(email_state_ref.clone())
            .app_data(server_info_state_ref.clone())
            .app_data(storage_state_ref.clone())
            .wrap(Logger::new("%a XF-%{X-Forwarded-For}i:%{X-Forwarded-Port}i %t \"%r\" %s %b \"%{Referer}i\" %T"))
            .wrap(
                CookieSession::signed(&[0; 32])
                    .secure(true)
                    .domain(session_domain.clone())
                    .name("thoughts_session")
                    .path("/")
            )

            .route("/ping", web::get().to(handler::ping::handle_get))

            .route("/", web::get().to(handler::handle_get))
            .route("/account", web::get().to(handler::account::handle_get))
            .route("/account", web::put().to(handler::account::handle_put))
            .service(
                web::scope("/admin")
                    .route("/users", web::get().to(handler::admin::users::handle_get))
                    .route("/users", web::post().to(handler::admin::users::handle_post))
                    .route("/users/{user_id}", web::get().to(handler::admin::users::user_id::handle_get))
                    .route("/users/{user_id}", web::put().to(handler::admin::users::user_id::handle_put))
                    .route("/users/{user_id}", web::delete().to(handler::admin::users::user_id::handle_delete))
            )
            .service(
                web::scope("/auth")
                    .route("/login", web::get().to(handler::auth::login::handle_get))
                    .route("/login", web::post().to(handler::auth::login::handle_post))
                    .route("/logout", web::post().to(handler::auth::logout::handle_post))
                    .route("/change", web::post().to(handler::auth::change::handle_post))
                    .route("/verify_email", web::get().to(handler::auth::verify_email::handle_get))
            )
            .route("/backup", web::get().to(handler::backup::handle_get))
            .route("/backup", web::post().to(handler::backup::handle_post))
            .service(
                web::scope("/custom_fields")
                    .route("", web::get().to(handler::custom_fields::handle_get))
                    .route("", web::post().to(handler::custom_fields::handle_post))
                    .service(
                        web::scope("/{field_id}")
                            .route("", web::get().to(handler::custom_fields::field_id::handle_get))
                            .route("", web::put().to(handler::custom_fields::field_id::handle_put))
                            .route("", web::delete().to(handler::custom_fields::field_id::handle_delete))
                    )
            )
            .route("/email", web::get().to(handler::email::handle_get))
            .service(
                web::scope("/entries")
                    .route("", web::get().to(handler::entries::handle_get))
                    .route("", web::post().to(handler::entries::handle_post))
                    .service(
                        web::scope("/{entry_id}")
                            .route("", web::get().to(handler::entries::entry_id::handle_get))
                            .route("", web::put().to(handler::entries::entry_id::handle_put))
                            .route("", web::delete().to(handler::entries::entry_id::handle_delete))
                            .service(
                                web::scope("/comments")
                                    .route("", web::get().to(handler::entries::entry_id::comments::handle_get))
                                    .route("", web::post().to(handler::entries::entry_id::comments::handle_post))
                                    .route("/{comment_id}", web::put().to(handler::entries::entry_id::comments::comment_id::handle_put))
                            )
                            .service(
                                web::scope("/audio")
                                    .route("", web::get().to(handler::entries::entry_id::audio::handle_get))
                                    .route("", web::post().to(handler::entries::entry_id::audio::handle_post))
                                    .service(
                                        web::scope("/{audio_id}")
                                            .route("", web::get().to(handler::entries::entry_id::audio::audio_id::handle_get))
                                            .route("", web::put().to(handler::entries::entry_id::audio::audio_id::handle_put))
                                    )
                            )
                    )
            )
            .service(
                web::scope("/global")
                    .service(
                        web::scope("/custom_fields")
                            .route("", web::get().to(handler::global::custom_fields::handle_get))
                            .route("", web::post().to(handler::global::custom_fields::handle_post))
                            .service(
                                web::scope("/{field_id}")
                                    .route("", web::get().to(handler::global::custom_fields::field_id::handle_get))
                                    .route("", web::put().to(handler::global::custom_fields::field_id::handle_put))
                                    .route("", web::delete().to(handler::global::custom_fields::field_id::handle_delete))
                            )
                    )
            )
            .route("/settings", web::get().to(handler::okay))
            .route("/settings", web::put().to(handler::okay))
            .service(
                web::scope("/tags")
                    .route("", web::get().to(handler::tags::handle_get))
                    .route("", web::post().to(handler::tags::handle_post))
                    .service(
                        web::scope("/{tag_id}")
                            .route("", web::get().to(handler::tags::tag_id::handle_get))
                            .route("", web::put().to(handler::tags::tag_id::handle_put))
                            .route("", web::delete().to(handler::tags::tag_id::handle_delete))
                    )
            )
            .service(
                web::scope("/users")
                    .route("", web::get().to(handler::users::handle_get))
                    .service(
                        web::scope("/{user_id}")
                            .route("", web::get().to(handler::users::user_id::handle_get))
                            .route("", web::put().to(handler::okay))
                            .service(
                                web::scope("/entries")
                                    .route("", web::get().to(handler::entries::handle_get))
                                    .service(
                                        web::scope("/{entry_id}")
                                            .route("", web::get().to(handler::entries::entry_id::handle_get))
                                            .service(
                                                web::scope("/comments")
                                                    .route("", web::get().to(handler::entries::entry_id::comments::handle_get))
                                                    .route("", web::post().to(handler::entries::entry_id::comments::handle_post))
                                                    .route("/{comment_id}", web::put().to(handler::entries::entry_id::comments::comment_id::handle_put))
                                            ).service(
                                                web::scope("/audio")
                                                    .route("", web::get().to(handler::entries::entry_id::audio::handle_get))
                                                    .route("/{audio_id}", web::get().to(handler::entries::entry_id::audio::audio_id::handle_get))
                                            )
                                    )
                            )
                            .service(
                                web::scope("/custom_fields")
                                    .route("", web::get().to(handler::custom_fields::handle_get))
                                    .route("/{field_id}", web::get().to(handler::custom_fields::field_id::handle_get))
                            )
                            .route("/tags", web::get().to(handler::tags::handle_get))
                    )
            )
            .service(
                actix_files::Files::new("/static", &file_serving_config.directory)
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
        let cert_file = ssl_config.cert.ok_or(AppError::SslError(format!("cert file not given")))?;
        let key_file = ssl_config.key.ok_or(AppError::SslError(format!("key file not given")))?;

        let key_path = Path::new(&key_file);
        let cert_path = Path::new(&cert_file);

        if !key_path.exists() {
            return Err(AppError::SslError(format!("key file given does not exist: {}", key_file)));
        }

        if !cert_path.exists() {
            return Err(AppError::SslError(format!("cert file given does not exist: {}", cert_file)));
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
