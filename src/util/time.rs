#[inline]
pub fn now_rfc3339() -> String {
    chrono::Local::now().to_rfc3339()
}

#[inline]
pub fn now() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

#[inline]
pub fn now_utc() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}