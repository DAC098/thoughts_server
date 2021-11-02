pub fn trimmed_optional_string(given: Option<String>) -> Option<String> {
    if let Some(value) = given {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    } else {
        None
    }
}

#[inline]
pub fn trimmed_string(given: String) -> String {
    given.trim().to_owned()
}