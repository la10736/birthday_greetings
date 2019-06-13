use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use temp_testdir::TempDir;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{Datelike, Duration};
use std::ops::Add;
use std::error::Error;
use json::{self, JsonValue};
use std::marker::PhantomData;

static APP_PATH: &'static str = "target/debug/birthday_greetings";
static SMTP_DOCKER_IMAGE: &'static str = "fake_smtp";
static SMTP_DOCKER_FILE_DIR: &'static str = "fake_smtp_docker";

struct DockerCommand;

trait ExecuteDockerCommand {
    fn execute(&mut self, message: &str) -> Result<String, String>;

    fn map(&self, output: Output, info: &str) -> Result<String, String> {
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            eprintln!(r#"{}:
        out -> {}
        err -> {}\n"#, info,
                      String::from_utf8_lossy(&output.stdout),
                      String::from_utf8_lossy(&output.stderr)
            );
            Err(info.to_owned())
        }
    }
}

impl<'a> ExecuteDockerCommand for &'a mut Command {
    fn execute(&mut self, message: &str) -> Result<String, String> {
        self
            .output()
            .map_err(|e| e.description().to_owned())
            .and_then(|o| self.map(o, message))
    }
}

impl DockerCommand {
    pub fn start(&self, image: &str) -> Result<String, String> {

        Command::new("docker")
            .arg("run")
            .arg("-d")
            .arg(image)
            .execute(&format!("On start docker imege {}", image))
    }

    pub fn stop(&self, id: &str) -> Result<(), String> {
        Command::new("docker")
            .arg("stop")
            .arg(id)
            .execute(&format!("On stop docker container {}", id))
            .map(|_| ())
    }

    pub fn inspect(&self, id: &str) -> Result<JsonValue, String> {
        Command::new("docker")
            .arg("inspect")
            .arg(id)
            .execute(&format!("On inspect docker container {}", id))
            .and_then(|s|
                json::parse(&s)
                    .map_err(|e| e.description().to_owned())
            )
    }

    pub fn logs(&self, id: &str) -> Result<String, String> {
        Command::new("docker")
            .arg("logs")
            .arg(id)
            .execute(&format!("On get logs docker container {}", id))
    }

    pub fn image_exists(&self, image: &str) -> Result<bool, String> {
        Command::new("docker")
            .arg("image")
            .arg("inspect")
            .arg(image)
            .execute(&format!("On looking for image {}", image))
            .map(|_| true)
    }

    fn build<P: AsRef<Path>>(&self, name: &str, path: P) -> Result<(), String> {
        Command::new("docker")
            .arg("build")
            .arg("-t").arg(name)
            .arg(path.as_ref())
            .execute(&format!("On build for image {} at {}",
                                               name, path.as_ref().to_string_lossy()))
            .map(|_| ())
    }
}

trait SmtpServerState {}

struct Created;

struct Initialized;

impl SmtpServerState for Created {}

impl SmtpServerState for Initialized {}


struct SmtpServer<S: SmtpServerState> {
    img_name: String,
    docker_id: Option<String>,
    phantom: PhantomData<S>,
}

impl<S: SmtpServerState> Drop for SmtpServer<S> {
    fn drop(&mut self) {
        match &self.docker_id {
            Some(id) => {
                let _ = DockerCommand.stop(id);
            }
            None => {}
        }
    }
}

impl SmtpServer<Created> {
    pub fn new(img_name: &str) -> SmtpServer<Created> {
        Self { img_name: img_name.to_owned(), docker_id: None, phantom: PhantomData }
    }

    pub fn start(self) -> Result<SmtpServer<Initialized>, String> {
        let id = DockerCommand.start(&self.img_name)?;
        Ok(SmtpServer { img_name: self.img_name.clone(), docker_id: Some(id.to_string()), phantom: PhantomData })
    }
}

impl SmtpServer<Initialized> {
    pub fn address(&self) -> Result<String, String> {
        let j = DockerCommand.inspect(self.id())?;
        Ok(format!("{}:2525", &j[0]["NetworkSettings"]["IPAddress"]))
    }

    pub fn logs(&self) -> Result<String, String> {
        DockerCommand.logs(self.id())
    }

    fn id(&self) -> &str {
        self.docker_id.as_ref().unwrap()
    }
}

fn build_smtp_container<P: AsRef<Path>>(name: &str, path: P) -> Result<(), String> {
    if DockerCommand.image_exists(name).unwrap_or(false) {
        Ok(())
    } else {
        DockerCommand.build(name, path)
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
               .format("%Y/%m/%d").to_string())
        .expect("Cannot write entry");
    write!(f, "Paperone, De Paperoni, {}, paperon.depaperoni@dmail.com",
           chrono::Local::today()
               .add(Duration::days(1))
               .with_year(1867).unwrap()
               .format("%Y/%m/%d").to_string())
        .expect("Cannot write entry");


    let smtp_server = smtp_server();

    let out = Command::new(&Path::new(APP_PATH))
        .arg(employees)
        .arg(dbg!(&smtp_server.address().unwrap()))
        .output()
        .expect(&format!("Cannot start App '{}'", APP_PATH));

    let logs = smtp_server.logs().expect("Cannot read smtp server logs");

    unimplemented!("Should be completed")
}
