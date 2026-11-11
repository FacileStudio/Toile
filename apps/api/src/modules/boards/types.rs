use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// The columns every board answer reads, as a literal for `concat!`.
macro_rules! columns {
    () => {
        "id, space_id, name, created_at, updated_at"
    };
}
pub(crate) use columns;

/// One live `boards` row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BoardRow {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub name: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// A board as the API answers it.
#[derive(Debug, Serialize)]
pub struct Board {
    pub id: Uuid,
    pub space_id: Option<String>,
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub access: &'static str,
}

impl Board {
    /// `row` as answered to a caller with `access`.
    pub fn from_row(row: BoardRow, access: super::access::Access) -> Board {
        Board {
            id: row.id,
            space_id: row.space_id,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            access: access.name(),
        }
    }
}

/// `?space_id=`: the scope of a list. Absent is the personal scope.
#[derive(Debug, Default, Deserialize)]
pub struct ScopeQuery {
    pub space_id: Option<String>,
}

/// `POST /boards`.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    #[serde(default)]
    pub name: String,
    pub space_id: Option<String>,
}
