use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::modules::board::schema::{MAX_COORD, MAX_TEXT};

/// The longest shape id an anchor names, in characters.
pub const MAX_SHAPE_ID: usize = 128;

/// Where a comment sits: a board point, optionally on a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    pub x: f64,
    pub y: f64,
}

impl Anchor {
    /// Refuses coordinates past ±[`MAX_COORD`] and a shape id over [`MAX_SHAPE_ID`] characters.
    pub fn check(&self) -> Result<()> {
        if ![self.x, self.y].iter().all(|n| n.is_finite() && n.abs() <= MAX_COORD) {
            return Err(Error::invalid(format!("anchor x and y must be within ±{MAX_COORD}")));
        }
        match self.shape.as_ref().is_some_and(|s| s.is_empty() || s.chars().count() > MAX_SHAPE_ID) {
            true => Err(Error::invalid(format!("anchor shape must be 1 to {MAX_SHAPE_ID} characters"))),
            false => Ok(()),
        }
    }
}

/// A comment body: trimmed, 1 to [`MAX_TEXT`] characters.
pub fn text(raw: &str) -> Result<String> {
    let body = raw.trim();
    match body.chars().count() {
        0 => Err(Error::invalid("body is empty")),
        n if n > MAX_TEXT => Err(Error::invalid(format!("body is longer than {MAX_TEXT} characters"))),
        _ => Ok(body.to_owned()),
    }
}

/// The columns of a [`Comment`], as a literal for `concat!`; `$2` is the caller's user id.
macro_rules! columns {
    () => {
        "c.id, c.parent_id AS parent, c.anchor, c.body, \
         COALESCE(NULLIF(u.name, ''), u.email, c.guest_name, '') AS author, c.guest_name IS NOT NULL AS guest, \
         COALESCE(c.author_user_id = $2::bigint, false) AS mine, c.created_at, c.updated_at, c.resolved_at \
         FROM comments c LEFT JOIN porte_users u ON u.id = c.author_user_id"
    };
}
pub(crate) use columns;

/// A comment as the API answers it.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub anchor: Json<Anchor>,
    pub body: String,
    pub author: String,
    pub guest: bool,
    pub mine: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub resolved_at: Option<OffsetDateTime>,
}

/// `POST /boards/{id}/comments`. A reply names its `parent` and may leave out `anchor`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    pub anchor: Option<Anchor>,
    pub body: String,
    pub parent: Option<Uuid>,
}

/// `POST /share/{token}/comments`: a guest's comment, signed with its display name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestRequest {
    pub name: String,
    pub anchor: Option<Anchor>,
    pub body: String,
    pub parent: Option<Uuid>,
}

/// `PATCH /boards/{id}/comments/{cid}`: a new body (its author only) or the resolved flag.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchRequest {
    pub body: Option<String>,
    pub resolved: Option<bool>,
}
