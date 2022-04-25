mod operations;
mod outcome;

pub use outcome::*;

use crate::util;
use anyhow::{anyhow, Result};
use drogue_client::registry::v1::{Client, Device};
use serde::Serialize;
use serde_json::{json, Value};
use crate::util::show_json;

/// DeviceOperation
struct DeviceOperation {
    app: String,
    device: Option<String>,
    payload: Option<Device>,

    json_output: bool,
}

impl DeviceOperation {
   pub fn new(
       application: String,
       device_name: Option<String>,
       file: Option<&str>,
       data: Option<Value>,
       json: bool,
   ) -> Self {

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
            device: device_name,
            app: application,
            payload: Some(device),
        }
    }
}