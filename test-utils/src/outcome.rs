use serde::{Deserialize, Serialize};

//FIXME: move more of drg util there ?

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonOutcome {
    pub status: OutcomeStatus,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
