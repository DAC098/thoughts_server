use actix_web::{web, http, HttpRequest, Responder};
use serde::Deserialize;
use serde::Serialize;
use futures::{stream::FuturesUnordered, StreamExt};
use lettre::message::Mailbox;

pub mod user_id;

use crate::db::query::QueryParams;
use crate::security::Initiator;
use crate::security::initiator;
use crate::net::http::error;
use crate::net::http::response;
use crate::net::http::response::json::JsonBuilder;
use crate::security;
use crate::state;
use crate::db;
use crate::email;
use crate::util;

#[derive(Serialize, Eq)]
pub struct UserJson {
    id: i32,
    username: String,
    email: Option<String>
}

impl std::cmp::PartialEq for UserJson {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl std::cmp::PartialOrd for UserJson {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for UserJson {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.username.cmp(&other.username)
    }
}

#[derive(Serialize)]
pub struct UserListJson {
    given: Vec<UserJson>,
    allowed: Vec<UserJson>
}

pub async fn handle_get(
    req: HttpRequest,
    security: state::WebSecurityState,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
) -> error::Result<impl Responder> {
    let accept_html = response::try_check_if_html_req(&req);
    let conn = &*db.get_conn().await?;
    let lookup = initiator::from_request(&security, conn, &req).await?;

    if accept_html {
        return if lookup.is_some() {
            Ok(response::respond_index_html(
                &template.into_inner(),
                Some(lookup.unwrap().user)
            )?)
        } else {
            Ok(response::redirect_to_login(&req))
        }
    }

    let initiator = lookup.try_into()?;

    // first we will check to see if they just have a sweeping ability to
    // view all users. eg root admin
    let view_all = conn.execute(
        "\
        with user_groups as (\
            select group_id \
            from group_users \
            where users_id = $1\
        ) \
        select id \
        from permissions \
        where roll = 'users' and \
                (ability = 'r' or ability = 'rw') and \
                resource_table is null and \
                resource_id is null and \
                (\
                    (subject_table = 'groups' and subject_id in (select group_id from user_groups)) or \
                    (subject_table = 'users' and subject_id = $1)\
                )",
        &[&initiator.user.id]
    ).await?;

    // this process is probably not the most efficient especially since
    // it will send back all results and not do any paging to send back
    // smaller portions

    if view_all > 0 {
        let users_rows = conn.query(
            "\
            select id, \
                    username, \
                    email \
            from users \
            where id != $1 \
            order by username",
            &[&initiator.user.id]
        ).await?;
        let mut rtn: Vec<UserJson> = Vec::with_capacity(users_rows.len());

        for row in users_rows {
            rtn.push(UserJson {
                id: row.get(0),
                username: row.get(1),
                email: row.get(2)
            });
        }

        return JsonBuilder::new(http::StatusCode::OK)
            .build(Some(rtn))
    }

    let mut failed = false;
    let mut rtn: Vec<UserJson> = Vec::new();
    let mut params = QueryParams::with_capacity(1);
    params.push(&initiator.user.id);

    {
        // life times will complain if outbound_queries is in a different
        // scope for the params slice
        let mut outbound_queries = FuturesUnordered::new();

        // we need to collect all users that this user has access to
        // directly or through the groups that they are attached to and not
        // duplicate retrieved users

        // user to user access, any users that the requester has direct 
        // access to
        outbound_queries.push(conn.query(
            "\
            select users.id, \
                    users.username, \
                    users.email \
            from users \
                join permissions user_to_user_permissions on (\
                    user_to_user_permissions.subject_table = 'users' and \
                    user_to_user_permissions.subject_id = $1 and \
                    user_to_user_permissions.roll = 'users' and \
                    (user_to_user_permissions.ability = 'r' or user_to_user_permissions.ability = 'rw') and \
                    users.id = user_to_user_permissions.resource_id\
                ) \
            where users.id != $1",
            params.slice()
        ));

        // user to groups access, any groups that the requester has been 
        // explicitly assigned
        outbound_queries.push(conn.query(
            "\
            select distinct on (users.id) users.id, \
                    users.username, \
                    users.email \
            from users \
                join group_users on users.id = group_users.users_id \
                join permissions user_to_group_permissions on (\
                    user_to_group_permissions.subject_table = 'users' and \
                    user_to_group_permissions.subject_id = $1 and \
                    user_to_group_permissions.roll = 'users' and \
                    (user_to_group_permissions.ability = 'r' or user_to_group_permissions.ability = 'rw') and \
                    user_to_group_permissions.resource_table = 'groups' and \
                    group_users.group_id = user_to_group_permissions.resource_id\
                ) \
            where users.id != $1",
            params.slice()
        ));

        // groups to users, groups that the requester belongs to and the
        // users that can be directly accessed
        outbound_queries.push(conn.query(
            "\
            select distinct on (users.id) users.id, \
                    users.username, \
                    users.email \
            from users \
                join permissions group_to_user_permissions on (\
                    group_to_user_permissions.subject_table = 'groups' and \
                    group_to_user_permissions.subject_id in (select group_id from group_users where users_id = $1) and \
                    group_to_user_permissions.roll = 'users' and \
                    (group_to_user_permissions.ability = 'r' or group_to_user_permissions.ability = 'rw') and \
                    group_to_user_permissions.resource_table = 'users' and \
                    users.id = group_to_user_permissions.resource_id\
                ) \
            where users.id != $1",
            params.slice()
        ));

        // groups to groups, groups that the requester belongs to and
        // the groups they can access
        outbound_queries.push(conn.query(
            "\
            select distinct on (users.id) users.id, \
                    users.username, \
                    users.email \
            from users \
                join group_users on users.id = group_users.users_id \
                join permissions group_to_group_permissions on (\
                    group_to_group_permissions.subject_table = 'groups' and \
                    group_to_group_permissions.subject_id in (select group_id from group_users where users_id = $1) and \
                    group_to_group_permissions.roll = 'users' and \
                    (group_to_group_permissions.ability = 'r' or group_to_group_permissions.ability = 'rw') and \
                    group_to_group_permissions.resource_table = 'groups' and \
                    group_users.group_id = group_to_group_permissions.resource_id\
                ) \
            where users.id = $1",
            params.slice()
        ));

        // the order does not really matter since we are going to have to
        // sort the results anyway but it could be improved upon
        while let Some(results) = outbound_queries.next().await {
            // go through and collect the results but if we failed then
            // dont go through those results
            if failed {
                continue;
            }

            match results {
                Ok(rows) => {
                    // since we are not guarrenteed to use all records
                    // from the query we will reserve any capacity need on
                    // the chance that we use all the records
                    let avail = rtn.capacity() - rtn.len();

                    if rows.len() > avail {
                        rtn.reserve(rows.len() - avail);
                    }

                    for row in rows {
                        let rec = UserJson {
                            id: row.get(0),
                            username: row.get(1),
                            email: row.get(2)
                        };

                        // not sure how efficient this is
                        match rtn.binary_search(&rec) {
                            Ok(_p) => {}, // user already exists
                            Err(p) => {
                                rtn.insert(p, rec);
                            }
                        }
                    }
                },
                Err(err) => {
                    failed = true;
                    log::error!("query failed {:?}", err);
                }
            }
        }
    }

    if failed {
        JsonBuilder::new(http::StatusCode::INTERNAL_SERVER_ERROR)
            .set_error("DatabaseError")
            .set_message("there was a database error when processing your request")
            .build(None::<()>)
    } else {
        JsonBuilder::new(http::StatusCode::OK)
            .build(Some(rtn))
    }
}

#[derive(Deserialize)]
pub struct PostUserData {
    prefix: Option<String>,
    suffix: Option<String>,
    first_name: String,
    last_name: String,
    middle_name: Option<String>,
    dob: String
}

#[derive(Deserialize)]
pub struct PostUser {
    username: String,
    password: String,
    email: String,
    level: i32
}

#[derive(Deserialize)]
pub struct PostUserJson {
    user: PostUser,
    data: PostUserData
}

pub async fn handle_post(
    initiator: Initiator,
    db: state::WebDbState,
    template: state::WebTemplateState<'_>,
    email: state::WebEmailState,
    server_info: state::WebServerInfoState,
    posted: web::Json<PostUserJson>,
) -> error::Result<impl Responder> {
    let email = email.into_inner();
    let server_info = server_info.into_inner();
    let posted = posted.into_inner();
    let conn = &mut *db.get_conn().await?;

    if !security::permissions::has_permission(
        conn, 
        &initiator.user.id, 
        db::permissions::rolls::USERS, 
        &[
            db::permissions::abilities::READ_WRITE
        ],
        None
    ).await? {
        return Err(error::build::permission_denied(
            "you do not have permission to create new users"
        ))
    }

    let (found_username, found_email) = db::users::check_username_email(
        conn, &posted.user.username, &posted.user.email
    ).await?;

    if found_username {
        return Err(error::build::username_exists(posted.user.username.clone()));
    }

    if found_email {
        return Err(error::build::email_exists(posted.user.email.clone()))
    }

    let email_verified: bool = false;
    let mut email_value: Option<String> = None;
    let mut to_mailbox: Option<Mailbox> = None;

    if email.is_enabled() {
        to_mailbox = Some(Mailbox::new(None, email::parse_email_address(&posted.user.email)?));
        email_value = Some(posted.user.email);
    }

    let hash = security::generate_new_hash(&posted.user.password)?;
    let transaction = conn.transaction().await?;

    let user_result = transaction.query_one(
        "\
        insert into users (level, username, hash, email, email_verified) \
        values ($1, $2, $3, $4, $5) \
        returning id",
        &[
            &posted.user.level, 
            &posted.user.username, 
            &hash, 
            &email_value,
            &email_verified
        ]
    ).await?;

    let user_id = user_result.get(0);

    if email.is_enabled() {
        email::send_verify_email(
            &transaction, 
            &server_info, 
            &email, 
            &template, 
            &user_id, 
            to_mailbox.unwrap()
        ).await?;
    }

    let user_data = {
        let prefix = util::string::trimmed_optional_string(posted.data.prefix);
        let suffix = util::string::trimmed_optional_string(posted.data.suffix);
        let first_name = util::string::trimmed_string(posted.data.first_name);
        let last_name = util::string::trimmed_string(posted.data.last_name);
        let middle_name = util::string::trimmed_optional_string(posted.data.middle_name);
        let dob: chrono::NaiveDate;

        if let Ok(date) = posted.data.dob.parse() {
            dob = date;
        } else {
            let mut message = String::with_capacity(posted.data.dob.len() + 55);
            message.push_str("invalid date format given. format: YYYY-MM-DD given: \"");
            message.push_str(&posted.data.dob);
            message.push('"');

            return Err(error::build::validation(message))
        }

        transaction.execute(
            "\
            insert into user_data (owner, prefix, suffix, first_name, last_name, middle_name, dob) \
            values ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &user_id,
                &prefix,
                &suffix,
                &first_name,
                &last_name,
                &middle_name,
                &dob
            ]
        ).await?;

        db::user_data::UserData {
            owner: user_id,
            prefix, suffix,
            first_name, last_name, middle_name,
            dob
        }
    };

    transaction.commit().await?;

    JsonBuilder::new(http::StatusCode::OK)
        .set_message("created account")
        .build(Some(db::composed::ComposedFullUser {
            user: db::users::User {
                id: user_result.get(0),
                username: posted.user.username.clone(),
                email: email_value,
                email_verified: false,
                level: posted.user.level
            },
            data: user_data,
            access: Vec::new()
        }))
}