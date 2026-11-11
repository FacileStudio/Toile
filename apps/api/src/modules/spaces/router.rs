use axum::Router;
use axum::extract::State;
use axum::routing::get;
use porte::Identity;
use serde::Serialize;
use tronc::Result;
use tronc::json::Json;

use crate::extract::db;
use crate::modules::boards::access::member_spaces;
use crate::state::AppState;

/// One space of the caller, with the caller's membership role.
#[derive(Serialize, sqlx::FromRow)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Serialize)]
struct Spaces {
    spaces: Vec<Space>,
}

/// The live spaces `user_id` belongs to, by name.
pub async fn mine(state: &AppState, user_id: i64) -> Result<Vec<Space>> {
    sqlx::query_as::<_, Space>(concat!(
        "SELECT s.id, s.name, m.role FROM spaces s JOIN (",
        member_spaces!(),
        ") m ON m.space_id = s.id ORDER BY lower(s.name), s.id"
    ))
    .bind(user_id)
    .bind(&*state.issuer)
    .fetch_all(&state.pool)
    .await
    .map_err(db)
}

async fn list(State(state): State<AppState>, who: Identity) -> Result<Json<Spaces>> {
    Ok(Json(Spaces { spaces: mine(&state, who.user_id).await? }))
}

/// `GET /spaces`, behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new().route("/spaces", get(list))
}
