use std::path::{Path, PathBuf};
use std::process::Command;
use temp_testdir::TempDir;
use std::env::temp_dir;
use std::fs::{OpenOptions};
use std::io::Write;
use chrono::{Datelike, Duration};
use std::ops::Add;

static APP_PATH: &'static str = "target/debug/birthday_greetings";

#[test]
fn should_fail() {
    // TODO:
    //  - [ ] instantiate docker server
    //  - [ ] Add two employees to a csv file (one with birthday today and the other tomorrow)
    //  - [ ] run app
    //  - [ ] check docker server output to find receipts for the one that today is hos birthday
    //  - [ ] close docker server

    // Run App
    let employees = "tests/resources/employees.csv";
    let temp = TempDir::default();
    let mut file_path = PathBuf::from(temp.as_ref());
    file_path.push("employees.csv");
    std::fs::copy(employees, &file_path)
        .expect(&format!("Cannot copy files {} -> {}", employees, file_path.to_string_lossy()));

    let mut f = OpenOptions::new()
        .write(true)
        .open(file_path.clone())
        .expect(&format!("Cannot open {}", file_path.to_string_lossy()));

    write!(f, "Paolino, Paperino, {}, paolino.paperino@dmail.com",
           chrono::Local::today()
               .with_year(1920).unwrap()
               .format("%Y/%m/%d").to_string());
    write!(f, "Paperone, De Paperoni, {}, paperon.depaperoni@dmail.com",
           chrono::Local::today()
               .add(Duration::days(1))
               .with_year(1867).unwrap()
               .format("%Y/%m/%d").to_string());


    let smtp_server = "smtp.host:1234";

    let out = Command::new(&Path::new(APP_PATH))
        .arg(employees)
        .arg(&format!("{}", smtp_server))
        .output()
        .expect(&format!("Cannot start App '{}'", APP_PATH));

    unimplemented!("Should be completed")
}
