use axum::{routing::get, Router};

use dotenv::dotenv;
use log::{info, LevelFilter};
use rusty_proxy::api::handlers::{hello, reqresps_list};
use rusty_proxy::api::AppState;
use rusty_proxy::config::Config;
use rusty_proxy::storage::mongodb_storage::MongoDbStorage;
use simplelog::SimpleLogger;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let config = Config::from_env()?;

    let client = mongodb::Client::with_uri_str(config.mongodb_uri()).await?;
    let db = Arc::new(MongoDbStorage::new(client));
    let app_state = Arc::new(AppState::new(db));

    SimpleLogger::init(LevelFilter::Info, simplelog::Config::default()).unwrap();
    let app = Router::new()
        .route("/", get(hello))
        .route("/requests/", get(reqresps_list))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    info!("Api listening on 0.0.0.0:8000");
    axum::serve(listener, app).await?;
    Ok(())
}
