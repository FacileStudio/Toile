use tronc::mcp::{Tool, tool};

use super::tools::{reading, spec, writing};
use super::{comment, comments, manage};
use crate::state::AppState;

/// `toile_create_board` and `toile_update_board`.
pub fn boards(state: &AppState, origin: &str) -> Vec<Tool> {
    let (a, b, o1, o2) = (state.clone(), state.clone(), origin.to_owned(), origin.to_owned());
    vec![
        tool(
            spec(
                "toile_create_board",
                "Create a board",
                "Call when the person wants a new, empty board: in their personal boards, or in a space (space_id \
                 from toile_list_boards) where they may edit. Answers its id and link.",
                writing(false, false),
            ),
            move |call, input| manage::create(a.clone(), o1.clone(), call, input),
        ),
        tool(
            spec(
                "toile_update_board",
                "Rename or move a board",
                "Call to rename a board (editor) or move it to another space or to `personal` (owner, and editor in \
                 the destination). A move reconnects everyone on the board under their new access.",
                writing(true, true),
            ),
            move |call, input| manage::update(b.clone(), o2.clone(), call, input),
        ),
    ]
}

/// `toile_trash_board` and `toile_restore_board`.
pub fn trash(state: &AppState, origin: &str) -> Vec<Tool> {
    let (a, b, o1, o2) = (state.clone(), state.clone(), origin.to_owned(), origin.to_owned());
    vec![
        tool(
            spec(
                "toile_trash_board",
                "Trash a board",
                "Call when the person wants a board gone (owner only). It moves to the trash, closes for everyone on \
                 it, and is deleted for good after 30 days unless toile_restore_board brings it back.",
                writing(true, true),
            ),
            move |call, input| manage::trash(a.clone(), o1.clone(), call, input),
        ),
        tool(
            spec(
                "toile_restore_board",
                "Restore a board",
                "Call to bring a trashed board back (owner only), within 30 days of trashing it.",
                writing(false, true),
            ),
            move |call, input| manage::restore(b.clone(), o2.clone(), call, input),
        ),
    ]
}

fn listing(state: &AppState) -> Tool {
    let a = state.clone();
    tool(
        spec(
            "toile_list_comments",
            "List comments",
            "Call to see what people said on a board: its threads with replies, where each is pinned, open ones by \
             default (status: open, resolved, all), a page at a time (limit, cursor).",
            reading(),
        ),
        move |call, input| comments::list(a.clone(), call, input),
    )
}

/// `toile_list_comments`, `toile_comment` and `toile_resolve_comment`.
pub fn comments(state: &AppState) -> Vec<Tool> {
    let (a, b) = (state.clone(), state.clone());
    vec![
        listing(state),
        tool(
            spec(
                "toile_comment",
                "Comment on a board",
                "Call to leave a note on a board: pins a comment on a shape or a board point, or replies in a thread \
                 (reply_to). Everyone on the board sees it live.",
                writing(false, false),
            ),
            move |call, input| comment::comment(a.clone(), call, input),
        ),
        tool(
            spec(
                "toile_resolve_comment",
                "Resolve a comment",
                "Call once a thread is dealt with: marks it resolved, or reopens it with resolved false. Everyone on \
                 the board sees it live.",
                writing(false, true),
            ),
            move |call, input| comments::resolve(b.clone(), call, input),
        ),
    ]
}
