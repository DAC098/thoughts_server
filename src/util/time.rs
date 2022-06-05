#[inline]
pub fn now() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}

#[inline]
pub fn now_utc() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}