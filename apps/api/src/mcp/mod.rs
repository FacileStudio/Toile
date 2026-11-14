//! `/mcp`: curated tools over boards, shapes and comments, `toile_delete_shapes` behind a
//! confirmation.
//! Free for every account: no plan gate.

mod boards;
mod catalog;
mod comment;
mod comments;
mod delete;
mod manage;
mod shapes;
mod tools;

use std::collections::HashMap;

use axum::Router;
use porte::mcpauth::require_own_session;
use serde::Serialize;
use tronc::mcp::{
    App, AppConfig, AppSpec, Call, Confirmations, PendingItem, PendingPage, ToolError, cursor, encoded_size, fit_count,
    offset,
};
use uuid::Uuid;

use crate::state::AppState;

pub use delete::KIND;

/// The porte user id of the caller.
pub fn user(call: &Call) -> Result<i64, ToolError> {
    call.caller.user_id.parse().map_err(|_| ToolError("the token names no local account".to_owned()))
}

/// A board id from a tool argument; anything else names no board.
pub fn board(raw: &str) -> Result<Uuid, ToolError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ToolError(format!("{raw:?} is not a board id; call toile_list_boards")))
}

/// The most items a list answers when the call names no `limit`.
pub const PAGE: usize = 50;
/// The most items a list answers in one call.
pub const MAX_PAGE: usize = 200;

/// One page of `items` from the opaque cursor `at` a previous page gave, at most `limit` long and
/// within the result budget (never empty while items remain), and the cursor of the next page
/// when more remain.
pub fn page<T: Serialize>(
    items: Vec<T>,
    at: Option<&str>,
    limit: Option<usize>,
) -> Result<(Vec<T>, Option<String>), ToolError> {
    let total = items.len();
    let start = offset(at.unwrap_or_default())?.min(total);
    let limit = limit.unwrap_or(PAGE).clamp(1, MAX_PAGE);
    let mut rest: Vec<T> = items.into_iter().skip(start).collect();
    let fitted = fit_count(rest.len().min(limit), |n| encoded_size(&rest[..n])).max(1).min(rest.len());
    let next = cursor(start + fitted, total);
    rest.truncate(fitted);
    Ok((rest, (!next.is_empty()).then_some(next)))
}

const INSTRUCTIONS: &str = "Toile holds whiteboards: stickies, text, boxes, arrows and frames on an infinite canvas, \
    a list per space. Call toile_list_boards for ids, then toile_read_board before you change a board: it lists every \
    shape with its id, grouped by frame. Writes show up live for everyone on the board. toile_list_comments shows the \
    open threads to answer with toile_comment or close with toile_resolve_comment.";

/// Toile's MCP server from the deployment `config`: the curated tools, `toile_delete_shapes`
/// behind a confirmation on the client's `/pending` page. It sets `state.confirmations`, which
/// the confirm page's API reads.
pub fn app(state: &mut AppState, config: AppConfig) -> Result<App, String> {
    let spec = AppSpec {
        name: "toile".to_owned(),
        title: "Toile".to_owned(),
        instructions: INSTRUCTIONS.to_owned(),
        kinds: HashMap::from([(KIND.to_owned(), delete::kind(state))]),
        ..AppSpec::default()
    };
    let mut app = App::new(AppConfig { version: crate::version::VERSION.to_owned(), ..config }, spec)?;
    let confirmations = app.confirmations.clone().ok_or("mcp: toile_delete_shapes needs confirmations")?;
    app.options.tools = tools::all(state, &confirmations, &app.url);
    state.confirmations = Some(confirmations);
    Ok(app)
}

/// A confirm page entry: the action and what it targets, `{"board_id", "ids"}`.
#[derive(Serialize)]
struct Pending {
    #[serde(flatten)]
    item: PendingItem,
    target: String,
}

/// `GET /mcp/pending`, `POST /mcp/pending/{id}/confirm|dismiss`, for `/api` behind
/// `require_auth`. Only the person's own browser session confirms; `None` answers 404.
pub fn pending(confirmations: Option<Confirmations>) -> Router<AppState> {
    PendingPage::new(confirmations, "toile-web", |parts: http::request::Parts| async move {
        require_own_session(&parts).map(|identity| identity.user_id.to_string())
    })
    .describe(|item, action| async move { Pending { item, target: action.target } })
    .routes()
}

#[cfg(test)]
mod tests {
    #[test]
    fn board_ids_must_be_uuids() {
        assert!(super::board("nope").unwrap_err().0.contains("toile_list_boards"));
    }

    #[test]
    fn pages_follow_their_cursor_stop_at_the_budget_and_the_last_one_has_none() {
        let (first, next) = super::page((0..5).collect(), None, Some(2)).unwrap();
        let (second, next) = super::page((0..5).collect(), next.as_deref(), Some(2)).unwrap();
        let (last, next) = super::page((0..5).collect(), next.as_deref(), Some(2)).unwrap();
        assert_eq!((first, second, last, next), (vec![0, 1], vec![2, 3], vec![4], None));
        assert!(super::page(vec![1], Some("x"), None).unwrap_err().0.contains("cursor"));
        let big = "x".repeat(tronc::mcp::RESULT_BUDGET / 3);
        let (first, next) = super::page(vec![big.clone(); 5], None, None).unwrap();
        assert_eq!(first.len(), 2);
        let (second, _) = super::page(vec![big.clone(); 5], next.as_deref(), None).unwrap();
        assert_eq!(second.len(), 2);
        let (alone, next) = super::page(vec![big.repeat(4), big], None, None).unwrap();
        assert_eq!((alone.len(), next.is_some()), (1, true));
    }
}
