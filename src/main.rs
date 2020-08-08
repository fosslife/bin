use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::StreamExt;
// use std::time::Instant;
use actix_files::Files;
use async_std::prelude::*;
use nanoid::nanoid;

async fn save_file(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    // let now = Instant::now();
    // file creation
    let filename: String = nanoid!(7);
    let filepath = format!("./pastes/{}", sanitize_filename::sanitize(&filename));
    let mut file = async_std::fs::File::create(filepath).await?;
    // _file.write_all(&data).await?;

    // get data stream
    // let headers=  payload.headers().get("");
    // let mut bytes = web::BytesMut::new();
    while let Some(item) = payload.next().await {
        let item = item?;
        file.write_all(&item).await?;
        // bytes.extend_from_slice(&item);
    }
    // println!("Time taken: {}ms", now.elapsed().as_millis());
    Ok(HttpResponse::Ok().body(format!("{}", filename)).into())
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    async_std::fs::create_dir_all("./pastes").await?;

    let ip = "0.0.0.0:3000";

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/create").route(web::post().to(save_file)))
            .service(Files::new("/", "./public/").index_file("index.html"))
    })
    .bind(ip)?
    .run()
    .await
}
