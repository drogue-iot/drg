mod operations;

pub use operations::pretty_list;

use crate::util::{self, DrogueError};
use anyhow::Result;
use drogue_client::registry::v1::Device;
use serde_json::Value;

#[derive(Debug)]
pub struct DeviceOperation {
    app: String,
    device: Option<String>,
    payload: Device,
}

impl DeviceOperation {
    pub fn new(
        application: String,
        device_name: Option<String>,
        file: Option<&str>,
        data: Option<Value>,
    ) -> Result<Self> {
        let (device, name) = match (file, data, device_name) {
            (Some(f), None, None) => util::get_data_from_file(f)?,
            (None, Some(data), Some(name)) => {
                let mut device = Device::new(application.clone(), name.clone());
                if let Some(spec) = data.as_object() {
                    device.spec = spec.clone();
                }
                (device, Some(name))
            }
            (None, None, Some(name)) => {
                (Device::new(application.clone(), name.clone()), Some(name))
            }
            (None, None, None) => (Device::new(application.clone(), "none"), None),
            _ => unreachable!(),
        };

        Ok(DeviceOperation {
            device: name,
            app: application,
            payload: device,
        })
    }

    fn device_id(&self) -> Result<&String> {
        self.device
            .as_ref()
            .ok_or_else(|| DrogueError::User("No device name provided".to_string()).into())
    }
}
