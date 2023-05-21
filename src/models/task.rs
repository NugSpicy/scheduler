/*
Task Model
  Struct{}
  Create
  Read
  ReadOne (Probably should be the same as Read)
  Delete
*/

use chrono::{DateTime, Utc};
use scylla::{
    transport::{errors::QueryError, query_result::FirstRowTypedError},
    FromRow, IntoTypedRows, ValueList,
};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use super::shims;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Invalid input to query: {0}")]
    QuerySucks(#[from] QueryError),
    #[error("No rows returned: {0}")]
    TypedError(#[from] FirstRowTypedError),
}

#[derive(Debug, Serialize, FromRow, ValueList)]
pub struct Task {
    pub id: Uuid,
    pub task_type: String,
    pub state: String,
    pub send_ts: shims::Timestamp,
    pub processor: Option<Uuid>,
}

static SELECT_TASK: &str = r#"
  SELECT id, task_type, state, send_at, processor FROM scheduler.tasks WHERE id = ?
"#;
pub async fn get_task(sess: scylla::Session, id: Uuid) -> Result<Task, TaskError> {
    let result = sess.query(SELECT_TASK, (id,)).await?;

    Ok(result.first_row_typed::<Task>()?)
}

static SELECT_TASKS: &str = r#"
  SELECT id, task_type, state, send_at, processor FROM scheduler.tasks
"#;
pub async fn get_tasks(
    sess: scylla::Session,
    _state: Option<String>,
    _since: Option<DateTime<Utc>>,
    _until: Option<DateTime<Utc>>,
) -> Result<Vec<Task>, QueryError> {
    let result = sess.query(SELECT_TASKS, ()).await?;

    let tasks = result
        .rows
        .unwrap_or_default()
        .into_typed::<Task>()
        .filter_map(|t| Some(t.ok()?))
        .collect();

    Ok(tasks)
}

static CREATE_TASK: &str = r#"
  INSERT INTO scheduler.tasks
    (id, task_type, send_at, state, processor)
  VALUES
    (?, ?, ?, ?, ?)
"#;
pub async fn create_task(sess: scylla::Session, task: Task) -> Result<Uuid, QueryError> {
    let _result = sess
        .query(
            CREATE_TASK,
            (
                task.id,
                task.task_type,
                task.send_ts,
                task.state,
                task.processor,
            ),
        )
        .await?;

    Ok(Uuid::new_v4())
}

static DELETE_TASK: &str = r#"
  DELETE FROM scheduler.tasks WHERE id = ?
"#;
pub async fn delete_task(sess: scylla::Session, id: Uuid) -> Result<(), QueryError> {
    sess.query(DELETE_TASK, (id,)).await?;

    Ok(())
}
