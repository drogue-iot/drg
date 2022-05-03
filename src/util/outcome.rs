use anyhow::Result;
use drogue_client::error::ClientError;
use serde::{Deserialize, Serialize};

use crate::util::show_json;
use thiserror::Error;

/// When it comes to operation results there are a three possible outputs:
/// TODO : add errors as outcome and serialize them this way ?
pub enum Outcome<T: Serialize> {
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

impl<T> Outcome<T>
where
    T: Serialize,
{
    pub fn display<F>(self, json: bool, f_data: F) -> Result<()>
    where
        F: FnOnce(&T),
    {
        match (self, json) {
            (outcome, true) => match outcome {
                Outcome::SuccessWithMessage(msg) => {
                    show_json(serde_json::to_string(&JsonOutcome::success(msg))?)
                }
                Outcome::SuccessWithJsonData(data) => show_json(serde_json::to_string(&data)?),
            },
            (outcome, false) => match outcome {
                Outcome::SuccessWithMessage(msg) => println!("{msg}"),
                Outcome::SuccessWithJsonData(data) => f_data(&data),
            },
        }
        Ok(())
    }

    /// fallback to showing the serialized object
    pub fn display_simple(self, json: bool) -> Result<()> {
        self.display(json, |data| show_json(serde_json::to_string(data).unwrap()))
    }
}

// TODO : wrap errors into JSON
#[derive(Error, Debug)]
pub enum DrogueError {
    #[error("The operation was not completed because `{0}`")]
    User(String),
    #[error("The application or device was not found")]
    NotFound,
    #[error("Error from drogue cloud")]
    Service {
        #[from]
        source: ClientError<reqwest::Error>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonOutcome {
    status: OutcomeStatus,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum OutcomeStatus {
    Success,
    Failure,
}

impl JsonOutcome {
    pub fn success(message: String) -> JsonOutcome {
        JsonOutcome {
            status: OutcomeStatus::Success,
            message,
        }
    }

    pub fn failure(message: String) -> JsonOutcome {
        JsonOutcome {
            status: OutcomeStatus::Failure,
            message,
        }
    }
}
