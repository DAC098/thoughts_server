use reqwest::blocking::Client;

#[test]
fn ping_server() {
    let res = Client::builder()
        .build()
        .expect("failed to create client request")
        .get("http://localhost:12345/ping")
        .send()
        .expect("failed to send request to test server");

    assert!(res.status().is_success(), "unsuccessful ping request");

    let body = res.text()
        .expect("failed to retrieve plaintext body");

    assert_eq!(body, "pong".to_owned(), "ping response is not pong");
}
