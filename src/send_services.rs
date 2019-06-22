use crate::Employ;

pub trait SendService {
    fn send(&self, employ: &Employ);
}

pub struct SmtpService {}

impl SendService for SmtpService {
    fn send(&self, employ: &Employ) {
        unimplemented!()
    }
}

impl SmtpService {
    pub fn new(address: impl AsRef<str>) -> Self {
        Self {}
    }
}
