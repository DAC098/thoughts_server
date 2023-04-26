use crate::common;

#[test]
fn ping_server() {
    let mut url = common::get_base_url();
    let client = common::create_client_blocking();

    url.set_path("/ping");

    let res = common::result::expect_with_err(
        client.get(url)
            .send(),
        "failed to send ping request"
    );

    assert!(res.status().is_success(), "failed ping request");

    let body = common::result::expect_with_err(
        res.text(),
        "failed to retrieve plaintext body"
    );

    assert_eq!(body, "pong".to_owned(), "ping response is not pong");
}
