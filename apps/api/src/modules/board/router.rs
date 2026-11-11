use axum::Router;
use axum::routing::get;
use serde_json::{Value, json};
use tronc::json::Json;

use super::schema::{MAX_COORD, MAX_INK_POINTS, MAX_SHAPES, MAX_TEXT, MAX_UPDATE};
use super::shape::TYPES;
use crate::state::AppState;

async fn describe() -> Json<Value> {
    Json(json!({
        "schema": super::SCHEMA,
        "types": TYPES,
        "limits": {
            "shapes": MAX_SHAPES,
            "ink_points": MAX_INK_POINTS,
            "text_chars": MAX_TEXT,
            "update_bytes": MAX_UPDATE,
            "coordinate": MAX_COORD,
        },
    }))
}

/// `GET /board-schema`: the schema version, shape types and limits this server enforces.
pub fn routes() -> Router<AppState> {
    Router::new().route("/board-schema", get(describe))
}
