use lettre::{
    Message, 
    SmtpTransport, 
    Transport,
    transport::smtp::{
        client::{
            TlsParameters,
            Tls
        },
        authentication::{
            Mechanism,
            Credentials
        }
    },
    message::{
        Mailbox
    },
    address::{
        Address
    }
};

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "full");
    env_logger::init();

    let from_address = Address::new("", "gmail.com").unwrap();
    let from_mailbox = Mailbox::new(None, from_address);
    let to_address = Address::new("", "gmail.com").unwrap();
    let to_mailbox = Mailbox::new(None, to_address);
    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject("test email")
        .body("test email being sent".to_owned())
        .unwrap();

    let credentials = Credentials::new(
        "".to_owned(), 
        "".to_owned()
    );

    let tls = TlsParameters::builder("smpt.gmail.com".to_owned())
        .build()
        .unwrap();
    
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .tls(Tls::Required(tls))
        .credentials(credentials)
        .authentication(vec!(Mechanism::Login))
        .port(465)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("email sending successful"),
        Err(e) => panic!("email sending failed\n{:?}", e)
    }
}