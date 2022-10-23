use actix_web::{http, HttpRequest, Responder};
use serde::Serialize;

pub mod user_id;

use crate::request::initiator_from_request;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::state;

#[derive(Serialize)]
pub struct UserJson {
    id: i32,
    username: String,
    ability: String
}

#[derive(Serialize)]
pub struct UserListJson {
    given: Vec<UserJson>,
    allowed: Vec<UserJson>
}

pub async fn handle_get(
    req: HttpRequest,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let initiator_opt = initiator_from_request(conn, &req).await?;

    if accept_html {
        if initiator_opt.is_some() {
            Ok(response::respond_index_html(&template.into_inner(), Some(initiator_opt.unwrap().user))?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    } else if initiator_opt.is_none() {
        Err(error::ResponseError::Session)
    } else {
        let initiator = initiator_opt.unwrap();
        let allowed_result = conn.query(
            "\
            select user_access.owner as id, \
                   users.username as username, \
                   user_access.ability as ability \
            from user_access \
            join users on user_access.owner = users.id \
            where user_access.allowed_for = $1 \
            order by user_access.owner",
            &[&initiator.user.id]
        ).await?;
        let mut allowed = Vec::<UserJson>::with_capacity(allowed_result.len());

        for user in allowed_result {
            allowed.push(UserJson {
                id: user.get(0),
                username: user.get(2),
                ability: user.get(3)
            });
        }

        let given_result = conn.query(
            "\
            select user_access.allowed_for as id, \
                   users.username as username, \
                   user_access.ability as ability \
            from user_access \
            join users on user_access.allowed_for = users.id \
            where user_access.owner = $1 \
            order by user_access.allowed_for",
            &[&initiator.user.id]
        ).await?;
        let mut given = Vec::<UserJson>::with_capacity(given_result.len());

        for user in given_result {
            given.push(UserJson {
                id: user.get(0),
                username: user.get(2),
                ability: user.get(3)
            });
        }

        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(UserListJson {given, allowed}))
    }
}