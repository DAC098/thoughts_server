use actix_web::{http, HttpRequest, HttpResponse};

pub mod json;

pub fn respond_index_html() -> HttpResponse {
    HttpResponse::Ok().body(r#"
<!DOCTYPE html>
<html>
    <head>
        <title>Thoughts</title>
        <link rel="icon" href="data:;base64,iVBORw0KGgo=">
        <link rel="stylesheet" href="https://static2.sharepointonline.com/files/fabric/office-ui-fabric-core/11.0.0/css/fabric.min.css">
        <script src="/static/runtime.b.js"></script>
        <script src="/static/main.b.js"></script>
        <style>
        * {
            box-sizing: border-box;
        }

        body {
            font-family: 'Roboto Mono', monospace;
            
            margin:0;
            padding:0;
        }
        </style>
        <script>
        function getJSON(path) {
            return fetch(path, {
                method: "GET",
                headers: {
                    "Accept": "application/json"
                },
                credentials: "same-origin"
            })
            .then(res => res.json())
            .catch(e => {console.error(e); return {};});
        }

        function postJSON(path, data) {
            return fetch(path, {
                method: "POST",
                headers: {
                    "Accept": "application/json",
                    "Content-Type": "application/json"
                },
                credentials: "same-origin",
                body: JSON.stringify(data)
            })
            .then(res => res.json())
            .catch(e => {console.error(e); return {};});
        }
        </script>
    </head>
    <body class="ms-Fabric" dir="ltr">
        <div id="render-root"></div>
    </body>
</html>
"#)
}

pub fn check_if_html(
    headers: &http::HeaderMap
) -> Result<bool, http::header::ToStrError> {
    let accept_opt = headers.get("accept");

    if let Some(accept_type) = accept_opt {
        let accept = accept_type.to_str()?;

        return Ok(accept.contains("text/html"));
    }

    return Ok(false);
}

pub fn check_if_html_req(
    req: &HttpRequest
) -> Result<bool, http::header::ToStrError> {
    check_if_html(req.headers())
}