use lettre::message::{Mailbox};
use lettre::address::{Address, AddressError};
use lettre::transport::smtp::{Error as SMTPError, SmtpTransport};
use lettre::transport::smtp::authentication::{Credentials};

pub mod message_body;

pub fn get_mailbox(email: String, name: Option<String>) -> Result<Mailbox, AddressError> {
    Ok(Mailbox::new(
        name,
        email.parse::<Address>()?
    ))
}

pub fn get_credentials(username: String, password: String) -> Credentials {
    Credentials::new(username, password)
}

pub fn get_smtp_transport(relay: &String, credentials: Credentials) -> Result<SmtpTransport, SMTPError> {
    Ok(SmtpTransport::relay(relay.as_str())?
        .credentials(credentials)
        .build())
}