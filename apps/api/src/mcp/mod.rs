//! `/mcp`: six curated tools over the boards, `toile_delete_shapes` behind a confirmation.
//! Free for every account: no plan gate.

mod boards;
mod comment;
mod delete;
mod shapes;
mod tools;

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use tronc::mcp::{Call, ConfirmConfig, Confirmations, Options, PostgresPending, ToolError};
use uuid::Uuid;

use crate::state::AppState;

pub use delete::KIND;
pub use shapes::add_to;

/// The porte user id of the caller.
pub fn user(call: &Call) -> Result<i64, ToolError> {
    call.caller.user_id.parse().map_err(|_| ToolError("the token names no local account".to_owned()))
}

/// A board id from a tool argument; anything else names no board.
pub fn board(raw: &str) -> Result<Uuid, ToolError> {
    Uuid::parse_str(raw.trim()).map_err(|_| ToolError(format!("{raw:?} is not a board id; call toile_list_boards")))
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
        every shape with its id, grouped by frame. Writes show up live for everyone on the board."
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
}
