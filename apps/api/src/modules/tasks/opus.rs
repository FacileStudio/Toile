use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tronc::{Error, Result};

use crate::state::AppState;

/// The task Opus created, as Toile answers it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Task {
    /// Opus's task id.
    pub id: i64,
    /// The task's title.
    pub title: String,
    /// Its page in Opus.
    pub url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Created {
    id: Value,
    workspace_id: Value,
    project_id: Value,
    title: String,
}

fn int(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| value.as_str()?.parse().ok())
}

/// The title of a task made from `text`: its first line, cut at 200 characters.
pub fn title(text: &str) -> String {
    text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or(text).chars().take(200).collect()
}

fn refused(status: reqwest::StatusCode) -> Error {
    match status.as_u16() {
        400 | 422 => Error::invalid("Opus refused the task"),
        403 | 404 => Error::not_found("project not found in Opus"),
        402 => Error::payment_required("your Opus plan does not allow more tasks"),
        other => {
            tracing::warn!(status = other, "tasks: opus refused the task");
            Error::unavailable("Opus could not create the task")
        }
    }
}

/// Creates a task in Opus project `project` for `user_id`, titled after the first line of
/// `text`, with the whole text as its description.
pub async fn create(state: &AppState, user_id: i64, project: i64, text: &str) -> Result<Task> {
    let opus = &state.suite.apps.opus;
    let endpoint = format!("{}/api/task/{project}", opus.url);
    let body = json!({"title": title(text), "description": text});
    let build = |http: &reqwest::Client, token: &str| http.post(&endpoint).bearer_auth(token).json(&body);
    let response = state.suite.send(user_id, opus, true, build).await?;
    if !response.status().is_success() {
        return Err(refused(response.status()));
    }
    let created: Created = response.json().await.map_err(|_| Error::unavailable("Opus answered an unreadable task"))?;
    let ids = (int(&created.id), int(&created.workspace_id), int(&created.project_id));
    let (Some(id), Some(workspace), Some(project)) = ids else {
        return Err(Error::unavailable("Opus answered a task without ids"));
    };
    let url = format!("{}/dashboard/workspace/{workspace}/project/{project}/task/{id}", opus.url);
    Ok(Task { id, title: created.title, url })
}
