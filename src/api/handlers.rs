use crate::storage::storage::ReqrespStorage;
use axum::response::IntoResponse;

use super::{AppState, Reqresp};
use crate::scanner::Scanner;
use axum::extract::Path;
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub async fn get_reqresps_list(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<Reqresp>>) {
    let reqresps = state.db().get_reqresps().await.unwrap();
    return (StatusCode::OK, Json(reqresps));
}

pub async fn get_reqresp_by_id(
    State(state): State<Arc<AppState>>,
    Path(reqresp_id): Path<String>,
) -> (StatusCode, Json<Option<Reqresp>>) {
    let reqresp = state.db().get_reqresp_by_id(&reqresp_id).await.unwrap();
    let result = match reqresp {
        Some(reqresp) => (StatusCode::OK, Json(Some(reqresp))),
        None => (StatusCode::NOT_FOUND, Json(None)),
    };
    result
}

pub async fn resend_request(
    State(state): State<Arc<AppState>>,
    Path(reqresp_id): Path<String>,
) -> hyper::Response<axum::body::Body> {
    let reqresp = match state.db().get_reqresp_by_id(&reqresp_id).await.unwrap() {
        Some(reqresp) => reqresp,
        None => {
            return (http::StatusCode::NOT_FOUND, axum::response::Html::from("")).into_response()
        }
    };
    state
        .scanner()
        .resend_request(reqresp.req)
        .await
        .unwrap()
        .into_response()
}
