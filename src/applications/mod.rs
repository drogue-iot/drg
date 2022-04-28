mod operations;

use crate::util;
use anyhow::Result;
use drogue_client::registry::v1::Application;
use serde_json::Value;

pub use operations::pretty_list;

#[derive(Debug, Default)]
pub struct ApplicationOperation {
    name: Option<String>,
    payload: Application,
}

impl ApplicationOperation {
    pub fn new(name: Option<String>, file: Option<&str>, data: Option<Value>) -> Result<Self> {
        let (app, name) = match (file, data, name) {
            (Some(f), None, None) => util::get_data_from_file(f)?,
            (None, Some(data), Some(name)) => {
                let mut app = Application::new(name.clone());
                if let Some(spec) = data.as_object() {
                    app.spec = spec.clone();
                }
                (app, Some(name))
            }
            (None, None, Some(name)) => (Application::new(name.clone()), Some(name)),
            (None, None, None) => (Application::new("empty"), None),
            _ => unreachable!(),
        };

        Ok(ApplicationOperation { name, payload: app })
    }
}
