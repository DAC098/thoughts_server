use actix_web::{web, http, HttpRequest, HttpResponse, Responder, error};

use crate::response;

pub async fn handle_get_users(req: HttpRequest) -> impl Responder {
    if let Ok(accept_html) = response::check_if_html_req(&req) {
        if accept_html {
            return response::respond_index_html();
        }
    }

    return HttpResponse::Ok().body("okay");
}

pub async fn handle_post_users(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("okay")
}

pub async fn handle_get_users_id(req: HttpRequest) -> impl Responder {
    if let Ok(accept_html) = response::check_if_html_req(&req) {
        if accept_html {
            return response::respond_index_html();
        }
    }

    return HttpResponse::Ok().body("okay");
}

pub async fn handle_put_users_id(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("okay")
}

pub async fn handle_delete_users_id(req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("okay")
}

pub async fn handle_get_dashboard() -> impl Responder {
    response::respond_index_html()
}