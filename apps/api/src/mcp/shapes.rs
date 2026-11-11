use std::sync::{Arc, Mutex};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

use super::{board, user};
use crate::modules::boards::access::{self, Access};
use crate::scene::change::{self, Change};
use crate::scene::input::{NewShape, Placed};
use crate::scene::{self, plan, read, write};
use crate::state::AppState;

/// `toile_add_shapes`.
#[derive(Deserialize, JsonSchema)]
pub struct AddIn {
    /// The board id.
    pub board_id: String,
    /// The shapes, in any order: frames are placed first, arrows last.
    pub shapes: Vec<NewShape>,
}

/// What `toile_add_shapes` answers.
#[derive(Serialize, JsonSchema)]
pub struct AddOut {
    /// Every new shape with its id and where it landed.
    pub created: Vec<Placed>,
}

/// Plans and writes `shapes` into board `id` in one room transaction, so the layout sees the
/// live board; answers where each landed.
pub async fn add_to(state: &AppState, id: uuid::Uuid, shapes: Vec<NewShape>) -> tronc::Result<Vec<Placed>> {
    let out = Arc::new(Mutex::new(Vec::new()));
    let sink = out.clone();
    let edit = Box::new(move |txn: &mut yrs::TransactionMut, map: &yrs::MapRef| {
        let planned = plan::plan(&read::all(txn, map), shapes, scene::fresh_id)?;
        write::insert(txn, map, &planned);
        *sink.lock().unwrap_or_else(|p| p.into_inner()) = planned.into_iter().map(|p| p.placed).collect();
        Ok(())
    });
    state.hub.write(state, id, edit).await?;
    Ok(std::mem::take(&mut *out.lock().unwrap_or_else(|p| p.into_inner())))
}

/// `toile_add_shapes`: new shapes, placed live.
pub async fn add(state: AppState, call: Call, input: AddIn) -> Result<Answer<AddOut>, ToolError> {
    let id = board(&input.board_id)?;
    access::check(&state, user(&call)?, id, Access::Editor).await?;
    if input.shapes.is_empty() {
        return Err(ToolError("give at least one shape".to_owned()));
    }
    Ok(AddOut { created: add_to(&state, id, input.shapes).await? }.into())
}

/// `toile_update_shapes`.
#[derive(Deserialize, JsonSchema)]
pub struct UpdateIn {
    /// The board id.
    pub board_id: String,
    /// The changes, all applied or none.
    pub shapes: Vec<Change>,
}

/// What `toile_update_shapes` answers.
#[derive(Serialize, JsonSchema)]
pub struct UpdateOut {
    /// The shapes changed, as `toile_read_board` shows them now.
    pub shapes: String,
}

/// `toile_update_shapes`: moves, resizes, relabels, recolors or reframes shapes, live.
pub async fn update(state: AppState, call: Call, input: UpdateIn) -> Result<Answer<UpdateOut>, ToolError> {
    let id = board(&input.board_id)?;
    access::check(&state, user(&call)?, id, Access::Editor).await?;
    let ids: Vec<String> = input.shapes.iter().map(|c| c.id.clone()).collect();
    let changes = input.shapes;
    let edit = Box::new(move |txn: &mut yrs::TransactionMut, map: &yrs::MapRef| change::apply(txn, map, &changes));
    state.hub.write(&state, id, edit).await?;
    let now = scene::load(&state, id).await?;
    let lines: Vec<String> = now.iter().filter(|s| ids.contains(&s.id)).map(scene::text::line).collect();
    Ok(UpdateOut { shapes: lines.join("\n") }.into())
}
