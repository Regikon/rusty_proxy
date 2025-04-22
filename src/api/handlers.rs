use crate::storage::storage::ReqrespStorage;

use super::{AppState, Reqresp};
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn hello() -> &'static str {
    "Hello, World!"
}

pub async fn reqresps_list(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Vec<Reqresp>>) {
    let reqresps = state.db.get_reqresps().await.unwrap();
    return (StatusCode::OK, Json(reqresps));
}
