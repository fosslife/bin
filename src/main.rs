#[macro_use]
extern crate actix_web;

use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use actix_files::{Files, NamedFile};
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::fs;
use nanoid::nanoid;

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<NamedFile, Error> {
    Ok(NamedFile::open("public/favicon.ico")?)
}

#[post("/create")]
async fn save_file(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    let filename: String = nanoid!(7);
    let filepath = format!("./pastes/{}", sanitize_filename::sanitize(&filename));
    let mut file = fs::File::create(filepath).await?;
    while let Some(item) = payload.next().await {
        let item = item?;
        file.write_all(&item).await?;
    }
    Ok(HttpResponse::Ok().body(format!("{}", filename)).into())
}

#[get("/b/{filename}")]
async fn get_file(filename: web::Path<String>) -> Result<HttpResponse, Error> {
    let content = fs::read_to_string(format!("./pastes/{}", filename)).await?;
    Ok(HttpResponse::Ok().body(content).into())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    fs::create_dir_all("./pastes").await?;

    let ip = "0.0.0.0:3000";

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(save_file)
            .service(get_file)
            .service(Files::new("/", "public").index_file("index.html"))
    })
    .workers(1)
    .bind(ip)?
    .run()
    .await
}
