use crate::structs::structs::{HtmlTemplate, Index, Paste, PasteId};
use askama_axum::IntoResponse;
use axum::{
    debug_handler,
    extract::{BodyStream, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    Json,
};
use deadpool_postgres::Pool;
use futures::StreamExt;

pub async fn index() -> impl IntoResponse {
    tracing::debug!("index page");
    let template = Index {
        language: "".to_string(),
    };
    HtmlTemplate(template)
}

#[debug_handler]
pub async fn create(
    State(pool): State<Pool>,
    headers: HeaderMap,
    mut input: BodyStream,
) -> impl IntoResponse {
    tracing::debug!("create page");
    let id = PasteId::new(7);

    let conn = pool
        .get()
        .await
        .expect("Could not get connection from pool for create");
    let stmt = conn
        .prepare_cached("INSERT INTO pastes (id, content, meta) VALUES ($1, $2, $3)")
        .await
        .unwrap();

    let default_header = HeaderValue::from_static("plaintext");
    let file_type = headers.get("X-language").unwrap_or(&default_header);

    let mut buffer = Vec::new();
    while let Some(chunk) = input.next().await {
        let chunk = chunk.expect("Failed to read body chunk");
        buffer.extend_from_slice(&chunk);
    }

    let paste = Paste {
        id: id.to_string(),
        content: String::from_utf8(buffer).unwrap(),
        meta: file_type.to_str().unwrap().to_string(),
    };

    conn.execute(&stmt, &[&paste.id, &paste.content, &paste.meta])
        .await
        .unwrap();

    (StatusCode::CREATED, format!("{} {}", id, 0))
}

#[debug_handler]
pub async fn retrieve_paste(
    State(pool): State<Pool>,
    Path(paste_id): Path<String>,
    _headers: HeaderMap,
) -> impl IntoResponse {
    tracing::debug!("retrieve paste page");
    let conn = pool
        .get()
        .await
        .expect("Could not get connection from pool for retrieve_paste");
    let stmt = conn
        .prepare_cached("SELECT id,content,meta FROM pastes WHERE id = $1")
        .await
        .unwrap();

    let res = conn.query_one(&stmt, &[&paste_id]).await;

    // println!("{} {} {}", id, content, meta);

    match res {
        Ok(row) => {
            let id: String = row.get(0);
            let content: String = row.get(1);
            let meta: String = row.get(2);

            let paste = Paste {
                id: id,
                content: content,
                meta: meta,
            };

            (StatusCode::OK, Json(paste))
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(Paste {
                id: "".to_string(),
                content: "".to_string(),
                meta: "".to_string(),
            }),
        ),
    }
}

// axum 404

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
