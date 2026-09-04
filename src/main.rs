use anyhow::Result;
use backend_for_website::{config::Config, run};

#[actix_web::main]
async fn main() -> Result<()> {
    let conf = Config::new("config/config.local.toml")
        .expect("failed to parse config");

    println!("conf: {:?}", conf);

    println!("starting backend");
    run(&conf).await
}
