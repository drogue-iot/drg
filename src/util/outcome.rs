use crate::util::error::DrogueError;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub enum Outcome<T: Serialize + Clone> {
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

pub struct MultipleOutcomes<T: Serialize + Clone> {
    pub status: OutcomeStatus,
    pub message: String,
    pub operations: Vec<Result<Outcome<T>, DrogueError>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonOutcome {
    pub status: OutcomeStatus,
    pub message: String,
    // The HTTP status code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<Vec<JsonOutcome>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OutcomeStatus {
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
                operations: None,
            },
            DrogueError::Service(e, status) => JsonOutcome {
                status: OutcomeStatus::Failure,
                message: e.clone(),
                http_status: Some(*status),
                operations: None,
            },
            DrogueError::InvalidInput(e) => JsonOutcome::failure(e.clone()),
            DrogueError::UnexpectedClient(e) => {
                JsonOutcome::failure(format!("Unexpected error: {}", e))
            }
            DrogueError::ConfigIssue(e) => JsonOutcome::failure(e.clone()),
        }
    }
}

impl<T: Serialize + Clone> From<&Outcome<T>> for JsonOutcome {
    fn from(outcome: &Outcome<T>) -> Self {
        match outcome {
            Outcome::SuccessWithMessage(msg) => JsonOutcome::success(msg.clone()),
            Outcome::SuccessWithJsonData(_) => unreachable!(),
        }
    }
}

impl JsonOutcome {
    pub fn success(message: String) -> JsonOutcome {
        JsonOutcome {
            status: OutcomeStatus::Success,
            message,
            http_status: None,
            operations: None,
        }
    }

    pub fn failure(message: String) -> JsonOutcome {
        JsonOutcome {
            status: OutcomeStatus::Failure,
            message,
            http_status: None,
            operations: None,
        }
    }
}

impl<T: Serialize + Clone> From<Vec<Result<Outcome<T>, DrogueError>>> for MultipleOutcomes<T> {
    fn from(results: Vec<Result<Outcome<T>, DrogueError>>) -> Self {
        let mut status = OutcomeStatus::Success;

        let _ = results.iter().map(|r| {
            if r.is_err() {
                status = OutcomeStatus::Failure
            }
        });

        MultipleOutcomes {
            status,
            message: String::new(),
            operations: results,
        }
    }
}

impl<T: Serialize + Clone> MultipleOutcomes<T> {
    pub fn message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }
}
