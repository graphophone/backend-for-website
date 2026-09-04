use anyhow::Result;
use backend_for_website::{config::Config, run};

#[tokio::main]
async fn main() -> Result<()> {
    let conf = Config::new("config/config.local.toml")
        .expect("failed to parse config");

    run(&conf).await
}
