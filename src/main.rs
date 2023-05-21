/*
Main
    Define Error Struct
    Define State Struct
    Create HTTP Server
        Router Endpoints
        Router Functions
*/

use std::net::SocketAddr;

use anyhow::{Error, Result};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

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

#[derive(Clone)]
struct RequestState {}

impl RequestState {
    async fn new() -> Result<RequestState> {
        Ok(Self {})
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create State Object and wait for any internal connections
    let state = RequestState::new().await?;

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
    unimplemented!()
}

#[derive(Debug, Deserialize, Default)]
pub struct TaskFilters {
    pub state: Option<String>,
    pub since_ts: Option<DateTime<Utc>>,
    pub until_ts: Option<DateTime<Utc>>,
}
async fn get_tasks(
    State(state): State<RequestState>,
    query_opts: Option<Query<TaskFilters>>,
) -> Result<String, RouterError> {
    unimplemented!()
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateTask {
    task: String,
    send_ts: DateTime<Utc>,
}
async fn create_task(
    State(state): State<RequestState>,
    Json(payload): Json<CreateTask>,
) -> Result<String, RouterError> {
    unimplemented!()
}

async fn delete_task(
    State(state): State<RequestState>,
    Path(id): Path<String>,
) -> Result<String, RouterError> {
    unimplemented!()
}
