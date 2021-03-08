use askama_tide::askama::Template;
use tide::{prelude::*, Request};
use tide::{Body, Response};

use async_std::{
    fs::{self, OpenOptions},
    io,
};
use std::{borrow::Cow, fmt};
use tide::StatusCode;

use nanoid;

#[derive(Template)]
#[template(path = "index.html")]
struct Index<'a> {
    language: &'a str,
}

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

#[derive(Serialize, Deserialize)]
struct PasteBody {
    meta: String,
    content: String,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    tide::log::start();
    fs::create_dir_all("pastes").await?;

    let mut app = tide::new();
    app.at("/").get(index);
    app.at("/").post(create_paste);
    app.at("/:id").get(retrieve_paste);
    app.at("/static").serve_dir("static/")?;
    app.at("/favicon.ico")
        .serve_file("static/img/favicon.ico")?;

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn index(_req: Request<()>) -> Result<Response, tide::Error> {
    let param = "";
    let res: Response = Index { language: param }.into();
    Ok(res)
}

async fn retrieve_paste(req: Request<()>) -> Result<Response, tide::Error> {
    let paste_id = req.param("id")?;
    let file = fs::read_to_string(format!("pastes/{}", paste_id)).await?;
    let paste: PasteBody = serde_json::from_str(file.as_str())?;

    if req.header("Accept").unwrap() == "text/plain" {
        let res = Response::builder(StatusCode::Ok)
            .body(paste.content)
            .build();
        return Ok(res);
    }

    let res: Response = Index {
        language: &paste.meta,
    }
    .into();
    Ok(res)
    // let resp = Response::builder(StatusCode::Ok).body(json!(x)).build();
    // Ok(resp)
}

async fn create_paste(req: Request<()>) -> Result<Response, tide::Error> {
    let id = PasteId::new(7);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&format!("pastes/{}", id))
        .await?;
    let _bytes_written = io::copy(req, file).await?;
    // println!("Bytes written {}", bytes_written);
    // fs::write(format!("pastes/{}", id), serde_json::to_string(&to_save)?).await?;
    let resp = Response::builder(StatusCode::Ok)
        .body(Body::from_string(format!("{}", id)))
        .build();
    Ok(resp)
}
