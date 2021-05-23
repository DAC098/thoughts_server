use actix_web::{web, http, HttpRequest, Responder};
use actix_session::{Session};

use crate::error;
use crate::request::from;
use crate::response;
use crate::state;
use crate::json;
use crate::parsing::url_paths;
use crate::security;

pub async fn handle_get(
    req: HttpRequest,
    session: Session,
    app: web::Data<state::AppState>,
    path: web::Path<url_paths::UserFieldPath>
) -> error::Result<impl Responder> {
    let accept_html = response::check_if_html_req(&req, true)?;
    let conn = &*app.get_conn().await?;
    let initiator_opt = from::get_initiator(conn, &session).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(Some(initiator_opt.unwrap().user)))
        } else {
            let redirect = format!("/auth/login?jump_to=/users/{}/custom_fields/{}", path.user_id, path.field_id);
            Ok(response::redirect_to_path(redirect.as_str()))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        security::assert::permission_to_read(conn, initiator.user.get_id(), path.user_id).await?;

        if let Some(field) = json::search_custom_field(conn, path.field_id).await? {
            if field.owner != path.user_id {
                Err(error::ResponseError::PermissionDenied(
                    format!("this user does not own the requested field. user[{}] field[{}]", path.user_id, path.field_id)
                ))
            } else {
                Ok(response::json::respond_json(
                    http::StatusCode::OK,
                    response::json::MessageDataJSON::<Option<()>>::build(
                        "successful",
                        None
                    )
                ))
            }
        } else {
            Err(error::ResponseError::CustomFieldNotFound(path.field_id))
        }
    }
}