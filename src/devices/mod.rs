mod operations;
mod outcome;

pub use operations::pretty_list;
pub use outcome::*;

use crate::util;
use anyhow::Result;
use drogue_client::registry::v1::Device;
use serde_json::Value;

#[derive(Debug)]
pub struct DeviceOperation {
    app: String,
    device: Option<String>,
    payload: Option<Device>,
}

impl DeviceOperation {
    pub fn new(
        application: String,
        device_name: Option<String>,
        file: Option<&str>,
        data: Option<Value>,
    ) -> Result<Self> {
        let device: Option<Device> = match (file, data, device_name) {
            (Some(f), None, None) => Some(util::get_data_from_file(f)?),
            (None, Some(data), Some(name)) => {
                let mut device = Device::new(application.clone(), name);
                if let Some(spec) = data.as_object() {
                    device.spec = spec.clone();
                }
                Some(device)
            }
            (None, None, Some(name)) => Some(Device::new(application.clone(), name)),
            (None, None, None) => None,
            _ => unreachable!(),
        };
        let name = device.as_ref().map(|d| d.metadata.name.clone());

        Ok(DeviceOperation {
            device: name,
            app: application,
            payload: device,
        })
    }
}
