pub fn now_rfc3339() -> String {
    chrono::Local::now().to_rfc3339()
}

pub fn now() -> chrono::DateTime<chrono::Local> {
    chrono::Local::now()
}