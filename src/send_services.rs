use crate::Employ;
use lettre_email::Email;
use lettre::{SmtpClient, ClientSecurity, Transport};
use std::error::Error;

pub trait SendService {
    fn send(&self, employ: &Employ) -> Result<(), String>;
}

pub struct SmtpService {
    address: String
}

impl SendService for SmtpService {
    fn send(&self, employ: &Employ) -> Result<(), String>{
        let email = Email::builder()
            // Addresses can be specified by the tuple (email, alias)
            .to((employ.email.clone(), format!("{} {}", employ.name, employ.surname)))
            .from("user@example.com")
            .subject("Hi, Hello world")
            .text(format!("Hi {}!\n\
            Happy birthday!
            ", employ.name))
            .build()
            .unwrap();

        // Open a local connection on port 25
        let mut mailer = SmtpClient::new(&self.address, ClientSecurity::None).unwrap().transport();
        // Send the email
        mailer.send(email.into())
            .map(|_| ())
            .map_err(|e| e.description().to_owned())
    }
}

impl SmtpService {
    pub fn new(address: impl AsRef<str>) -> Self {
        Self { address: address.as_ref().to_owned() }
    }
}
