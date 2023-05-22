use anyhow::Result;
use scylla::{
    transport::{
        errors::{NewSessionError, QueryError},
        session_builder::{DefaultMode, GenericSessionBuilder},
    },
    Session, SessionBuilder,
};
use std::fs;
use thiserror::Error;

const SCYLLA_URI: &str = "scylla:9042";

#[derive(Debug, Error)]
pub enum ScyllaError {
    #[error("Failed to create new session: {0}")]
    SessionInitFailed(#[from] NewSessionError),
    #[error("Schema update failed while running query \"{1}\" with error: {0}")]
    SchemaFailedError(#[source] QueryError, String),
}

#[derive(Clone)]
pub struct ScyllaConnection {
    builder: GenericSessionBuilder<DefaultMode>,
}

impl ScyllaConnection {
    pub fn new() -> Self {
        Self {
            builder: SessionBuilder::new()
                .known_node(SCYLLA_URI)
                .user("cassandra", "cassandra"), // meta
        }
    }

    pub async fn update_schema(&self, schema_path: &str) -> Result<(), ScyllaError> {
        let schema = fs::read_to_string(schema_path).expect("Should be able to read schema.cql");
        let session = self.session().await?;

        let queries = schema
            .split_inclusive(";")
            .map(|e| e.trim())
            .filter(|e| !e.is_empty());

        for query in queries {
            if let Err(e) = session.query(query, ()).await {
                return Err(ScyllaError::SchemaFailedError(e, query.to_string()));
            }
        }

        Ok(())
    }

    pub async fn session(&self) -> Result<Session, NewSessionError> {
        let session = self.builder.build().await?;

        Ok(session)
    }
}
