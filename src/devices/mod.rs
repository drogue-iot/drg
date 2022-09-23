mod operations;

pub use operations::pretty_list;

use crate::util;
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
            (Some(f), None, None) => {
                let dev: Device = util::get_data_from_file(f)?;
                (dev, None)
            }
            (None, Some(data), Some(name)) => {
                let mut device = Device::new(application.clone(), name);
                if let Some(spec) = data.as_object() {
                    device.spec = spec.clone();
                }
                (device, None)
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

    pub fn from_device(dev: Device) -> Self {
        DeviceOperation {
            device: None,
            app: dev.metadata.application.clone(),
            payload: dev,
        }
    }
}
