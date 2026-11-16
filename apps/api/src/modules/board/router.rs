use axum::Router;
use axum::routing::get;
use planche::keys::BOARD_TYPES;
use planche::{MAX_COORD, MAX_POINTS, MAX_SHAPES, MAX_TEXT, MAX_UPDATE, SCHEMA};
use serde_json::{Value, json};
use tronc::json::Json;

use crate::state::AppState;

async fn describe() -> Json<Value> {
    Json(json!({
        "schema": SCHEMA,
        "types": BOARD_TYPES,
        "limits": {
            "shapes": MAX_SHAPES,
            "ink_points": MAX_POINTS,
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
