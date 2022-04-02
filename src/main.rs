use askama::Template;
use axum::{
    extract::Path,
    // extract,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use nanoid;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, fs, net::SocketAddr};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    fs::create_dir_all("pastes/metadata").unwrap();

    // Axum:
    let app = Router::new()
        .route("/", get(index).post(create))
        .route("/api/:id", get(retrieve_paste))
        .route("/:id", get(retrieve_paste_doc))
        .nest(
            "/static",
            get_service(ServeDir::new("static")).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        .layer(TraceLayer::new_for_http());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    let template = Index {
        language: "".to_string(),
    };
    HtmlTemplate(template)
}

async fn create(input: String, headers: HeaderMap) -> impl IntoResponse {
    let id = PasteId::new(7);

    let file_type = headers.get("X-language").unwrap();
    fs::write(
        format!("pastes/metadata/{}", id),
        file_type.to_str().unwrap(),
    )
    .unwrap();

    fs::write(format!("pastes/{}", id), input).unwrap();
    (StatusCode::CREATED, format!("{} {}", id, 0)) // hardcoding bytes_written to 0 for now, calculate it later
}

// #[debug_handler]
async fn retrieve_paste(Path(paste_id): Path<String>, _headers: HeaderMap) -> impl IntoResponse {
    let content = fs::read_to_string(format!("pastes/{}", paste_id)).unwrap();
    return (StatusCode::OK, content);
}

async fn retrieve_paste_doc(Path(paste_id): Path<String>) -> impl IntoResponse {
    let metadata = fs::read_to_string(format!("pastes/metadata/{}", paste_id)).unwrap();
    let page = Index { language: metadata };
    HtmlTemplate(page)
}
