mod initiator;
pub use initiator::*;

pub fn is_json_mime(mime: mime::Mime) -> bool {
    mime == mime::APPLICATION_JAVASCRIPT
}