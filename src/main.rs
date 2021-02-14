#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use fs::File;
use nanoid;
use rocket::http::RawStr;
use rocket::response::NamedFile;
use rocket::Data;
use rocket_contrib::templates::Template;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::{borrow::Cow, collections::HashMap};

pub struct PasteId<'a>(Cow<'a, str>);

impl<'a> PasteId<'a> {
    pub fn new(size: usize) -> PasteId<'a> {
        let id = nanoid::nanoid!(size);
        PasteId(Cow::Owned(id))
    }
}

impl<'a> fmt::Display for PasteId<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// favicon
#[get("/favicon.ico")]
fn favicon() -> std::io::Result<NamedFile> {
    let resource_path = PathBuf::from("views/assets/img/favicon.ico");
    NamedFile::open(resource_path)
}

#[get("/")]
fn index() -> Template {
    let hm: HashMap<String, String> = HashMap::new();
    Template::render("index", hm)
}

#[get("/<id>", format = "text/html")]
fn retrieveindex(id: &RawStr) -> Template {
    print!("{}", id);
    // let filename = format!("pastes/{id}", id = id);
    // File::open(&filename).ok()
    let mut lang: HashMap<String, String> = HashMap::new();
    lang.insert("language".to_string(), "javascript".to_string());
    Template::render("index", lang)
}

#[get("/<id>", format = "text/plain", rank = 1)]
fn retrievepaste(id: &RawStr) -> Option<File> {
    print!("{}", id);
    let filename = format!("pastes/{id}", id = id);
    File::open(&filename).ok()
}

#[post("/", data = "<paste>")]
fn upload(paste: Data) -> Result<String, io::Error> {
    let id = PasteId::new(7);
    let filename = format!("pastes/{id}", id = id);
    paste.stream_to_file(Path::new(&filename))?;
    Ok(format!("{}", id))
}

#[get("/<path..>", rank = 1)]
pub fn staticfiles(path: PathBuf) -> std::io::Result<NamedFile> {
    let static_path = PathBuf::from("views/assets/");
    let resource_path = static_path.join(path);
    NamedFile::open(resource_path)
}

fn main() {
    // print!("{}", PasteId::new(7));
    fs::create_dir_all("pastes").unwrap();
    rocket::ignite()
        .mount(
            "/",
            routes![index, upload, retrieveindex, retrievepaste, favicon],
        )
        .mount(
            "/static",
            routes! {
              staticfiles,
            },
        )
        .attach(Template::fairing())
        .launch();
}
