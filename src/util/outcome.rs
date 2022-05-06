use crate::util::error::DrogueError;
use serde::{Deserialize, Serialize};

pub enum Outcome<T: Serialize> {
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonOutcome {
    status: OutcomeStatus,
    message: String,
    // The HTTP status code
    http_status: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
enum OutcomeStatus {
    Success,
    Failure,
}

impl From<&DrogueError> for JsonOutcome {
    fn from(error: &DrogueError) -> Self {
        JsonOutcome {
            status: OutcomeStatus::Failure,
            message: error.to_string(),
            http_status: error.status(),
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
}
