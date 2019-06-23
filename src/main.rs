// TODO: Remove me
#![allow(unused_variables)]

use chrono::{NaiveDate, Local, Datelike};

mod employ;
/// Rexport
pub use employ::Employ;

mod repositories;
use repositories::*;

mod send_services;
use send_services::*;

struct BirthdayGreetingService<R: Repository, S: SendService> {
    repository: R,
    send_service: S
}

impl<R: Repository, S: SendService> BirthdayGreetingService<R, S> {
    pub fn new(repository: R, send_service: S) -> Self {
        Self { repository, send_service }
    }

    pub fn send_greetings(&self, date: NaiveDate) -> Result<(), String >{
        let is_birthday = |e: &&Employ| e.birthday.day() == date.day() && e.birthday.month() == date.month();

        self.repository.entries()
            .filter(is_birthday)
            .map(|e| self.send_service.send(e) )
            .collect()
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
    use rstest::*;

    use std::collections::HashSet;
    use std::cell::RefCell;
    use std::rc::Rc;
    use core::borrow::Borrow;

    type Employees = Vec<Employ>;
    type RcEmployees = Rc<Employees>;

    impl Repository for Employees {
        fn entries<'iter, 's:'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter> {
            Box::new(self.iter())
        }
    }

    impl Repository for RcEmployees {
        fn entries<'iter, 's:'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter> {
            let vref: &Employees = self.borrow();
            vref.entries()
        }
    }
    trait AsRc: Default {
        fn rc() -> Rc<Self> {
            Self::default().into()
        }
    }

    impl<D: Default> AsRc for D {}

    trait AsSet<T: Clone> {
        fn as_set(&self) -> HashSet<T>;
    }
    impl AsSet<Employ> for RcEmployees {
        fn as_set(&self) -> HashSet<Employ> {
            self.iter().cloned().collect()
        }
    }


    #[derive(Default)]
    struct NotCallService;
    impl SendService for NotCallService {
        fn send(&self, employ: &Employ) -> Result<(), String>{
            panic!(format!("Should never call {:?}", employ))
        }
    }
    #[derive(Default)]
    struct NoMoreThanOneCallService { calls: RefCell<HashSet<Employ>> }
    impl SendService for NoMoreThanOneCallService {
        fn send(&self, employ: &Employ) -> Result<(), String>{
            if !self.calls.borrow_mut().insert(employ.clone()) {
                panic!("Already sent to {:?}", employ)
            }
            Ok(())
        }
    }
    impl NoMoreThanOneCallService {
        fn notified(&self) -> HashSet<Employ> {
            self.calls.borrow().clone()
        }
    }

    impl <SS: SendService>  SendService for Rc<SS> {
        fn send(&self, entry: &Employ) -> Result<(), String>{
            let ss: &SS = self.borrow();
            ss.send(entry)
        }
    }

    pub fn date<S: AsRef<str>>(data: S) -> NaiveDate {
        NaiveDate::parse_from_str(data.as_ref(),"%Y/%m/%d")
            .expect("Cannot parse date")
    }

    fn employees(data: &[&str]) -> RcEmployees {
        data.iter()
            .map(Employ::from)
            .collect::<Vec<_>>()
            .into()
    }

    fn no_employees() -> RcEmployees {
        employees(&[])
    }

    #[fixture]
    fn just_one_call() -> NoMoreThanOneCallService {
        NoMoreThanOneCallService::default()
    }

    #[rstest_parametrize(employees, date,
        case::no_employees(no_employees(), date("2018/12/3")),
        case::no_birthday_miss_month(employees(&["Bernard,Trump,1992/11/1", "Ronald,Dump,1995/5/1"]), date("2018/1/1")),
        case::no_birthday_miss_day(employees(&["Bernard,Trump,1992/11/1", "Ronald,Dump,1995/5/1"]), date("2018/5/4")),
    )]
    fn should_not_send_any_mail(employees: RcEmployees, date: NaiveDate) {
        let birthday_service = BirthdayGreetingService::new(employees,
                                                            NotCallService);

        birthday_service.send_greetings(date);
    }

    #[rstest_parametrize(repo, date, expected,
        case(employees(&["John,Doe,1998/12/3"]), date("2018/12/3"),employees(&["John,Doe,1998/12/3"])),
        case(employees(&["Bernard,Trump,1992/11/1", "Ronald,Dump,1995/11/1"]), date("2018/11/1"),
                employees(&["Bernard,Trump,1992/11/1", "Ronald,Dump,1995/11/1"])),
        case::just_one(employees(&["Bernard,Trump,1992/11/1", "Ronald,Dump,1995/5/1"]), date("2018/11/1"),
            employees(&["Bernard,Trump,1992/11/1"]))
    )]
    fn should_send_email(just_one_call: NoMoreThanOneCallService, repo: RcEmployees, date: NaiveDate, expected: RcEmployees) {
        let service = Rc::new(just_one_call);

        let birthday_service = BirthdayGreetingService::new(repo.clone(),
                                                            service.clone());
        birthday_service.send_greetings(date);

        assert_eq!(service.notified(), expected.as_set());
    }
}
