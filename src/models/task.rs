/*
Task Model
  Struct{}
  Create
  Read
  ReadOne (Probably should be the same as Read)
  Delete
*/

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
    pub processor: Uuid,
}

static SELECT_TASK: &str = r#"
  SELECT id, task_type, state, send_ts, processor FROM scheduler.tasks WHERE id = ?
"#;
pub async fn get_task(sess: scylla::Session, id: Uuid) -> Result<Task, TaskError> {
    let result = sess.query(SELECT_TASK, (id,)).await?;

    Ok(result.first_row_typed::<Task>()?)
}

// TODO: Filtering
static SELECT_TASKS: &str = r#"
  SELECT id, task_type, state, send_ts, processor FROM scheduler.tasks
"#;
pub async fn get_tasks(
    sess: scylla::Session,
    _state: Option<String>,
    _type: Option<String>,
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
    (partition, id, task_type, send_ts, state, processor)
  VALUES
    (?, ?, ?, ?, ?, ?)
"#;
pub async fn create_task(sess: scylla::Session, task: Task) -> Result<Uuid, QueryError> {
    let _result = sess
        .query(
            CREATE_TASK,
            (
                0,
                task.id,
                task.task_type,
                task.send_ts,
                task.state,
                task.processor,
            ),
        )
        .await?;

    Ok(task.id)
}

static DELETE_TASK: &str = r#"
  DELETE FROM scheduler.tasks WHERE partition = 0 AND id = ?
"#;
pub async fn delete_task(sess: scylla::Session, id: Uuid) -> Result<(), QueryError> {
    sess.query(DELETE_TASK, (id,)).await?;

    Ok(())
}

static PROCESS_PREP: &str = r#"
  SELECT id FROM scheduler.tasks
  WHERE processor = 00000000-0000-0000-0000-000000000000 AND state = 'READY' AND send_ts <= toTimeStamp(now())
  LIMIT ? ALLOW FILTERING
"#;
static PROCESS_MUT: &str = r#"
  UPDATE scheduler.tasks
  SET processor = ?, state = 'PROCESSING'
  WHERE partition = 0 AND id IN ?
"#;
static PROCESS_GET: &str = r#"
  SELECT id, task_type, state, send_ts, processor FROM scheduler.tasks
  WHERE partition = 0 AND id IN ? AND processor = ? ALLOW FILTERING
"#;
pub async fn process_tasks(
    sess: scylla::Session,
    processor: &Uuid,
    size: i32,
) -> Result<Vec<Task>, QueryError> {
    let found_tasks = sess.query(PROCESS_PREP, (size,)).await?;
    let tasks: Vec<Uuid> = found_tasks
        .rows()
        .unwrap_or_default()
        .into_typed::<(Uuid,)>()
        .filter_map(|t| Some(t.ok()?.0))
        .collect();
    if tasks.len() == 0 {
        return Ok(Vec::new());
    }
    sess.query(PROCESS_MUT, (processor, &tasks)).await?;
    let result = sess.query(PROCESS_GET, (&tasks, processor)).await?;

    let tasks = result
        .rows
        .unwrap_or_default()
        .into_typed::<Task>()
        .filter_map(|t| Some(t.ok()?))
        .collect();

    Ok(tasks)
}

static COMPLETE_TASK: &str = r#"
    UPDATE scheduler.tasks
    SET state = 'COMPLETE'
    WHERE partition = 0 AND id = ?
"#;
pub async fn complete_task(sess: scylla::Session, task: Task) -> Result<(), QueryError> {
    sess.query(COMPLETE_TASK, (task.id,)).await?;

    Ok(())
}
