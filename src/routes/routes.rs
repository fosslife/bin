use crate::structs::structs::{HtmlTemplate, Index, Paste, PasteId};
use askama_axum::IntoResponse;
use axum::{
    debug_handler,
    extract::{BodyStream, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    Json,
};
use deadpool_sqlite::Pool;
use futures::StreamExt;
use rusqlite::params;

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

    let default_header = HeaderValue::from_static("plaintext");
    let file_type = headers.get("X-language").unwrap_or(&default_header);

    let mut buffer = Vec::new();
    while let Some(chunk) = input.next().await {
        let chunk = chunk.expect("Failed to read body chunk");
        buffer.extend_from_slice(&chunk);
    }

    let paste = Paste {
        id: id.to_string(),
        content: buffer,
        meta: file_type.to_str().unwrap().to_string(),
    };

    conn.interact(move |conn| {
        let mut stmt = conn
            .prepare("INSERT INTO pastes (id, content, meta) VALUES (?, ?, ?)")
            .expect("Failed creating prepared statement for create");

        stmt.execute(params![paste.id, paste.content, paste.meta])
            .expect("Failed inserting records");
    })
    .await
    .expect("Something went wrong with the database");

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
    let res = conn
        .interact(move |conn| {
            let mut stmt = conn
                .prepare("SELECT id, content, meta FROM pastes WHERE id = ?")
                .expect("Failed preparing statement");

            let mut rowsiter = stmt
                .query_map(params![paste_id], |row| {
                    Ok(Paste {
                        id: row.get(0)?,
                        content: row.get(1)?,
                        meta: row.get(2)?,
                    })
                })
                .expect("Failed querying map");
            match rowsiter.next() {
                Some(Ok(paste)) => Ok(paste),
                Some(Err(e)) => Err(e),
                None => Err(rusqlite::Error::QueryReturnedNoRows),
            }
            // tracing::debug!("rowsiter: {:?}", rowsiter.next());
            // let row = rowsiter.next().unwrap();
            // row
        })
        .await
        .expect("Something went wrong with the database in retrieve_paste");

    match res {
        Ok(content) => (StatusCode::OK, Json(content)),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(Paste {
                id: "".to_string(),
                content: vec![],
                meta: "".to_string(),
            }),
        ),
    }
}

// axum 404

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here")
}
