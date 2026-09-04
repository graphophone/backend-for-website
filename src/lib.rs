use anyhow::Result;

pub mod config;

pub async fn run(conf: &config::Config) -> Result<()> {
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
    .map_err(anyhow::Error::from)
}