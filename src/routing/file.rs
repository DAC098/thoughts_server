//! deals with static file serving

use std::path::PathBuf;
use std::time::Instant;

use actix_files::NamedFile;
use actix_web::http::Method;
use actix_web::{http, HttpRequest, Responder};

use crate::net::http::{error, response::json::JsonBuilder};
use crate::state;

/// handles static file serving
///
/// GET /*
///
/// this will be a catch all while also handling static file serving. anything
/// other than a GET http request will respond with METHOD_NOT_ALLOWED.
pub async fn handle_file_serving(
    req: HttpRequest,
    file_serving: state::WebFileServingState
) -> error::Result<impl Responder> {
    if req.method() != Method::GET {
        return JsonBuilder::new(http::StatusCode::METHOD_NOT_ALLOWED)
            .set_error("MethodNotAllowed")
            .set_message("requested method is not accepted by this resource")
            .build_empty()
    }

    let start_time = Instant::now();
    let mut should_cache = false;
    let mut from_cache = false;
    let lookup = req.uri().path();
    let mut to_send: Option<PathBuf> = None;

    if let Some(file_path) = file_serving.files.get(lookup) {
        to_send = Some(file_path.clone());
    } else if let Some(cached) = file_serving.check_cache(lookup).await {
        to_send = Some(cached);
        from_cache = true;
    } else {
        for (key, path) in file_serving.directories.iter() {
            if let Some(stripped) = lookup.strip_prefix(key.as_str()) {
                let mut sanitize = String::with_capacity(stripped.len());
                let mut first = true;

                for value in stripped.split("/") {
                    if value == ".." || value == "." || value.len() == 0 {
                        return JsonBuilder::new(http::StatusCode::BAD_REQUEST)
                            .set_error("MalformedResourcePath")
                            .set_message("resource path given contains invalid segments. \"..\", \".\", and \"\" are not allowed in the path")
                            .build_empty()
                    }

                    if first {
                        first = false;
                    } else {
                        sanitize.push('/');
                    }

                    sanitize.push_str(value);
                }

                let mut file_path = path.clone();
                file_path.push(sanitize);

                to_send = Some(file_path);
                break;
            }
        }

        should_cache = true;
    }

    if let Some(file_path) = to_send {
        if log::log_enabled!(log::Level::Debug) {
            let elapsed = start_time.elapsed();

            log::debug!(
                "static file serving lookup\nrequested path: {}\nfound: {:#?}\ntime: {}:{:06}\nfrom cache: {}",
                lookup,
                file_path,
                elapsed.as_secs(),
                elapsed.subsec_micros(),
                from_cache
            );
        }

        if file_path.exists() && should_cache {
            file_serving.cache_file(lookup, file_path.clone()).await;
        }

        Ok(NamedFile::open_async(file_path)
            .await?
            .into_response(&req))
    } else {
        JsonBuilder::new(http::StatusCode::NOT_FOUND)
            .set_error("NotFound")
            .set_message("the requested resource was not found")
            .build_empty()
    }
}
