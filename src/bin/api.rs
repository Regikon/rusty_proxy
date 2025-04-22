use axum::{routing::get, Router};

use dotenv::dotenv;
use log::{info, LevelFilter};
use rusty_proxy::api::handlers::{get_reqresp_by_id, get_reqresps_list, resend_request, scan_xss};
use rusty_proxy::api::AppState;
use rusty_proxy::config::Config;
use rusty_proxy::scanner::SimpleScanner;
use rusty_proxy::storage::mongodb_storage::MongoDbStorage;
use simplelog::SimpleLogger;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let config = Config::from_env()?;

    let client = mongodb::Client::with_uri_str(config.mongodb_uri()).await?;
    let db = Arc::new(MongoDbStorage::new(client));
    let scanner = SimpleScanner {};
    let app_state = Arc::new(AppState::new(db, scanner));

    SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default()).unwrap();
    let app = Router::new()
        .route("/requests", get(get_reqresps_list))
        .route("/requests/{reqresp_id}", get(get_reqresp_by_id))
        .route("/repeat/{reqresp_id}", get(resend_request))
        .route("/scan/{reqresp_id}", get(scan_xss))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    info!("Api listening on 0.0.0.0:8000");
    axum::serve(listener, app).await?;
    Ok(())
}
