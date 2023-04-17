mod routes;
mod structs;

use axum::{
    http::{HeaderValue, Method},
    routing::{get, get_service, post},
    Router,
};
use deadpool_postgres::Runtime;
use dotenv::dotenv;
use routes::routes::*;
use std::net::SocketAddr;
use tokio::signal;
use tokio_postgres::NoTls;
use tower_http::{cors, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Config {
    listen: String,
    pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "binrs=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = Config::from_env().unwrap();

    let pool = cfg.pg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    //initial DB setup
    let conn = pool.get().await.unwrap();
    let stmt = conn
        .prepare_cached(
            "CREATE TABLE IF NOT EXISTS pastes (id TEXT PRIMARY KEY, content TEXT, meta TEXT);",
        )
        .await
        .unwrap();

    conn.execute(&stmt, &[]).await.unwrap();

    tracing::debug!("DB setup complete");

    let cors = cors::CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::POST])
        // allow requests from any origin
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_origin("https://tauri.localhost".parse::<HeaderValue>().unwrap());

    // Axum:
    let app = Router::new()
        .route("/", get(index))
        .route("/:id", get(index))
        .route("/api/create", post(create))
        .route("/api/:id", get(retrieve_paste))
        .nest_service("/static", get_service(ServeDir::new("static")))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(pool);
    let app = app.fallback(handler_404);

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
