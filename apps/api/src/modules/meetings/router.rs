use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::service::{self, Meeting, MeetingRequest};
use crate::state::AppState;

async fn meeting(
    State(state): State<AppState>,
    who: Identity,
    Json(request): Json<MeetingRequest>,
) -> Result<(StatusCode, Json<Meeting>)> {
    let meeting = service::open(&state, who.user_id, request).await?;
    let status = if meeting.created { StatusCode::CREATED } else { StatusCode::OK };
    Ok((status, Json(meeting)))
}

/// `POST /boards/meeting`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/boards/meeting", post(meeting))
}
