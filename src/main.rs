use actix_web::{web, App, HttpServer};
use actix_web::middleware::Logger;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tokio_postgres::{Config as PGConfig, NoTls};
use bb8_postgres::{PostgresConnectionManager, bb8};

use tlib::cli;

mod error;
mod config;
mod security;
mod state;
mod db;
mod handler;
mod response;
mod request;
mod parsing;
mod util;
mod email;
mod getters;
mod template;

use error::Result;

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

    log::debug!("conf: {:#?}", conf);

    let result = actix_web::rt::System::new()
        .block_on(server_runner(conf));

    log::info!("server shutdown");

    result
}

async fn server_runner(config: config::ServerConfig) -> Result<i32> {
    let db_config = {
        let mut rtn = PGConfig::new();
        rtn.user(&config.db.username);
        rtn.password(config.db.password);
        rtn.host(&config.db.hostname);
        rtn.port(config.db.port);
        rtn.dbname(&config.db.database);
        rtn
    };

    let session_domain = config.session.domain;
    let bind_config = config.bind;

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
    let file_serving_ref = web::Data::new(state::file_serving::FileServingState::from(
        config.file_serving
    ));

    let mut server = HttpServer::new(move || {
        App::new()
            .app_data(
                web::JsonConfig::default()
                    .content_type_required(true)
                    .content_type(request::is_json_mime)
                    .error_handler(handler::handle_json_error)
            )
            .app_data(db_state_ref.clone())
            .app_data(template_state_ref.clone())
            .app_data(email_state_ref.clone())
            .app_data(server_info_state_ref.clone())
            .app_data(storage_state_ref.clone())
            .app_data(file_serving_ref.clone())
            .wrap(Logger::new("%a XF-%{X-Forwarded-For}i:%{X-Forwarded-Port}i %t \"%r\" %s %b \"%{Referer}i\" %T"))

            .route("/ping", web::get().to(handler::ping::handle_get))
            .route("/", web::get().to(handler::handle_get))
            .route("/account", web::get().to(handler::account::handle_get))
            .route("/account", web::put().to(handler::account::handle_put))
            .service(
                web::scope("/admin")
                    .service(
                        web::scope("/users")
                            .route("", web::get().to(handler::admin::users::handle_get))
                            .route("", web::post().to(handler::admin::users::handle_post))
                            .service(
                                web::scope("/{user_id}")
                                    .route("", web::get().to(handler::admin::users::user_id::handle_get))
                                    .route("", web::put().to(handler::admin::users::user_id::handle_put))
                                    .route("", web::delete().to(handler::admin::users::user_id::handle_delete))
                            )
                    )
            )
            .service(
                web::scope("/auth")
                    .route("/login", web::get().to(handler::auth::session::handle_get))
                    .route("/login", web::post().to(handler::auth::session::handle_post))
                    .route("/logout", web::post().to(handler::auth::session::handle_delete))
                    .service(
                        web::scope("/session")
                            .route("", web::get().to(handler::auth::session::handle_get))
                            .route("", web::post().to(handler::auth::session::handle_post))
                            .route("", web::delete().to(handler::auth::session::handle_delete))
                    )
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
            .default_service(
                web::route().to(handler::handle_file_serving)
            )
    })
        .backlog(config.backlog)
        .max_connections(config.max_connections)
        .max_connection_rate(config.max_connection_rate);

    for (_key, info) in bind_config {
        if let Some(ssl) = info.ssl {
            let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            ssl_builder.set_private_key_file(ssl.key, SslFiletype::PEM).unwrap();
            ssl_builder.set_certificate_chain_file(ssl.cert).unwrap();

            log::info!("attaching secure listener to: {}", info.addr);
            server = server.bind_openssl(info.addr, ssl_builder)?;
        } else {
            log::info!("attaching listener to: {}", info.addr);
            server = server.bind(info.addr)?;
        }
    }

    log::info!("server listening for requests");

    server.workers(config.threads).run().await?;

    Ok(0)
}
