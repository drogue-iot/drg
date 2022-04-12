mod operations;

use crate::util;
use anyhow::Result;
use drogue_client::registry::v1::{Client, Device};
use serde_json::Value;

/// DeviceOperation
struct DeviceOperation {
    app: String,
    device: Option<String>,
    payload: Option<Device>,

    json_output: bool,

    res: Option<Result<Outcome>>,
}

// todo : move this to a separate mod ?
/// When it comes to operation results there are a two possible outputs:
enum Outcome<T> {
    Success,
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

/// custom errors ?
///  ServiceError,   ---> error we find out after sending request
///  UserError,      ---> error we catch before sending the request

impl DeviceOperation {
    pub fn new(
        application: String,
        device_name: Option<String>,
        file: Option<&str>,
        data: Option<Value>,
        json: bool,
    ) -> Self {

        let device: Device = match file {
            Some(f) => util::get_data_from_file(f)?,
            None => {
                let mut device = Device::new(application.clone(), device_name.clone());
                if let Some(spec) = data.as_object() {
                    device.spec = spec.clone();
                }
                device
            }
        };

        DeviceOperation {
            json_output: json,
            device: device_name,
            app: application,
            payload: Some(device),
            res: None,
        }
    }
}
