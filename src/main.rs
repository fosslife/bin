use axum::{
    body::{boxed, Full},
    // debug_handler,
    extract::{BodyStream, Path},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use nanoid;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env, fmt, fs, io::prelude::*, net::SocketAddr};
use tower_http::{cors, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use futures::StreamExt;
use tokio::signal;

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
    #[cfg(debug_assertions)]
    env::set_var("RUST_LOG", "binrs=debug");

    #[cfg(not(debug_assertions))]
    env::set_var("RUST_LOG", "binrs=info");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "binrs=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    fs::create_dir_all("pastes/metadata").expect("Failed creating initial storage directories");
    let cors = cors::CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::POST])
        // allow requests from any origin
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_origin("https://tauri.localhost".parse::<HeaderValue>().unwrap());

    // Axum:
    let app = Router::new()
        .route("/", get(index).post(create))
        .route("/static/*file", get(static_handler))
        .route("/api/:id", get(retrieve_paste))
        .route("/api/:id/lang", get(retrieve_paste_doc_content))
        .route("/:id", get(retrieve_paste_doc))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .fallback_service(get(handler_404));

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
    tracing::debug!("fetching index");
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

// #[debug_handler]
async fn create(headers: HeaderMap, mut input: BodyStream) -> impl IntoResponse {
    let id = PasteId::new(7);
    tracing::debug!("creating paste: {:?}", id.0);

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

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    tracing::debug!("serving static file: {}", path);
    if path.starts_with("static/") {
        path = path.replace("static/", "");
    }

    StaticFile(path)
}

// #[debug_handler]
async fn retrieve_paste(Path(paste_id): Path<String>, _headers: HeaderMap) -> impl IntoResponse {
    let res = fs::read_to_string(format!("pastes/{}", paste_id));
    tracing::debug!("retrieve_paste: {:?}", paste_id);
    match res {
        Ok(content) => (StatusCode::OK, content),
        Err(_) => (StatusCode::BAD_REQUEST, "Paste Doesn't exist".to_string()),
    }
}

async fn retrieve_paste_doc(Path(paste_id): Path<String>) -> impl IntoResponse {
    tracing::debug!("retrieve_paste_doc: {:?}", paste_id);
    static_handler("/index.html".parse::<Uri>().unwrap()).await
}

async fn retrieve_paste_doc_content(Path(paste_id): Path<String>) -> impl IntoResponse {
    let res = fs::read_to_string(format!("pastes/metadata/{}", paste_id));
    tracing::debug!("retrieve_paste_doc_content: {:?}", paste_id);
    match res {
        Ok(content) => (StatusCode::OK, content),
        Err(_) => (StatusCode::OK, "text".to_string()),
    }
}

#[derive(RustEmbed)]
#[folder = "static/"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();

                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
