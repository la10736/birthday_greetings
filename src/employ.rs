use chrono::NaiveDate;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Employ{
    pub name: String,
    pub surname: String,
    pub birthday: NaiveDate,
    pub email: String,
}

impl Employ {
    pub fn new(name: &str, surname: &str, birthday: NaiveDate, email: &str) -> Self {
        Employ{ name: name.to_owned(), surname: surname.to_owned(), birthday, email: email.to_owned() }
    }
}

#[cfg(test)]
impl<S: AsRef<str>> From<S> for Employ {
    fn from(s: S) -> Self {
        let mut data = s.as_ref().splitn(4, ',');
        let name = data.next().expect("Cannot find name").trim();
        let surname = data.next().expect("Cannot find surname").trim();
        let birth = crate::test::date(data.next().expect("Cannot find birth date"));
        let email = data.next().unwrap_or_default();

        return Employ::new(name, surname, birth, email)
    }
}
