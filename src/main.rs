// TODO: Remove me
#![allow(unused_variables)]

use chrono::{NaiveDate, Local};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Employ;

impl Employ {
    pub fn new(name: &str, lastname: &str, birthday: NaiveDate) -> Self {
        Employ
    }
}

trait Repository {}

struct CsvRepository {}

impl Repository for CsvRepository {}

impl CsvRepository {
    pub fn by_path(path: impl AsRef<str>) -> Result<Self, String> {
        Ok(Self {})
    }
}


trait SendService {}
struct SmtpService {}

impl SendService for SmtpService {}

impl SmtpService {
    pub fn new(address: impl AsRef<str>) -> Self {
        Self {}
    }
}

struct BirthdayGreetingService {}

impl BirthdayGreetingService {
    pub fn new(repository: impl Repository, send_service: impl SendService) -> Self {
        Self {}
    }

    pub fn send_greetings(&self, date: NaiveDate) {
    }
}


fn main() {
    let repository = CsvRepository::by_path(
        std::env::args().nth(1).unwrap()
    ).unwrap();
    let email_service = SmtpService::new(std::env::var("SMTP_SERVER").unwrap());

    let birthday_service = BirthdayGreetingService::new(repository, email_service);

    birthday_service.send_greetings(Local::today().naive_local());
}


#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::cell::RefCell;
    use std::rc::Rc;

    impl Repository for Vec<Employ> {}
    impl<R: Repository> Repository for Rc<R> {}
    #[derive(Default)]
    struct NotCallService;
    impl SendService for NotCallService {

    }
    #[derive(Default)]
    struct CountCallsService<'a> { calls: RefCell<HashMap<&'a Employ, usize>> }
    impl<'a> SendService for CountCallsService<'a> {

    }
    impl<'a> CountCallsService<'a> {
        fn count(&self, employ: &Employ) -> Option<usize> {
            self.calls.borrow().get(employ).map(|c| *c)
        }
    }

    #[test]
    fn should_not_send_any_mail() {
        let employees: Vec<Employ> = vec![];

        let birthday_service = BirthdayGreetingService::new(employees,
                                                            NotCallService);

        birthday_service.send_greetings(NaiveDate::from_ymd(2018,12,3));
    }

    impl <SS: SendService>  SendService for Rc<SS> {}

    #[test]
    fn should_send_email() {
        let employees: Rc<Vec<Employ>> = Rc::new(vec![Employ::new("John", "Doe", NaiveDate::from_ymd(1998,12,3))]);
        let service = Rc::new(CountCallsService::default());

        let birthday_service = BirthdayGreetingService::new(employees.clone(),
                                                            service.clone());

        assert_eq!(service.count(&employees[0]).unwrap(), 1)
    }
}
