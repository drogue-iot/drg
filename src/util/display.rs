use crate::util::{show_json, show_json_string, DrogueError, JsonOutcome, Outcome, OutcomeStatus};
use crate::MultipleOutcomes;
use serde::Serialize;

pub fn display<T, F>(
    outcome: Result<Outcome<T>, DrogueError>,
    json: bool,
    f_data: F,
) -> anyhow::Result<i32>
where
    T: Serialize + Clone,
    F: FnOnce(&T),
{
    match (outcome, json) {
        (Ok(outcome), true) => match outcome {
            Outcome::SuccessWithMessage(msg) => {
                show_json(&serde_json::to_value(&JsonOutcome::success(msg))?)
            }
            Outcome::SuccessWithJsonData(data) => show_json(&serde_json::to_value(&data)?),
        },
        (Err(e), true) => {
            show_json_string(serde_json::to_string(&JsonOutcome::from(&e))?);
            return Ok(1);
        }
        (Ok(outcome), false) => match outcome {
            Outcome::SuccessWithMessage(msg) => println!("{msg}"),
            Outcome::SuccessWithJsonData(data) => f_data(&data),
        },
        (Err(e), false) => {
            println!("{}", e);
            return Ok(1);
        }
    }
    Ok(0)
}

/// fallback to showing the serialized object
pub fn display_simple<T: Serialize + Clone>(
    outcome: Result<Outcome<T>, DrogueError>,
    json: bool,
) -> anyhow::Result<i32> {
    display(outcome, json, |data: &T| {
        show_json_string(serde_json::to_string(data).unwrap())
    })
}

pub fn display_multiple<T: Serialize + Clone>(
    outcomes: MultipleOutcomes<T>,
    json: bool,
) -> anyhow::Result<i32> {
    if outcomes.operations.len() == 1 {
        // This is ugly but i can't derive clone for Result<Outome, DrogueError>
        let outcome = match outcomes.operations.get(0).unwrap() {
            Ok(o) => Ok(o.clone()),
            Err(e) => Err(e.clone()),
        };
        return display_simple(outcome, json);
    } else if json {
        let jsons: Vec<JsonOutcome> = outcomes
            .operations
            .iter()
            .map(|r| match r {
                Ok(outcome) => JsonOutcome::from(outcome),
                Err(e) => JsonOutcome::from(e),
            })
            .collect();

        let outcome = JsonOutcome {
            status: outcomes.status,
            message: outcomes.message,
            http_status: None,
            operations: Some(jsons),
        };

        show_json(&serde_json::to_value(&outcome)?);
        match outcomes.status {
            OutcomeStatus::Success => Ok(0),
            OutcomeStatus::Failure => Ok(1),
        }
    } else {
        println!("{}", outcomes.message);
        for result in outcomes.operations {
            match result {
                Ok(success) => match success {
                    Outcome::SuccessWithMessage(msg) => println!("{msg}"),
                    Outcome::SuccessWithJsonData(data) => unreachable!(),
                },
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
        match outcomes.status {
            OutcomeStatus::Success => Ok(0),
            OutcomeStatus::Failure => Ok(1),
        }
    }
}
