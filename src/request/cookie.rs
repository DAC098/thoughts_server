use std::{
    collections::HashMap, 
    convert::{TryFrom, TryInto}, 
    hash::Hash, 
    borrow::Borrow, 
    pin::Pin,
    future::{Future, ready}
};

use actix_web::{
    http::header::{HeaderMap, SET_COOKIE, HeaderName, HeaderValue, InvalidHeaderValue, TryIntoHeaderValue, TryIntoHeaderPair}, 
    FromRequest, HttpRequest
};
use chrono::{
    DateTime, 
    Utc, 
    Duration
};

use crate::response::error;

pub struct CookieMap(HashMap<String, Vec<String>>);

impl CookieMap {

    // pub fn has_key<K>(&self, key: &K) -> bool
    // where
    //     K: ?Sized + Hash + Eq,
    //     String: Borrow<K>
    // {
    //     self.0.contains_key(key)
    // }

    pub fn get_value_ref<K>(&self, key: &K) -> Option<&String>
    where
        K: ?Sized + Hash + Eq,
        String: Borrow<K>
    {
        if let Some(list) = self.0.get(key) {
            list.first()
        } else {
            None
        }
    }

    // pub fn get_value<K>(&self, key: &K) -> Option<String>
    // where
    //     K: ?Sized + Hash + Eq,
    //     String: Borrow<K>
    // {
    //     if let Some(list) = self.0.get(key) {
    //         list.first()
    //             .map(|v| v.clone())
    //     } else {
    //         None
    //     }
    // }

}

impl From<&HeaderMap> for CookieMap {
    fn from(headers: &HeaderMap) -> Self {
        let mut rtn: HashMap<String, Vec<String>> = HashMap::new();

        for cookies in headers.get_all("cookie") {
            let to_str = cookies.to_str();

            if to_str.is_err() {
                continue;
            }

            for value in to_str.unwrap().split("; ") {
                if let Some((k, v)) = value.split_once('=') {
                    let k_string = k.to_owned();

                    if let Some(list) = rtn.get_mut(&k_string) {
                        list.push(v.to_owned());
                    } else {
                        let mut vec = Vec::with_capacity(1);
                        vec.push(v.to_owned());

                        rtn.insert(k_string, vec);
                    }
                }
            }
        }

        CookieMap(rtn)
    }
}

impl From<&HttpRequest> for CookieMap {
    fn from(req: &HttpRequest) -> Self {
        CookieMap::from(req.headers())
    }
}

impl FromRequest for CookieMap {
    type Error = error::ResponseError;
    type Future = Pin<Box<dyn Future<Output = std::result::Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        let map = CookieMap::from(req.headers());
        let fut = ready(Ok(map));

        Box::pin(fut)
    }
}

pub enum SameSite {
    Strict,
    Lax,
    None
}

impl SameSite {
    pub fn as_str(&self) -> &str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None"
        }
    }
}

pub struct SetCookie {
    pub key: String,
    pub value: String,

    pub expires: Option<DateTime<Utc>>,
    pub max_age: Option<Duration>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>
}

impl SetCookie {
    pub fn new<K,V>(key: K, value: V) -> SetCookie
    where
        K: Into<String>,
        V: Into<String>
    {
        SetCookie {
            key: key.into(), 
            value: value.into(),
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: false,
            http_only: false,
            same_site: None
        }
    }

    pub fn set_expires(&mut self, expires: DateTime<Utc>) -> () {
        self.expires = Some(expires);
    }

    pub fn set_max_age(&mut self, max_age: Duration) -> () {
        self.max_age = Some(max_age);
    }

    pub fn set_domain<D>(&mut self, domain: D) -> ()
    where
        D: Into<String>
    {
        self.domain = Some(domain.into())
    }

    pub fn set_path<P>(&mut self, path: P) -> ()
    where
        P: Into<String>
    {
        self.path = Some(path.into());
    }

    pub fn set_secure(&mut self, secure: bool) -> () {
        self.secure = secure;
    }

    pub fn set_http_only(&mut self, http_only: bool) -> () {
        self.http_only = http_only;
    }

    pub fn set_same_site(&mut self, same_site: SameSite) -> () {
        self.same_site = Some(same_site);
    }

    pub fn into_header_value(self) -> std::result::Result<HeaderValue, InvalidHeaderValue> {
        let mut rtn = format!("{}={}", self.key, self.value);

        if let Some(expire) = self.expires {
            let date = expire.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
            rtn.push_str("; Expires=");
            rtn.push_str(date.as_str());
        }

        if let Some(duration) = self.max_age {
            let seconds = duration.num_seconds().to_string();
            rtn.push_str("; Max-Age=");
            rtn.push_str(seconds.as_str());
        }

        if let Some(domain) = self.domain {
            rtn.push_str("; Domain=");
            rtn.push_str(domain.as_str());
        }

        if let Some(path) = self.path {
            rtn.push_str("; Path=");
            rtn.push_str(path.as_str());
        }

        if self.secure {
            rtn.push_str("; Secure");
        }

        if self.http_only {
            rtn.push_str("; HttpOnly");
        }

        if let Some(same_site) = self.same_site {
            rtn.push_str("; SameSite=");
            rtn.push_str(same_site.as_str());
        }

        HeaderValue::from_str(&rtn)
    }
}

impl TryFrom<SetCookie> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: SetCookie) -> Result<Self, Self::Error> {
        value.into_header_value()
    }
}

impl TryIntoHeaderValue for SetCookie {
    type Error = InvalidHeaderValue;

    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        HeaderValue::try_from(self)
    }
}

impl TryIntoHeaderPair for SetCookie {
    type Error = InvalidHeaderValue;

    fn try_into_pair(self) -> Result<(HeaderName, HeaderValue), Self::Error> {
        let value = self.try_into()?;

        Ok((SET_COOKIE, value))
    }
}