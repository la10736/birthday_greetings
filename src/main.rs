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

trait Repository {
    fn entries<'iter, 's:'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter>;
}

struct CsvRepository {}

impl Repository for CsvRepository {
    fn entries<'iter, 's:'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter> {
        unimplemented!()
    }
}

impl CsvRepository {
    pub fn by_path(path: impl AsRef<str>) -> Result<Self, String> {
        Ok(Self {})
    }
}


trait SendService {
    fn send(&self, employ: &Employ);
}
struct SmtpService {}

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

struct BirthdayGreetingService<R: Repository, S: SendService> {
    repository: R,
    send_service: S
}

impl<R: Repository, S: SendService> BirthdayGreetingService<R, S> {
    pub fn new(repository: R, send_service: S) -> Self {
        Self { repository, send_service }
    }

    pub fn send_greetings(&self, date: NaiveDate) {
        self.repository.entries().for_each(|e| self.send_service.send(e))
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
    use core::borrow::Borrow;

    impl Repository for Vec<Employ> {
        fn entries<'iter, 's:'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter> {
            Box::new(self.iter())
        }
    }

    impl Repository for Rc<Vec<Employ>> {
        fn entries<'iter, 's:'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter> {
            let vref: &Vec<Employ> = self.borrow();
            vref.entries()
        }
    }
    #[derive(Default)]
    struct NotCallService;
    impl SendService for NotCallService {
        fn send(&self, employ: &Employ) {
            panic!(format!("Should never call {:?}", employ))
        }
    }
    #[derive(Default)]
    struct CountCallsService { calls: RefCell<HashMap<Employ, usize>> }
    impl SendService for CountCallsService {
        fn send(&self, employ: &Employ) {
            self.calls.borrow_mut()
                .entry(employ.clone())
                .and_modify(|v| { *v = *v + 1 })
                .or_insert(1);
        }
    }
    impl CountCallsService {
        fn count(&self, employ: &Employ) -> Option<usize> {
            self.calls.borrow().get(employ).map(|c| *c)
        }
    }

    #[test]
    fn should_not_send_any_mail_if_no_employs() {
        let employees: Vec<Employ> = vec![];

        let birthday_service = BirthdayGreetingService::new(employees,
                                                            NotCallService);

        birthday_service.send_greetings(NaiveDate::from_ymd(2018,12,3));
    }

    impl <SS: SendService>  SendService for Rc<SS> {
        fn send(&self, entry: &Employ) {
            let ss: &SS = self.borrow();
            ss.send(entry)
        }
    }

    #[test]
    fn should_send_email() {
        let employees: Rc<Vec<Employ>> = Rc::new(vec![Employ::new("John", "Doe", NaiveDate::from_ymd(1998,12,3))]);
        let service = Rc::new(CountCallsService::default());

        let birthday_service = BirthdayGreetingService::new(employees.clone(),
                                                            service.clone());
        birthday_service.send_greetings(NaiveDate::from_ymd(2018,12,3));

        assert_eq!(service.count(&employees[0]).unwrap(), 1)
    }
}
