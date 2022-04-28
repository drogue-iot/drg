use crate::config::Context;
use crate::util;
use anyhow::{anyhow, Result};
use clap::Values;
use json_value_merge::Merge;

use serde_json::{json, Value};
use sha_crypt::sha512_simple;
use tabular::{Row, Table};

use crate::devices::DeviceOperation;
use crate::outcome::{DrogueError, Outcome};
use drogue_client::registry::v1::Password::Sha512;
use drogue_client::registry::v1::{Client, Credential, Device};

impl DeviceOperation {
    pub async fn delete(&self, config: &Context, ignore_missing: bool) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        match (
            client.delete_device(&self.app, self.device_id()?).await,
            ignore_missing,
        ) {
            (Ok(true), _) => Ok(Outcome::SuccessWithMessage("Device deleted".to_string())),
            (Ok(false), false) => Err(DrogueError::NotFound.into()),
            (Ok(false), true) => Ok(Outcome::SuccessWithMessage(
                "No device to delete, ignoring.".to_string(),
            )),
            (Err(e), _) => Err(e.into()),
        }
    }

    pub async fn read(&self, config: &Context) -> Result<Outcome<Device>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        match client.get_device(&self.app, self.device_id()?).await {
            Ok(Some(dev)) => Ok(Outcome::SuccessWithJsonData(dev)),
            Ok(None) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn create(&self, config: &Context) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        Ok(client
            .create_device(&self.payload)
            .await
            .map(|_| Outcome::SuccessWithMessage("Device created".to_string()))?)
        // .map_err(DrogueError::Service(e))?)
    }

    pub async fn edit(&self, config: &Context) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let op = match &self.device {
            None => client.update_device(&self.payload).await,
            Some(name) => {
                //read device data
                let data = client.get_device(&self.app, name).await?;
                match data {
                    Some(dev) => {
                        let edited = util::editor(dev)?;
                        client.update_device(&edited).await
                    }
                    None => Ok(false),
                }
            }
        };

        match op {
            Ok(true) => Ok(Outcome::SuccessWithMessage("Device updated".to_string())),
            Ok(false) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn list(
        &self,
        config: &Context,
        labels: Option<Values<'_>>,
    ) -> Result<Outcome<Vec<Device>>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let labels = util::clap_values_to_labels(labels);

        match client.list_devices(&self.app, labels).await {
            Ok(Some(devices)) => Ok(Outcome::SuccessWithJsonData(devices)),
            Ok(None) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn set_gateway(
        &self,
        config: &Context,
        gateway_id: String,
    ) -> Result<Outcome<String>> {
        // prepare json data to merge
        let data = json!({"spec": {
        "gatewaySelector": {
            "matchNames": [gateway_id]
        }
        }});

        self.merge_in(data, config).await
    }

    pub async fn set_password(
        &self,
        config: &Context,
        password: String,
        username: Option<&str>,
    ) -> Result<Outcome<String>> {
        let hash = sha512_simple(&password, &Default::default())
            .map_err(|err| anyhow!("Failed to hash password: {:?}", err))?;

        let credential = match username {
            Some(user) => Credential::UsernamePassword {
                username: user.to_string(),
                password: Sha512(hash),
                unique: false,
            },
            None => Credential::Password { 0: Sha512(hash) },
        };

        // prepare json data to merge
        let data = json!({"spec": {
        "credentials": {
            "credentials": [
              credential
            ]
        }
        }});

        self.merge_in(data, config).await
    }

    pub async fn add_alias(&self, config: &Context, new_alias: String) -> Result<Outcome<String>> {
        // prepare json data to merge
        let data = json!({"spec": {
        "alias": [
              new_alias
            ]
        }});

        self.merge_in(data, config).await
    }

    pub async fn add_labels(&self, config: &Context, args: &Values<'_>) -> Result<Outcome<String>> {
        let data = util::process_labels(&args);
        self.merge_in(data, config).await
    }

    /// todo merge that with the same method in apps ?
    /// merges a serde Value into the device object that exist on the server
    async fn merge_in(&self, data: Value, config: &Context) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        //retrieve device
        let op = match client.get_device(&self.app, self.device_id()?).await {
            Ok(Some(device)) => {
                serde_json::to_value(&device)?.merge(data);
                client.update_device(&device).await
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        };

        match op {
            Ok(true) => Ok(Outcome::SuccessWithMessage("Device updated.".to_string())),
            Ok(false) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }
}

// todo the firmware status section is not part of the core types. If we see a use case arise
// where there is a need for a generic schema extension mechanism that the CLI tool can handle,
// this part needs to be refactored.

pub fn pretty_list(data: &[Device], wide: bool) {
    let mut header = Row::new().with_cell("NAME").with_cell("AGE");
    let mut table = if wide {
        header.add_cell("FIRMWARE");
        header.add_cell("CURRENT");
        header.add_cell("TARGET");
        Table::new("{:<} {:<} {:<} {:<} {:<}")
    } else {
        Table::new("{:<} {:<}")
    };

    table.add_row(header);

    for dev in data {
        let name = dev.metadata.name.clone();
        let creation = dev.metadata.creation_timestamp;

        let mut row = Row::new()
            .with_cell(name)
            .with_cell(util::age_from_timestamp(&creation));

        if wide {
            if let Some(firmware) = dev.status.get("firmware") {
                let current = firmware["current"].as_str();
                let target = firmware["target"].as_str();

                let mut in_sync = None;
                let mut update = None;
                for item in firmware["conditions"].as_array().unwrap() {
                    if let Some("InSync") = item["type"].as_str() {
                        in_sync.replace(item["status"].as_str().unwrap() == "True");
                    }

                    if let Some("UpdateProgress") = item["type"].as_str() {
                        update = item["message"].as_str();
                    }
                }

                match (in_sync, update) {
                    (Some(true), _) => row.add_cell("InSync"),
                    (Some(false), Some(update)) => row.add_cell(format!("Updating ({})", update)),
                    (Some(false), _) => row.add_cell("NotInSync"),
                    _ => row.add_cell("Unknown"),
                };

                if let Some(current) = current {
                    row.add_cell(current);
                }

                if let Some(target) = target {
                    row.add_cell(target);
                }
            } else {
                row.add_cell("");
                row.add_cell("");
                row.add_cell("");
            }
        }

        table.add_row(row);
    }

    print!("{}", table);
}
