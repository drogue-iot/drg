use crate::util::error::DrogueError;
use serde::{Deserialize, Serialize};

pub enum Outcome<T: Serialize> {
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

impl<T: Serialize> Outcome<T> {
    pub fn inner(self) -> Result<T, DrogueError> {
        match self {
            Outcome::SuccessWithJsonData(t) => Ok(t),
            // todo add a drogueError variant for this type of stuff ?
            Outcome::SuccessWithMessage(msg) => Err(DrogueError::InvalidInput(msg)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonOutcome {
    status: OutcomeStatus,
    message: String,
    // The HTTP status code
    #[serde(skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum OutcomeStatus {
    Success,
    Failure,
}

impl From<&DrogueError> for JsonOutcome {
    fn from(error: &DrogueError) -> Self {
        match error {
            DrogueError::NotFound => JsonOutcome {
                status: OutcomeStatus::Failure,
                message: error.to_string(),
                http_status: Some(404),
            },
            DrogueError::Service(e, status) => JsonOutcome {
                status: OutcomeStatus::Failure,
                message: e.clone(),
                http_status: Some(*status),
            },
            DrogueError::InvalidInput(e) => JsonOutcome::failure(e.clone()),
            DrogueError::UnexpectedClient(e) => {
                JsonOutcome::failure(format!("Unexpected error: {}", e))
            }
            DrogueError::ConfigIssue(e) => JsonOutcome::failure(e.clone()),
        }
    }
}

impl JsonOutcome {
    pub fn success(message: String) -> JsonOutcome {
        JsonOutcome {
            status: OutcomeStatus::Success,
            message,
            http_status: None,
        }
    }

    pub fn failure(message: String) -> JsonOutcome {
        JsonOutcome {
            status: OutcomeStatus::Failure,
            message,
            http_status: None,
        }
    }
}
