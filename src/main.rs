#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_web::HttpServer::new(|| {
        actix_web::App::new()
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
