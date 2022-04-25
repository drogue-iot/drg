use drogue_client::error::ClientError;
use serde::Serialize;
use serde_json::json;

/// When it comes to operation results there are a two possible outputs:
enum Outcome<T: Serialize> {
    Success,
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

impl Outcome<T>
    where T: Serialize,
{
    fn display(&self, json: bool, f: Fn(Option<T>)) {
        match (self, json) {
            (Ok(outcome), true) => {
                match outcome {
                    Outcome::SuccessWithJsonData(data) => show_json(data),
                    Outcome::Success => show_json(json!({"success": true})),
                    Outcome::SuccessWithMessage(msg) => show_json(json!({"status": "success", "message": msg})),
                }
            },
            (Ok(outcome), false) => {
                match outcome {
                    Outcome::SuccessWithJsonData(data) => f(data),
                    Outcome::Success => f(None),
                    Outcome::SuccessWithMessage(msg) => f(msg),
                }
            },
            (Err(error), true) => show_json(json!({"status": "error", "message": error.0})),
            (Err(error), false) => {
                println!("Error: {}", error.0)
            },
        }
    }
}

use thiserror::Error;
use crate::util::show_json;

#[derive(Error, Debug)]
pub enum DrogueError {
    #[error("The operation was not completed because `{0}`")]
    User(String),
    #[error("The application or device was not found")]
    NotFound,
    #[error("Error from drogue cloud {}")]
    Service(ClientError<reqwest::Error>),
}