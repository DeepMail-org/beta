use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("neo4j connection error: {0}")]
    Neo4jConnection(String),

    #[error("neo4j query error: {0}")]
    Neo4jQuery(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("payload parse error: {0}")]
    PayloadParse(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("invalid label: {0}")]
    InvalidLabel(String),

    #[error("invalid relation type: {0}")]
    InvalidRelType(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl GraphError {
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            GraphError::Db(_) | GraphError::Neo4jConnection(_) | GraphError::Nats(_)
        )
    }
}

impl From<GraphError> for tonic::Status {
    fn from(e: GraphError) -> Self {
        match &e {
            GraphError::NotFound(_) => tonic::Status::not_found(e.to_string()),
            GraphError::InvalidArgument(_) => tonic::Status::invalid_argument(e.to_string()),
            GraphError::InvalidLabel(_) => tonic::Status::invalid_argument(e.to_string()),
            GraphError::InvalidRelType(_) => tonic::Status::invalid_argument(e.to_string()),
            GraphError::PayloadParse(_) => tonic::Status::invalid_argument(e.to_string()),
            _ => tonic::Status::internal(e.to_string()),
        }
    }
}
