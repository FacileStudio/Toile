use tronc::mcp::{Annotations, Confirmations, Tool, ToolSpec, tool};

use super::{boards, comment, delete, shapes};
use crate::state::AppState;

fn spec(name: &str, title: &str, what: &str, annotations: Annotations) -> ToolSpec {
    ToolSpec { name: name.into(), title: title.into(), description: what.into(), annotations: Some(annotations) }
}

fn reading() -> Annotations {
    Annotations { read_only: true, destructive: false, idempotent: true, open_world: false }
}

fn writing(destructive: bool) -> Annotations {
    Annotations { read_only: false, destructive, idempotent: false, open_world: false }
}

fn reads(state: &AppState, origin: &str) -> Vec<Tool> {
    let (a, b, o1, o2) = (state.clone(), state.clone(), origin.to_owned(), origin.to_owned());
    vec![
        tool(
            spec(
                "toile_list_boards",
                "List boards",
                "Your boards, personal and per space, with ids and roles, newest change first, and your spaces.",
                reading(),
            ),
            move |call, input| boards::list(a.clone(), o1.clone(), call, input),
        ),
        tool(
            spec(
                "toile_read_board",
                "Read a board",
                "Every shape of one board as one text line each (type, id, position, size, color, text), grouped \
                 by frame, plus links to open it and to its SVG export. Read before you change a board.",
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
                "Adds stickies, text, boxes (rect, ellipse, diamond), frames and arrows in one go. Leave out x and y \
                 and Toile lays shapes out: inside their frame, or to the right of the board's content. Give shapes \
                 a ref so arrows (from, to) and children (frame) can point at them in the same call.",
                writing(false),
            ),
            move |call, input| shapes::add(a.clone(), call, input),
        ),
        tool(
            spec(
                "toile_update_shapes",
                "Change shapes",
                "Moves, resizes, relabels, recolors or reframes shapes by id; all changes apply or none. Moving a \
                 frame moves what is inside it.",
                writing(false),
            ),
            move |call, input| shapes::update(b.clone(), call, input),
        ),
    ]
}

/// The six tools.
pub fn all(state: &AppState, confirmations: &Confirmations, origin: &str) -> Vec<Tool> {
    let (a, b, confirm) = (state.clone(), state.clone(), confirmations.clone());
    let mut tools = reads(state, origin);
    tools.extend(edits(state));
    tools.push(tool(
        spec(
            "toile_delete_shapes",
            "Delete shapes",
            "Deletes shapes by id; a frame takes its contents along. The person confirms first; nothing is deleted \
             until they do.",
            writing(true),
        ),
        move |call, input| delete::delete(a.clone(), confirm.clone(), call, input),
    ));
    tools.push(tool(
        spec(
            "toile_comment",
            "Comment on a board",
            "Pins a comment on a shape or a board point, or replies in a thread. Everyone on the board sees it live.",
            writing(false),
        ),
        move |call, input| comment::comment(b.clone(), call, input),
    ));
    tools
}
