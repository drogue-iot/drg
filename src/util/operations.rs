macro_rules! handle_operation {
    ($res:expr, $msg:expr) => {{
        match $res {
            Ok(true) => Ok(Outcome::SuccessWithMessage($msg.to_string())),
            Ok(false) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }};
    ($res:expr) => {{
        match $res {
            Ok(Some(data)) => Ok(Outcome::SuccessWithJsonData(data)),
            Ok(None) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }};
}
pub(crate) use handle_operation;
