use askama::Template;
use askama_axum::{IntoResponse, Response};
use axum::{http::StatusCode, response::Html};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

// =========================== INDEX PAGE ===========================
#[derive(Template)]
#[template(path = "index.html")]
pub struct Index {
    pub language: String,
}

// =========================== HTML TEMPLATE ===========================
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

// =========================== PASTE ID ===========================

pub struct PasteId<'a>(pub Cow<'a, str>);

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

// =========================== PASTE BODY ===========================

#[derive(Debug, Serialize, Deserialize)]
pub struct Paste {
    pub id: String,
    pub meta: String,
    #[serde(serialize_with = "serialize_bytes_as_string")]
    pub content: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub input: String,
}

fn serialize_bytes_as_string<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bytes_as_string = String::from_utf8_lossy(bytes).to_string();
    serializer.serialize_str(&bytes_as_string)
}
