use serde::{Deserialize, Serialize};

//FIXME: import drg util there

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonOutcome {
    pub status: OutcomeStatus,
    pub message: String,
    // The HTTP status code
    // todo skip ser and deser if none
    pub http_status: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutcomeStatus {
    Success,
    Failure,
}

impl JsonOutcome {
    pub fn is_success(&self) -> bool {
        self.status == OutcomeStatus::Success
    }

    pub fn is_failure(&self) -> bool {
        self.status == OutcomeStatus::Failure
    }
}
