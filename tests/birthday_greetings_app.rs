use std::path::{Path, PathBuf};
use std::process::Command;
use temp_testdir::TempDir;
use std::env::temp_dir;
use std::fs::{OpenOptions};
use std::io::Write;
use chrono::{Datelike, Duration};
use std::ops::Add;
use std::error::Error;
use json;

static APP_PATH: &'static str = "target/debug/birthday_greetings";
static SMTP_DOCKER_IMAGE: &'static str = "fake_smtp";
static SMTP_DOCKER_FILE_DIR: &'static str = "fake_smtp_docker";

trait SmtpServerState {}
struct Created;
struct Initialized;
impl SmtpServerState for Created {}
impl SmtpServerState for Initialized {}
use std::marker::PhantomData;
use json::JsonValue;

struct SmtpServer<S: SmtpServerState> {
    img_name: String,
    docker_id: Option<String>,
    phantom: PhantomData<S>
}

impl <S: SmtpServerState> Drop for SmtpServer<S> {
    fn drop(&mut self) {
        match &self.docker_id {
            Some(id) => {
                Command::new("docker")
                    .arg("stop")
                    .arg(id)
                    .output()
                    .map_err(|e| e.description().to_owned())
                    .and_then(|o|
                        if o.status.success() {
                            Ok(())
                        } else {
                            eprintln!("On stopping docker imege {} [{}]: \
                out -> {}\
                err -> {}", self.img_name, id,
                                      String::from_utf8_lossy(&o.stdout),
                                      String::from_utf8_lossy(&o.stderr)
                            );
                            Err("Cannot stop docker container".to_owned())
                        }
                    );

            },
            None => {}
        }
    }
}

impl SmtpServer<Created> {
    pub fn new(img_name: &str) -> Self {
        Self { img_name: img_name.to_owned(), docker_id: None, phantom: PhantomData }
    }

    pub fn start(self) -> Result<SmtpServer<Initialized>, String> {
        let id = Command::new("docker")
            .arg("run")
            .arg("-d") // Detached
            .arg(&self.img_name)
            .output()
            .map_err(|e| e.description().to_owned())
            .and_then(|o|
                if o.status.success() {
                    Ok(String::from_utf8_lossy(&o.stdout).trim().to_owned())
                } else {
                    eprintln!("On starting docker imege {}: \
                out -> {}\
                err -> {}", self.img_name,
                              String::from_utf8_lossy(&o.stdout),
                              String::from_utf8_lossy(&o.stderr)
                    );
                    Err("Cannot start docker image".to_owned())
                }
            )?;
        Ok(SmtpServer { img_name: self.img_name.clone(), docker_id: Some(id.to_string()), phantom: PhantomData })
    }
}

impl SmtpServer<Initialized> {
    pub fn address(&self) -> Result<String, String> {
        let j = self.inspect()?;
        Ok(format!("{}:2525", &j[0]["NetworkSettings"]["IPAddress"]))
    }

    pub fn logs(&self) -> Result<String, String> {
        Command::new("docker")
            .arg("logs")
            .arg(self.id())
            .output()
            .map_err(|e| e.description().to_owned())
            .and_then(|o|
                if o.status.success() {
                    Ok(String::from_utf8_lossy(&o.stdout).trim().to_owned())
                } else {
                    eprintln!("On inspecting docker logs {}: \
                out -> {}\
                err -> {}", self.id(),
                              String::from_utf8_lossy(&o.stdout),
                              String::from_utf8_lossy(&o.stderr)
                    );
                    Err("Cannot inspect docker logs".to_owned())
                }
            )
    }

    fn id(&self) -> &str {
        self.docker_id.as_ref().unwrap()
    }

    fn inspect(&self) -> Result<JsonValue, String> {
        Command::new("docker")
            .arg("inspect")
            .arg(self.id())
            .output()
            .map_err(|e| e.description().to_owned())
            .and_then(|o|
                if o.status.success() {
                    json::parse(&String::from_utf8_lossy(&o.stdout))
                        .map_err(|e| e.description().to_owned())
                } else {
                    eprintln!("On inspecting docker imege {}: \
                out -> {}\
                err -> {}", self.id(),
                              String::from_utf8_lossy(&o.stdout),
                              String::from_utf8_lossy(&o.stderr)
                    );
                    Err("Cannot inspect docker image".to_owned())
                }
            )
    }


}

fn docker_image_exists(name: &str) -> bool {
    Command::new("docker")
        .arg("image")
        .arg("inspect")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn docker_image_build<P: AsRef<Path>>(name: &str, path: P) -> Result<(), String> {
    Command::new("docker")
        .arg("build")
        .arg("-t").arg(name)
        .arg(path.as_ref())
        .output()
        .map_err(|e| e.description().to_owned())
        .and_then(|o|
            if o.status.success() {
                Ok(())
            } else {
                eprintln!("On building docker imege {}: \
                out -> {}\
                err -> {}", path.as_ref().to_string_lossy(),
                          String::from_utf8_lossy(&o.stdout),
                          String::from_utf8_lossy(&o.stderr)
                );
                Err("Cannot build docker image".to_owned())
            }
        )
}

fn build_smtp_container<P: AsRef<Path>>(name: &str, path: P) -> Result<(),String> {
    if docker_image_exists(name) {
        Ok(())
    } else {
        docker_image_build(name, path)
    }
}

fn smtp_server() -> SmtpServer<Initialized> {
    build_smtp_container(SMTP_DOCKER_IMAGE, SMTP_DOCKER_FILE_DIR)
        .expect("Cannot build smtp docker image");
    SmtpServer::new(SMTP_DOCKER_IMAGE)
        .start()
        .expect("Cannot start smtp server")
}

#[test]
fn should_fail() {
    // TODO:
    //  - [x] instantiate docker server
    //  - [x] get address
    //  - [x] Add two employees to a csv file (one with birthday today and the other tomorrow)
    //  - [x] run app
    //  - [ ] check docker server output to find receipts for the one that today is hos birthday
    //  - [x] close docker server
    //  - [ ] Refactor docker calls

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


    let smtp_server = smtp_server();

    let out = Command::new(&Path::new(APP_PATH))
        .arg(employees)
        .arg(dbg!(&smtp_server.address().unwrap()))
        .output()
        .expect(&format!("Cannot start App '{}'", APP_PATH));

    let logs = smtp_server.logs().expect("Cannot read smtp server logs");

    unimplemented!("Should be completed")
}
