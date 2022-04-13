mod operations;

use crate::util;
use anyhow::Result;
use drogue_client::registry::v1::{Client, Device};
use serde::Serialize;
use serde_json::Value;

/// DeviceOperation
struct DeviceOperation {
    app: String,
    device: Option<String>,
    payload: Option<Device>,

    json_output: bool,

    op: OperationType,
}

// todo : move this to a separate mod ?
/// When it comes to operation results there are a two possible outputs:
enum Outcome<T: Serialize> {
    Success,
    SuccessWithMessage(String),
    SuccessWithJsonData(T),
}

enum OperationType {
    Read,
    List,
    Create,
    Delete,
    Update,
}

/// custom errors ?
///  ServiceError,   ---> error we find out after sending request
///  UserError,      ---> error we catch before sending the request

impl DeviceOperation {
    pub fn read(application: String, device_name: String, json: bool) -> Result<Self> {
        Ok(DeviceOperation {
            json_output: json,
            device: Some(device_name),
            app: application,
            payload: None,
            op: OperationType::Read,
        })
    }

    pub fn creation(
        application: String,
        device_name: Option<String>,
        file: Option<&str>,
        data: Option<Value>,
        json: bool,
    ) -> Result<Self> {
        let device: Option<Device> = match (file, data) {
            (Some(f), None) => Some(util::get_data_from_file(f)?),
            (None, Some(data)) => {
                let mut device = Device::new(application.clone(), device_name.clone());
                if let Some(spec) = data.as_object() {
                    device.spec = spec.clone();
                }
                Some(device)
            }
            (None, None) => Some(Device::new(application.clone(), device_name.clone())),
            _ => unreachable!(),
        };

        Ok(DeviceOperation {
            json_output: json,
            device: device_name,
            app: application,
            payload: device,
            op: OperationType::Create,
        })
    }
}
