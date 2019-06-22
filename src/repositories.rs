use crate::Employ;
use std::error::Error;
use chrono::NaiveDate;
use std::io::Read;
use std::fs::File;
use std::path::Path;

pub trait Repository {
    fn entries<'iter, 's: 'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter>;
}

#[derive(Default, Debug)]
pub struct CsvRepository {
    employees: Vec<Employ>
}

impl Repository for CsvRepository {
    fn entries<'iter, 's: 'iter>(&'s self) -> Box<dyn Iterator<Item=&'s Employ> + 'iter> {
        Box::new(self.employees.iter())
    }
}

impl CsvRepository {
    pub fn by_path(path: impl AsRef<Path>) -> Result<Self, String> {
        File::open(path)
            .map_err(|e| e.description().to_owned())
            .and_then(Self::read)
    }

    fn read(data: impl Read) -> Result<Self, String> {
        let mut reader = csv::Reader::from_reader(data);
        let mut employees: Vec<Employ> = Vec::new();
        for record in reader.records() {
            let record = record.map_err(|e| e.description().to_owned())?;
            employees.push(Employ::new(
                &record[1].trim(), &record[0].trim(), Self::parse_date(&record[2])?, &record[3].trim()))
        }

        Ok(CsvRepository { employees })
    }

    fn parse_date(date: &str) -> Result<NaiveDate, String> {
        NaiveDate::parse_from_str(date.trim(), "%Y/%m/%d").map_err(|e| e.description().to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;
    use deindent::Deindent;

    #[test]
    fn read_employees() {
        let data = r#"
        last_name, first_name, date_of_birth, email
        Doe, John, 1982/10/08, john.doe@foobar.com
        Ann, Mary, 1975/09/11, mary.ann@foobar.com
        "#.deindent();

        let repo = CsvRepository::read(data.as_bytes()).unwrap();

        let mut employees = repo.entries();

        assert_eq!(employees.next().unwrap(),
                   &Employ::new("John", "Doe", NaiveDate::from_ymd(1982, 10, 8), "john.doe@foobar.com"));
        assert_eq!(employees.next().unwrap(),
                   &Employ::new("Mary", "Ann", NaiveDate::from_ymd(1975, 9, 11), "mary.ann@foobar.com"));
        assert_eq!(None, employees.next())
    }

    pub mod deindent {
        pub trait Deindent {
            fn deindent(self) -> String;
        }

        impl<S: AsRef<str>> Deindent for S {
            fn deindent(self) -> String {
                use std::fmt::Write;
                let message = self.as_ref();
                let skip = min_indent_size(message);
                let mut output = String::new();
                message.lines().for_each(|l|
                    writeln!(&mut output, "{}", &l[skip.min(l.len())..])
                        .unwrap());
                output
            }
        }

        fn min_indent_size(message: &str) -> usize {
            let skip = message.lines()
                .filter(|l| l.len() > 0)
                .map(|l| count_start(l, ' '))
                .min().unwrap_or(0);
            skip
        }

        fn count_start(l: &str, ch: char) -> usize {
            l.chars().take_while(|&c| c == ch).count()
        }
    }
}
