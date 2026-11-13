use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::{boards, catalog, delete, shapes};
use crate::state::AppState;

pub(super) fn spec(name: &str, title: &str, what: &str, annotations: Annotations) -> ToolSpec {
    ToolSpec { name: name.into(), title: title.into(), description: what.into(), annotations: Some(annotations) }
}

pub(super) fn reading() -> Annotations {
    Annotations { read_only: true, destructive: false, idempotent: true, open_world: false }
}

pub(super) fn writing(destructive: bool, idempotent: bool) -> Annotations {
    Annotations { read_only: false, destructive, idempotent, open_world: false }
}

fn reads(state: &AppState, origin: &str) -> Vec<Tool> {
    let (a, b, o1, o2) = (state.clone(), state.clone(), origin.to_owned(), origin.to_owned());
    vec![
        tool(
            spec(
                "toile_list_boards",
                "List boards",
                "Call first to find a board id or a space id. Your boards, personal and per space, with roles, newest \
                 change first, a page at a time (limit, cursor), and your spaces.",
                reading(),
            ),
            move |call, input| boards::list(a.clone(), o1.clone(), call, input),
        ),
        tool(
            spec(
                "toile_read_board",
                "Read a board",
                "Call before you change a board's shapes: every shape as one text line (type, id, position, size, \
                 color, text), grouped by frame, plus links to open it and to its SVG export. A huge board is cut and \
                 flagged truncated; open the SVG for the rest.",
                reading(),
            ),
            move |call, input| boards::read(b.clone(), o2.clone(), call, input),
        ),
    ]
}

fn edits(state: &AppState) -> Vec<Tool> {
    let (a, b) = (state.clone(), state.clone());
    vec![
        tool(
            spec(
                "toile_add_shapes",
                "Add shapes",
                "Call to draw on a board: stickies, text, boxes (rect, ellipse, diamond), frames and arrows in one go. \
                 Leave out x and y and Toile lays shapes out: inside their frame, or to the right of the board's \
                 content. Give shapes a ref so arrows (from, to) and children (frame) can point at them in the same \
                 call. Open editors see them live.",
                writing(false, false),
            ),
            move |call, input| shapes::add(a.clone(), call, input),
        ),
        tool(
            spec(
                "toile_update_shapes",
                "Change shapes",
                "Call to move, resize, relabel, recolor or reframe shapes by id; overwrites those fields, all changes \
                 apply or none. Moving a frame moves what is inside it. Open editors see it live.",
                writing(true, true),
            ),
            move |call, input| shapes::update(b.clone(), call, input),
        ),
    ]
}

fn deletes(state: &AppState, confirmations: &Confirmations) -> Tool {
    let (c, confirm) = (state.clone(), confirmations.clone());
    tool(
        spec(
            "toile_delete_shapes",
            "Delete shapes",
            "Call to remove shapes by id; a frame takes its contents along. The person confirms first; nothing is \
             deleted until they do.",
            writing(true, false),
        ),
        move |call, input| delete::delete(c.clone(), confirm.clone(), call, input),
    )
}

/// Every tool: reads, shape edits, board management and comments.
pub fn all(state: &AppState, confirmations: &Confirmations, origin: &str) -> Vec<Tool> {
    let mut tools = reads(state, origin);
    tools.extend(edits(state));
    tools.push(deletes(state, confirmations));
    tools.extend(catalog::boards(state, origin));
    tools.extend(catalog::trash(state, origin));
    tools.extend(catalog::comments(state));
    tools
}
