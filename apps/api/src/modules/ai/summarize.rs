use uuid::Uuid;

use super::bulle::{self, Ask};
use crate::scene::{self, read::Shape, text};
use crate::state::AppState;

const PROMPT: &str = "Summarize this whiteboard for the people working on it, in Markdown. \
Start with a one-sentence overview, then group the content under short headings, and end with \
the decisions and open questions the board holds, if any. Write in the language the board is \
written in. Never mention shape ids, coordinates, sizes or colors.";

fn worded(shape: &Shape) -> bool {
    [&shape.text, &shape.name, &shape.title].iter().any(|value| !value.trim().is_empty())
}

/// The Markdown summary of board `board`'s live content, written by Bulle for `user_id`.
pub async fn summarize(state: &AppState, user_id: i64, board: Uuid) -> tronc::Result<String> {
    let shapes = scene::load(state, board).await?;
    if !shapes.iter().any(worded) {
        return Err(tronc::Error::invalid("the board has no text to summarize"));
    }
    let ask = Ask { prompt: PROMPT.to_owned(), input: text::render(&shapes), schema: None, max_tokens: 1500 };
    match bulle::complete(state, user_id, board, ask).await? {
        serde_json::Value::String(markdown) => Ok(markdown.trim().to_owned()),
        _ => Err(tronc::Error::unavailable("Bulle answered no summary")),
    }
}
