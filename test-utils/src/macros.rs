#[macro_export]

/// Sometimes, drogue-cloud can't keep up with the tests.
/// This retries the specified number of times the call in case the reponse is a 409.
macro_rules! retry_409 {
    ($number_of_retries:expr, $command:expr) => {{

        use drg_test_utils::{JsonOutcome, OutcomeStatus};

        let mut count = 0u32;
        let res = loop {
            count += 1;
            let command_res = $command.assert();

            let res: JsonOutcome = serde_json::from_slice(&command_res.get_output().stdout).unwrap();
            if res.status == OutcomeStatus::Failure && res.http_status != Some(409) {
                panic!("{}", format!("The operation failed with HTTP {}", res.http_status.unwrap()));
            } else if res.status == OutcomeStatus::Success {
                command_res.success();
                break res
            }

            if count == $number_of_retries {
                panic!("Reached the max number of attempts for conflicts retries");
            }
        };

    }};
}