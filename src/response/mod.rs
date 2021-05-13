use actix_web::{http, HttpRequest, HttpResponse};
use serde_json::{to_string};

pub mod json;

use crate::db::users;

pub fn respond_index_html(user_opt: Option<users::User>) -> HttpResponse {
    let user_json = match user_opt {
        Some(user) => match to_string(&user) {
            Ok(rtn) => rtn,
            Err(_) => "null".to_owned()
        },
        None => "null".to_owned()
    };

    HttpResponse::Ok().body(format!(r#"
<!DOCTYPE html>
<html>
    <head>
        <title>Thoughts</title>
        <link rel="icon" href="data:;base64,iVBORw0KGgo=">
        <link rel="stylesheet" href="https://static2.sharepointonline.com/files/fabric/office-ui-fabric-core/11.0.0/css/fabric.min.css">
        <script src="/static/runtime.b.js"></script>
        <script>
            const active_user = {}; 
            window.active_user = active_user;
        </script>
        <script src="/static/main.b.js"></script>
    </head>
    <body class="ms-Fabric" dir="ltr" style="margin: 0">
        <div id="render-root"></div>
    </body>
</html>
"#, user_json))
}

pub fn check_if_html(
    headers: &http::HeaderMap,
    ignore_err: bool
) -> Result<bool, http::header::ToStrError> {
    let accept_opt = headers.get("accept");

    if let Some(accept_type) = accept_opt {
        match accept_type.to_str() {
            Ok(accept) => Ok(accept.contains("text/html")),
            Err(e) => if ignore_err { Ok(false) } else { Err(e) }
        }
    } else {
        Ok(false)
    }
}

pub fn check_if_html_req(
    req: &HttpRequest,
    ignore_err: bool
) -> Result<bool, http::header::ToStrError> {
    check_if_html(req.headers(), ignore_err)
}

pub fn redirect_to_path(path: &str) -> HttpResponse {
    HttpResponse::Found().insert_header((http::header::LOCATION, path)).finish()
}