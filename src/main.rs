/*
Main
    Define Error Struct
    Define State Struct
    Create HTTP Server
        Router Endpoints
        Router Functions
*/

use std::net::SocketAddr;

use adapters::scylla::ScyllaConnection;
use anyhow::{Error, Result};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use models::Task;

mod adapters;
mod models;
mod worker;

pub struct RouterError(Error);

impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("An unexpected error occurred: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for RouterError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Clone)]
struct RequestState {
    db: ScyllaConnection,
}

impl RequestState {
    async fn new() -> Result<RequestState> {
        let conn = ScyllaConnection::new();
        conn.update_schema("src/procedures/schema.cql").await?;

        Ok(Self { db: conn })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create State Object and wait for any internal connections
    let state = RequestState::new().await?;

    // Create the schedule Supervisor
    let _sup = worker::Supervisor::new((&state).db.clone());

    // Create Router w/ Endpoints defined
    let router = Router::new()
        .route("/task/:id", routing::get(get_task).delete(delete_task))
        .route("/task", routing::get(get_tasks).post(create_task))
        .with_state(state);

    // Define the address and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn get_task(
    State(state): State<RequestState>,
    Path(id): Path<String>,
) -> Result<String, RouterError> {
    let uid = Uuid::try_parse(&id)?;
    let sess = state.db.session().await?;

    let task = models::get_task(sess, uid).await?;

    Ok(serde_json::to_string(&task)?)
}

#[derive(Debug, Deserialize, Default)]
pub struct TaskFilters {
    pub state: Option<String>,
    pub task_type: Option<String>,
}

async fn get_tasks(
    State(state): State<RequestState>,
    opts: Option<Query<TaskFilters>>,
) -> Result<String, RouterError> {
    let Query(opts) = opts.unwrap_or_default();
    let sess = state.db.session().await?;

    let tasks = models::get_tasks(sess, opts.state, opts.task_type).await?;

    Ok(serde_json::to_string(&tasks)?)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateTask {
    task_type: String,
    send_ts: DateTime<Utc>,
}

async fn create_task(
    State(state): State<RequestState>,
    Json(payload): Json<CreateTask>,
) -> Result<String, RouterError> {
    let task = Task {
        id: Uuid::new_v4(),
        task_type: payload.task_type,
        send_ts: Duration::seconds(payload.send_ts.timestamp()).into(),
        state: "READY".to_string(),
        processor: Uuid::nil(),
    };
    let sess = state.db.session().await?;

    let id = models::create_task(sess, task).await?;

    Ok(id.to_string())
}

async fn delete_task(
    State(state): State<RequestState>,
    Path(id): Path<String>,
) -> Result<(), RouterError> {
    let uid = Uuid::try_parse(&id)?;
    let sess = state.db.session().await?;

    Ok(models::delete_task(sess, uid).await?)
}
