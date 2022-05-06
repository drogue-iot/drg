use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::util::error::DrogueError;
use crate::util::show_json;

pub enum Outcome<T: Serialize> {
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

pub trait Display<T>
where
    T: Serialize,
{
    fn get_self(self) -> Result<Outcome<T>, DrogueError>;

    fn display<F>(&self, json: bool, f_data: F) -> Result<()>
    where
        F: FnOnce(&T),
    {
        match (&self.get_self(), json) {
            (Ok(outcome), true) => match outcome {
                Outcome::SuccessWithMessage(msg) => {
                    show_json(serde_json::to_string(&JsonOutcome::success(msg.clone()))?)
                }
                Outcome::SuccessWithJsonData(data) => show_json(serde_json::to_string(&data)?),
            },
            (Err(e), true) => show_json(serde_json::to_string(&JsonOutcome::from(e))?),
            (Ok(outcome), false) => match outcome {
                Outcome::SuccessWithMessage(msg) => println!("{msg}"),
                Outcome::SuccessWithJsonData(data) => f_data(&data),
            },
            (Err(e), false) => println!("{}", e),
        }
        Ok(())
    }

    /// fallback to showing the serialized object
    fn display_simple(&self, json: bool) -> Result<()> {
        self.display(json, |data: &T| {
            show_json(serde_json::to_string(data).unwrap())
        })
    }
}

impl<T> Display<T> for Result<Outcome<T>, DrogueError>
where
    T: Serialize,
{
    fn get_self(self) -> Result<Outcome<T>, DrogueError> {
        self
    }
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
            message: error.to_string().clone(),
            http_status: error.status().clone(),
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
