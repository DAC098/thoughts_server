use reqwest::{StatusCode, cookie::Jar};
use serde::Serialize;

use crate::common;

#[derive(Serialize)]
pub struct LoginBody {
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(tag = "method")]
pub enum VerifyBody {
    Totp {
        value: String,
    },
    TotpHash {
        value: String,
    }
}

#[test]
fn login() {
    let mut url = common::get_base_url();
    let client = common::create_client_blocking();

    url.set_path("/auth/session");

    let login = LoginBody {
        username: String::from("password_only"),
        password: String::from("password_only"),
    };

    let res = common::result::expect_with_err(
        client.post(url)
            .json(&login)
            .send(),
        "failed to send post request to server"
    );

    if res.status() != StatusCode::OK {
        let json: serde_json::Value = res.json()
            .expect("password login failed. unknown response body");

        panic!("password login failed.\n{:#?}", json);
    }
}

#[test]
fn login_with_totp() {
    let secret: [u8; 25] = [0x76, 0x1F, 0x52, 0xA3, 0x97, 0x65, 0x41, 0x37, 0x26, 0x76, 0x6D, 0xC2, 0x1A, 0x5B, 0x9D, 0x1F, 0x77, 0x1C, 0x8B, 0xC8, 0x82, 0x01, 0xBD, 0x59, 0x9C];
    let digits = 6;
    let step = 30;

    let cookie_jar = std::sync::Arc::new(Jar::default());
    let mut url = common::get_base_url();
    let client = common::create_cookie_client_blocking(cookie_jar.clone());

    url.set_path("/auth/session");

    let login = LoginBody {
        username: String::from("password_totp"),
        password: String::from("password_totp"),
    };

    let res = common::result::expect_with_err(
        client.post(url)
            .json(&login)
            .send(),
        "failed to send post request to server"
    );

    if res.status() != StatusCode::UNAUTHORIZED {
        let json: serde_json::Value = res.json()
            .expect("password login failed. unknown response body");

        panic!("password login failed.\n{:#?}", json);
    }

    let json: serde_json::Value = common::result::expect_with_err(
        res.json(),
        "unknown response body"
    );

    let Some(map) = json.as_object() else {
        panic!("unexpected json value. {:#?}", json);
    };

    let error = common::json::get_string("error", &map);

    if error != "VerifySession" {
        panic!("unexpected err type. {:#?}", error);
    }

    let data = common::json::get_object("data", &map);
    let method = common::json::get_string("method", &data);

    if method != "Totp" {
        panic!("unexpected verify type. {:#?}", method);
    }

    let Some(now) = common::unix_epoch_sec() else {
        panic!("failed to get current unix timestamp");
    };

    let mut url = common::get_base_url();
    let client = common::create_cookie_client_blocking(cookie_jar.clone());

    url.set_path("/auth/session/verify");

    let verify = VerifyBody::Totp {
        value: rust_otp::totp(&rust_otp::Algo::SHA1, secret, digits, step, now)
    };

    let res = common::result::expect_with_err(
        client.post(url)
            .json(&verify)
            .send(),
        "failed to send verify post request to server"
    );

    if res.status() != StatusCode::OK {
        let json: serde_json::Value = res.json()
            .expect("totp verify failed. unknown response body");

        panic!("totp verify failed.\n{:#?}", json);
    }
}

#[test]
fn login_with_totp_recovery() {
    let recovery_codes = vec![
        "6D5B2RWB",
        "QNFYC4IZ",
        "OIVQPTB7",
        "2PRDASIW",
        "TCEJSDCH",
        "KPVM6ZRF",
        "4K7Z4VAV",
        "GF2SZU6J",
        "AEW3GUZH",
        "Q3MWCURK"
    ];

    let cookie_jar = std::sync::Arc::new(Jar::default());
    let mut url = common::get_base_url();
    let client = common::create_cookie_client_blocking(cookie_jar.clone());

    url.set_path("/auth/session");

    let login = LoginBody {
        username: String::from("password_totp_recovery"),
        password: String::from("password_totp_recovery"),
    };

    let res = common::result::expect_with_err(
        client.post(url)
            .json(&login)
            .send(),
        "failed to send post request to server"
    );

    if res.status() != StatusCode::UNAUTHORIZED {
        let json: serde_json::Value = res.json()
            .expect("password login failed. unknown response body");

        panic!("password login failed.\n{:#?}", json);
    }

    let json: serde_json::Value = common::result::expect_with_err(
        res.json(),
        "unknown response body"
    );

    let Some(root) = json.as_object() else {
        panic!("unexpect json value: {:#?}", json);
    };

    let error = common::json::get_string("error", &root);

    if error != "VerifySession" {
        panic!("unexpected error type. {:#?}", error);
    }

    let data = common::json::get_object("data", &root);
    let method = common::json::get_string("method", &data);

    if method != "Totp" {
        panic!("unexpected verify type. {:#?}", method);
    }

    let mut url = common::get_base_url();
    let base_client = common::create_cookie_client_blocking(cookie_jar.clone());

    url.set_path("/auth/session/verify");

    for code in recovery_codes {
        let totp_hash = VerifyBody::TotpHash {
            value: code.to_owned(),
        };

        let res = common::result::expect_with_err(
            base_client.clone()
                .post(url.clone())
                .json(&totp_hash)
                .send(),
            "failed to send verify post request to server"
        );

        if res.status() == StatusCode::UNAUTHORIZED {
            let json: serde_json::Value = res.json()
                .expect("hash verify failed. unknown response body");

            let Some(root) = json.as_object() else {
                panic!("expected root value: {:#?}", json);
            };

            let error = common::json::get_string("error", &root);

            if error != "TotpHashInvalid" {
                panic!("unexpected error trype: {:#?}", error);
            }
        } else if res.status() == StatusCode::OK {
            return;
        } else {
            let json: serde_json::Value = res.json()
                .expect("hash verify failed. unknown response body");

            panic!("hash verify failed.\n{:#?}", json);
        }
    }

    panic!("hash verify failed. all codes rejected");
}

