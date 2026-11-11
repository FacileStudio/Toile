use serde::Serialize;
use tronc::json::Json;

/// The release this binary was built from, stamped by the Dockerfile through `APP_VERSION`.
pub const VERSION: &str = match option_env!("APP_VERSION") {
    Some(version) => version,
    None => "dev",
};

/// The board document schema this binary reads and writes (`docs/board-schema.md`).
pub const BOARD_SCHEMA: u32 = 1;

/// What `GET /version` answers.
#[derive(Serialize)]
pub struct VersionAnswer {
    version: &'static str,
    board_schema: u32,
}

/// `GET /version`.
pub async fn handler() -> Json<VersionAnswer> {
    Json(VersionAnswer { version: VERSION, board_schema: BOARD_SCHEMA })
}
