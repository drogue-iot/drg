use anyhow::anyhow;
use drogue_client::error::ClientError;
use serde_json::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DrogueError {
    #[error("The operation was not completed because `{0}`")]
    InvalidInput(String),
    #[error("The application or device was not found")]
    NotFound,
    #[error("Error from drogue cloud: ")]
    Service(String, u16),
    #[error("Unexpected error from the client library: {0}")]
    UnexpectedClient(#[from] anyhow::Error),
    #[error("There is an issue in drg configuration: {0}")]
    ConfigIssue(String),
}

impl From<ClientError> for DrogueError {
    fn from(error: ClientError) -> Self {
        match error {
            ClientError::Client(e) => DrogueError::UnexpectedClient(anyhow!(e)),
            ClientError::Request(msg) => DrogueError::UnexpectedClient(anyhow!("{}", msg)),
            ClientError::Service(e) => DrogueError::Service(e.message, e.status.as_u16()),
            ClientError::Token(e) => DrogueError::UnexpectedClient(anyhow!(e)),
            ClientError::Url(e) => DrogueError::ConfigIssue(format!("Invalid url: {}", e)),
            ClientError::Syntax(e) => {
                DrogueError::UnexpectedClient(anyhow!("JSON parsing error: {}", e))
            }
        }
    }
}

impl From<serde_json::Error> for DrogueError {
    fn from(e: Error) -> Self {
        DrogueError::InvalidInput(format!("Deserialization error: {}", e))
    }
}

impl DrogueError {
    pub fn status(&self) -> Option<u16> {
        if let DrogueError::Service(_, status) = self {
            Some(*status)
        } else {
            None
        }
    }
}
