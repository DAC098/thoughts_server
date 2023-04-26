use std::sync::Arc;

use reqwest::{Url, StatusCode, cookie::{Jar, CookieStore}, blocking::{Client, RequestBuilder}};
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub mod result {
    pub fn expect_with_err<T, E: std::fmt::Debug>(res: std::result::Result<T, E>, msg: &str) -> T {
        match res {
            Ok(rtn) => rtn,
            Err(err) => panic!("{}\n{:#?}", msg, err)
        }
    }
}

pub mod json {
    use core::hash::Hash;
    use core::borrow::Borrow;
    use std::fmt::Display;

    use serde_json::{Map, Value};

    pub fn get_object<'a, K>(key: &K, obj: &'a Map<String, Value>) -> &'a Map<String, Value>
    where
        String: Borrow<K> + Ord,
        K: ?Sized + Ord + Eq + Hash + Display,
    {
        let Some(v) = obj.get(key) else {
            panic!("missing key \"{}\" in object. {:#?}", key, obj);
        };

        let Some(m) = v.as_object() else {
            panic!("key \"{}\" is not an object. {:#?}", key, obj);
        };

        m
    }

    pub fn get_string<'a, K>(key: &K, obj: &'a Map<String, Value>) -> &'a str
    where
        String: Borrow<K> + Ord,
        K: ?Sized + Ord + Eq + Hash + Display
    {
        let Some(v) = obj.get(key) else {
            panic!("missing key \"{}\" in object. {:#?}", key, obj);
        };

        let Some(s) = v.as_str() else {
            panic!("key \"{}\" is not a string. {:#?}", key, obj);
        };

        s
    }
}

pub fn unix_epoch_sec() -> Option<u64> {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(d) => Some(d.as_secs()),
        Err(_err) => None,
    }
}

pub fn create_cookie_client_blocking<C>(store: Arc<C>) -> Client
where
    C: CookieStore + 'static
{
    Client::builder()
        .cookie_provider(store)
        .build()
        .expect("failed to create blocking client")
}

pub fn create_client_blocking() -> Client {
    Client::builder()
        .build()
        .expect("failed to create blocking client")
}

#[derive(Deserialize)]
pub struct TestArgs {
    host: Option<String>,
    port: Option<u16>,
}

pub fn get_base_url() -> Url {
    let mut path = result::expect_with_err(
        std::env::current_dir(),
        "failed to get current working directory"
    );

    path.push("test_args");
    path.set_extension("json");

    let args: TestArgs = if result::expect_with_err(path.try_exists(), "failed to check if file exists") {
        let file = result::expect_with_err(
            std::fs::OpenOptions::new()
                .read(true)
                .open(path),
            "failed to open test_args.json"
        );
        let reader = std::io::BufReader::new(file);

        result::expect_with_err(
            serde_json::from_reader(reader),
            "failed to parse json from test_args.json"
        )
    } else {
        TestArgs {
            host: None,
            port: None,
        }
    };

    let mut url = Url::parse("http://localhost/").unwrap();

    if let Some(host) = args.host {
        url.set_host(Some(host.as_str()))
            .expect("invalid host name for url");
    }

    if let Some(port) = args.port {
        url.set_port(Some(port)).unwrap();
    }

    url
}

#[derive(Serialize)]
pub struct PasswordLogin {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub enum VerifySession {
    Totp {
        value: String
    }
}

pub struct Totp {
    algo: Option<rust_otp::Algo>,
    secret: Vec<u8>,
    digits: Option<u32>,
    step: Option<u64>,
}

pub struct User {
    username: String,
    password: String,
    totp: Option<Totp>,
}

pub struct UserClient {
    client: Client,
    cookie_jar: Arc<Jar>,
    user: User,
    base_url: Url,
    active_session: bool,
}

impl UserClient {
    pub fn new(user: User, base_url: Url) -> UserClient {
        let cookie_jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("failed to create blocking client");

        UserClient {
            client,
            cookie_jar,
            user,
            base_url,
            active_session: false,
        }
    }

    fn handle_verify(&mut self, json: Value) {
        let Some(root) = json.as_object() else {
            panic!("unexpected session verify object. {:#?}", json);
        };

        let error = json::get_string("error", &root);

        if error != "VerifySession" {
            panic!("unexpected session verify error type. {:#?}", error);
        }

        let data = json::get_object("data", &root);
        let method = json::get_string("method", &data);

        if method != "Totp" {
            panic!("unexpected session verify method. {:#?}", method);
        }

        let Some(totp_data) = self.user.totp.as_ref() else {
            panic!("no totp data provided. cannot verify session");
        };

        let Some(now) = unix_epoch_sec() else {
            panic!("failed to get current unix epoch");
        };

        let body = VerifySession::Totp {
            value: rust_otp::totp(
                totp_data.algo.as_ref().unwrap_or(&rust_otp::Algo::SHA1),
                &totp_data.secret,
                totp_data.digits.unwrap_or(6),
                totp_data.step.unwrap_or(30),
                now
            )
        };

        let res = result::expect_with_err(
            self.post("/auth/session/verify")
                .json(&body)
                .send(),
            "failed to send session verify to server"
        );

        if res.status() != StatusCode::OK {
            let json: Value = res.json()
                .expect("verify session failed. unknown response body");

            panic!("verify session failed.\n{:#?}", json);
        }

        self.active_session = true;
    }

    pub fn get_session(&mut self) {
        if self.active_session {
            return;
        }

        let body = PasswordLogin {
            username: self.user.username.clone(),
            password: self.user.password.clone(),
        };

        let res = result::expect_with_err(
            self.post("/auth/session")
                .json(&body)
                .send(),
            "failed to send password login to server"
        );

        if res.status() == StatusCode::OK {
            self.active_session = true;
        } else if res.status() == StatusCode::UNAUTHORIZED {
            let json: Value = res.json()
                .expect("password login failed. unknown response body");

            self.handle_verify(json);
        } else {
            let json: Value = res.json()
                .expect("password login failed. unknown response body");

            panic!("password login failed.\n{:#?}", json);
        }
    }

    pub fn has_active_session(&self) -> bool {
        self.active_session
    }

    pub fn get_url<S>(&self, url: S) -> Url
    where
        S: AsRef<str>
    {
        result::expect_with_err(
            self.base_url.join(url.as_ref()),
            "invalid url provided"
        )
    }

    pub fn get<S>(&self, url: S) -> RequestBuilder
    where
        S: AsRef<str>
    {
        self.client.get(self.get_url(url))
    }

    pub fn post<S>(&self, url: S) -> RequestBuilder
    where
        S: AsRef<str>
    {
        self.client.post(self.get_url(url))
    }

    pub fn put<S>(&self, url: S) -> RequestBuilder
    where
        S: AsRef<str>
    {
        self.client.put(self.get_url(url))
    }

    pub fn patch<S>(&self, url: S) -> RequestBuilder
    where
        S: AsRef<str>
    {
        self.client.patch(self.get_url(url))
    }

    pub fn delete<S>(&self, url: S) -> RequestBuilder
    where
        S: AsRef<str>
    {
        self.client.delete(self.get_url(url))
    }

    pub fn head<S>(&self, url: S) -> RequestBuilder
    where
        S: AsRef<str>
    {
        self.client.head(self.get_url(url))
    }
}




