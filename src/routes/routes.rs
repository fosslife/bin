use std::{collections::HashMap, sync::Arc};

use crate::{
    structs::structs::{HtmlTemplate, Index, Paste, PasteId},
    RedisConnection,
};
use askama_axum::IntoResponse;
use axum::{
    debug_handler,
    extract::{BodyStream, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    Json,
};
use futures::StreamExt;
use redis::{Commands, RedisResult};

pub async fn index() -> impl IntoResponse {
    tracing::debug!("index page");
    let template = Index {
        language: "".to_string(),
    };
    HtmlTemplate(template)
}

#[debug_handler]
pub async fn create(
    State(pool): State<Arc<RedisConnection>>,
    headers: HeaderMap,
    mut input: BodyStream,
) -> impl IntoResponse {
    tracing::debug!("create page");
    let id = PasteId::new(7);

    let mut conn = pool.clone().client.get_connection().unwrap();

    let default_header = HeaderValue::from_static("plaintext");
    let file_type = headers.get("X-language").unwrap_or(&default_header);

    let mut buffer = Vec::new();
    while let Some(chunk) = input.next().await {
        let chunk = chunk.expect("Failed to read body chunk");
        buffer.extend_from_slice(&chunk);
    }
    let content = String::from_utf8(buffer).unwrap();
    let meta = file_type.to_str().unwrap().to_string();

    let _: () = conn
        .hset(format!("paste:{}", id), "content", content)
        .unwrap();
    let _: () = conn.hset(format!("paste:{}", id), "meta", meta).unwrap();

    (StatusCode::CREATED, format!("{} {}", id, 0))
}

#[debug_handler]
pub async fn retrieve_paste(
    State(pool): State<Arc<RedisConnection>>,
    Path(paste_id): Path<String>,
    _headers: HeaderMap,
) -> impl IntoResponse {
    tracing::debug!("retrieve paste page");
    let mut conn = pool.clone().client.get_connection().unwrap();

    let res: RedisResult<String> = conn.hgetall(format!("paste:{}", paste_id));
    let res2 = res.unwrap().clone();
    tracing::error!("{}", res.unwrap());
    if let Ok(value) = res2 {
        Json(value)
    } else {
        // let empty: HashMap<String, String> = HashMap::new();
        Json("".to_string())
    }
}

// axum 404

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
