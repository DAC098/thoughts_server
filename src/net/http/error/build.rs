//! common errors that the server can respond with
//! 
//! over time this may get reduced in favor of a better aproach if one is
//! though of. more of a drop in replacement for how errors where created
//! previously.

use actix_web::http::StatusCode;

use super::Error;

#[inline]
pub fn permission_denied<M>(message: M) -> Error
where
    M: Into<String>
{
    Error::new()
        .set_status(StatusCode::UNAUTHORIZED)
        .set_name("PermissionDenied")
        .set_message(message)
}

#[inline]
pub fn invalid_password() -> Error
{
    Error::new()
        .set_status(StatusCode::UNAUTHORIZED)
        .set_name("InvalidPassword")
        .set_message("invalid password provided")
}

#[inline]
pub fn user_id_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("UserIDNotFound")
        .set_message(format!("failed to find the requested user id: {}", id))
}

#[inline]
pub fn entry_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("EntryNotFound")
        .set_message(format!("failed to find the requested entry id: {}", id))
}

#[inline]
pub fn text_entry_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("TextEntryNotFound")
        .set_message(format!("failed to find the requested text entry id: {}", id))
}

#[inline]
pub fn global_custom_field_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("GlobalCustomFieldNotFound")
        .set_message(format!("failed to find the requested global custom field id: {}", id))
}

#[inline]
pub fn custom_field_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("CustomFieldNotFound")
        .set_message(format!("failed to find the requested custom field id: {}", id))
}

#[inline]
pub fn tag_not_found(id: &i32) -> Error 
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("TagNotFound")
        .set_message(format!("failed to find the requested tag id: {}", id))
}

#[inline]
pub fn entry_marker_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("EntryMarkerNotFound")
        .set_message(format!("failed to find the requested entry marker id: {}", id))
}

#[inline]
pub fn entry_comment_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("EntryCommentNotFound")
        .set_message(format!("failed to find the requested entry commit id: {}", id))
}

#[inline]
pub fn audio_entry_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("AudioEntryNotFound")
        .set_message(format!("failed to find the requested audio entry id: {}", id))
}

#[inline]
pub fn group_not_found(id: &i32) -> Error
{
    Error::new()
        .set_status(StatusCode::NOT_FOUND)
        .set_name("GroupNotFound")
        .set_message(format!("failed to find the requested group id: {}", id))
}

#[inline]
pub fn username_exists<U>(_username: U) -> Error
where
    U: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("UsernameExists")
        .set_message("given username already exists")
}

#[inline]
pub fn email_exists<E>(_email: E) -> Error
where
    E: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("EmailExists")
        .set_message("given email already exists")
}

#[inline]
pub fn entry_exists<C>(created: C) -> Error
where
    C: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("EntryExists")
        .set_message(format!("given entry date already exists. date: {}", created.into()))
}

#[inline]
pub fn custom_field_exists<N>(name: N) -> Error
where
    N: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("CustomFieldExists")
        .set_message(format!("given custom field already exists. name: {}", name.into()))
}

#[inline]
pub fn global_custom_field_exists<N>(name: N) -> Error
where
    N: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("GlobalCustomFieldExists")
        .set_message(format!("given global custom field already exists. name: {}", name.into()))
}

#[inline]
pub fn group_already_exists<N>(name: N) -> Error
where
    N: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("GroupAlreadyExists")
        .set_message(format!("given group name already exists. name: {}", name.into()))
}

#[inline]
pub fn validation<M>(message: M) -> Error
where
    M: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("Validation")
        .set_message(message)
}

#[inline]
pub fn bad_request<M>(message: M) -> Error
where 
    M: Into<String>
{
    Error::new()
        .set_status(StatusCode::BAD_REQUEST)
        .set_name("BadRequest")
        .set_message(message)
}