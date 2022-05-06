use crate::util::{show_json, DrogueError, JsonOutcome, Outcome};
use serde::Serialize;

pub fn display<T, F>(
    outcome: Result<Outcome<T>, DrogueError>,
    json: bool,
    f_data: F,
) -> anyhow::Result<()>
where
    T: Serialize,
    F: FnOnce(&T),
{
    match (outcome, json) {
        (Ok(outcome), true) => match outcome {
            Outcome::SuccessWithMessage(msg) => {
                show_json(serde_json::to_string(&JsonOutcome::success(msg))?)
            }
            Outcome::SuccessWithJsonData(data) => show_json(serde_json::to_string(&data)?),
        },
        (Err(e), true) => show_json(serde_json::to_string(&JsonOutcome::from(&e))?),
        (Ok(outcome), false) => match outcome {
            Outcome::SuccessWithMessage(msg) => println!("{msg}"),
            Outcome::SuccessWithJsonData(data) => f_data(&data),
        },
        (Err(e), false) => println!("{}", e),
    }
    Ok(())
}

/// fallback to showing the serialized object
pub fn display_simple<T: Serialize>(
    outcome: Result<Outcome<T>, DrogueError>,
    json: bool,
) -> anyhow::Result<()> {
    display(outcome, json, |data: &T| {
        show_json(serde_json::to_string(data).unwrap())
    })
}
