use askama::Template;
use axum::{
    // debug_handler,
    extract::{BodyStream, Path},
    // extract,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use nanoid;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, fs, io::prelude::*, net::SocketAddr};
use tokio::signal;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use futures::StreamExt;

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    language: String,
}

struct HtmlTemplate<T>(T);

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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "binrs=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    fs::create_dir_all("pastes/metadata").expect("Failed creating initial storage directories");

    // Axum:
    let app = Router::new()
        .route("/", get(index).post(create))
        .route("/api/:id", get(retrieve_paste))
        .route("/:id", get(retrieve_paste_doc))
        .nest_service("/static", get_service(ServeDir::new("static")))
        .layer(TraceLayer::new_for_http());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("user interrupt. shutting server down");
}

async fn index() -> impl IntoResponse {
    let template = Index {
        language: "".to_string(),
    };
    HtmlTemplate(template)
}

#[derive(Debug, Serialize, Deserialize)]
struct Input {
    input: String,
}

// #[debug_handler]
async fn create(headers: HeaderMap, mut input: BodyStream) -> impl IntoResponse {
    let id = PasteId::new(7);

    let def = HeaderValue::from_static("plaintext");
    if headers.contains_key("X-language") {
        let file_type = headers.get("X-language").unwrap_or(&def);
        fs::write(
            format!("pastes/metadata/{}", id),
            file_type.clone().to_str().unwrap(),
        )
        .expect("Failed to write file");
    }
    let pastefilename = format!("pastes/{}", id);
    let mut pastefile = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(pastefilename)
        .expect("Failed to open file in append mode");

    while let Some(chunk) = input.next().await {
        let chunk = chunk.unwrap();
        pastefile.write(&chunk).expect("Failed to write file");
    }

    (StatusCode::CREATED, format!("{} {}", id, 0))
}

// #[debug_handler]
async fn retrieve_paste(Path(paste_id): Path<String>, _headers: HeaderMap) -> impl IntoResponse {
    let res = fs::read_to_string(format!("pastes/{}", paste_id));
    match res {
        Ok(content) => (StatusCode::OK, content),
        Err(_) => (StatusCode::BAD_REQUEST, "Paste Doesn't exist".to_string()),
    }
}

async fn retrieve_paste_doc(Path(paste_id): Path<String>) -> impl IntoResponse {
    let res = fs::read_to_string(format!("pastes/metadata/{}", paste_id));
    if let Ok(metadata) = res {
        let page = Index { language: metadata };
        HtmlTemplate(page)
    } else {
        HtmlTemplate(Index {
            language: "text".to_string(),
        })
    }
}
