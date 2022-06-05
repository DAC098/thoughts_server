use actix_web::{web, http, HttpRequest, Responder};

use tlib::db;

use crate::request::initiator_from_request;
use crate::response;
use crate::response::json::JsonBuilder;
use crate::state;
use crate::parsing::url_paths;
use crate::security;

use response::error;

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    path: web::Path<url_paths::UserPath>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            let redirect = format!("/auth/login?jump_to=/users/{}", path.user_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        security::assert::permission_to_read(conn, &initiator.user.id, &path.user_id).await?;

        if let Some(user) = db::users::find_from_id(conn, &path.user_id).await? {
            JsonBuilder::new(http::StatusCode::OK)
                .build(Some(user))
        } else {
            Err(error::ResponseError::UserIDNotFound(path.user_id))
        }
    }
}