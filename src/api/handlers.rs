use crate::storage::storage::ReqrespStorage;
use log::debug;

use super::{AppState, Reqresp};
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn get_reqresps_list(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<Reqresp>>) {
    let reqresps = state.db.get_reqresps().await.unwrap();
    return (StatusCode::OK, Json(reqresps));
}

pub async fn get_reqresp_by_id(
    State(state): State<Arc<AppState>>,
    Path(reqresp_id): Path<String>,
) -> (StatusCode, Json<Option<Reqresp>>) {
    debug!("Id is {:?}", reqresp_id);
    let reqresp = state.db.get_reqresp_by_id(&reqresp_id).await.unwrap();
    let result = match reqresp {
        Some(reqresp) => (StatusCode::OK, Json(Some(reqresp))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    };
    result
}
