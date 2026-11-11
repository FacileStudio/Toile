use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::access::Access;

/// The columns every board answer reads, as a literal for `concat!`.
macro_rules! columns {
    () => {
        "id, space_id, name, created_at, updated_at, deleted_at, thumbnail_at"
    };
}
pub(crate) use columns;

/// One `boards` row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BoardRow {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
    pub thumbnail_at: Option<OffsetDateTime>,
}

/// A board as the API answers it.
#[derive(Debug, Serialize)]
pub struct Board {
    pub id: Uuid,
    pub name: String,
    pub space_id: Option<String>,
    pub role: &'static str,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub thumbnail_url: Option<String>,
    #[serde(with = "time::serde::rfc3339::option", skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<OffsetDateTime>,
}

impl Board {
    /// `row` as answered to a caller with `access`.
    pub fn from_row(row: BoardRow, access: Access) -> Board {
        let thumbnail_url =
            row.thumbnail_at.map(|at| format!("/api/boards/{}/thumbnail.png?v={}", row.id, at.unix_timestamp()));
        Board {
            id: row.id,
            name: row.name,
            space_id: row.space_id,
            role: access.name(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            thumbnail_url,
            deleted_at: row.deleted_at,
        }
    }
}

/// `?space_id=&trash=`: the scope of a list, and whether to list its trash instead.
#[derive(Debug, Default, Deserialize)]
pub struct ScopeQuery {
    pub space_id: Option<String>,
    #[serde(default)]
    pub trash: bool,
}

/// `POST /boards`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    #[serde(default)]
    pub name: String,
    pub space_id: Option<String>,
}

/// `PATCH /boards/{id}` and `POST /boards/{id}/duplicate`: an optional new name.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NameRequest {
    pub name: Option<String>,
}

/// `DELETE /boards/{id}?permanent=true`: delete a trashed board for good.
#[derive(Debug, Default, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub permanent: bool,
}
