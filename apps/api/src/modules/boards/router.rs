use axum::Router;
use axum::routing::{get, post};

use super::handlers::{create, duplicate, list, read, remove, rename, restore, thumbnail};
use crate::state::AppState;

/// The board routes, mounted under `/api` behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/boards", get(list).post(create))
        .route("/boards/{id}", get(read).patch(rename).delete(remove))
        .route("/boards/{id}/duplicate", post(duplicate))
        .route("/boards/{id}/restore", post(restore))
        .route("/boards/{id}/thumbnail.png", get(thumbnail))
}
