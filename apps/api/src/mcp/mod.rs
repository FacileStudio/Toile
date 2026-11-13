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
use std::sync::Arc;

use axum::Router;
use serde::Serialize;
use tronc::mcp::{
    Call, ConfirmConfig, Confirmations, Options, PostgresPending, ToolError, decode_cursor, encode_cursor,
    encoded_size, fit_count,
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

/// One page of `items` from the opaque `cursor` a previous page gave, at most `limit` long and
/// within the result budget (never empty while items remain), and the cursor of the next page
/// when more remain.
pub fn page<T: Serialize>(
    items: Vec<T>,
    cursor: Option<&str>,
    limit: Option<usize>,
) -> Result<(Vec<T>, Option<String>), ToolError> {
    let start: usize = cursor.map(decode_cursor).transpose()?.flatten().unwrap_or(0);
    let limit = limit.unwrap_or(PAGE).clamp(1, MAX_PAGE);
    let mut rest: Vec<T> = items.into_iter().skip(start).collect();
    let fitted = fit_count(rest.len().min(limit), |n| encoded_size(&rest[..n])).max(1).min(rest.len());
    let next = (rest.len() > fitted).then(|| encode_cursor(&(start + fitted)));
    rest.truncate(fitted);
    Ok((rest, next))
}

/// The origin of `resource`, `https://toile.facile.studio` for `https://toile.facile.studio/mcp`.
pub fn origin(resource: &str) -> &str {
    resource.strip_suffix("/mcp").unwrap_or(resource)
}

/// The confirmation flow: actions in `mcp_actions`, confirmed on the client's `/pending` page.
pub fn confirmations(state: &AppState, resource: &str, key: &[u8]) -> Result<Confirmations, String> {
    Confirmations::new(ConfirmConfig {
        store: Arc::new(PostgresPending::new(state.pool.clone())),
        key: key.to_vec(),
        confirm_url: format!("{}/pending", origin(resource)),
        kinds: HashMap::from([(KIND.to_owned(), delete::kind(state))]),
        pending_ttl: std::time::Duration::ZERO,
        ask_ttl: std::time::Duration::ZERO,
    })
}

/// Mounts `/mcp` and its metadata on `router`, outside `/api` and outside the CSRF rule.
pub fn mount(
    router: Router,
    state: &AppState,
    options: Options,
    confirmations: Confirmations,
) -> Result<Router, String> {
    let mut options = options;
    let origin = origin(&options.resource).to_owned();
    options.title = "Toile".to_owned();
    options.version = crate::version::VERSION.to_owned();
    options.instructions = "Toile holds whiteboards: stickies, text, boxes, arrows and frames on an infinite canvas, \
        a list per space. Call toile_list_boards for ids, then toile_read_board before you change a board: it lists \
        every shape with its id, grouped by frame. Writes show up live for everyone on the board. \
        toile_list_comments shows the open threads to answer with toile_comment or close with toile_resolve_comment."
        .to_owned();
    options.tools = tools::all(state, &confirmations, &origin);
    tronc::mcp::mount(router, options)
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_origin_drops_the_mcp_path_and_board_ids_must_be_uuids() {
        assert_eq!(super::origin("https://toile.facile.studio/mcp"), "https://toile.facile.studio");
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
