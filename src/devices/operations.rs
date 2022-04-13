use crate::config::Context;
use crate::util;
use anyhow::{anyhow, Context as AnyhowContext, Result};
use clap::Values;
use json_value_merge::Merge;

use serde_json::{json, Value};
use sha_crypt::sha512_simple;
use tabular::{Row, Table};

use crate::devices::Outcome::SuccessWithJsonData;
use crate::devices::{DeviceOperation, OperationType, Outcome};
use drogue_client::registry::v1::Password::Sha512;
use drogue_client::registry::v1::{Client, Credential, Device};

impl DeviceOperation {
    async fn delete(&self, config: &Context, ignore_missing: bool) -> Result<Outcome<()>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        match (
            client
                .delete_device(&self.app, &self.device.as_ref().unwrap())
                .await,
            ignore_missing,
        ) {
            (Ok(true), _) => Ok(Outcome::SuccessWithMessage(format!("Device deleted"))),
            (Ok(false), false) => Err(anyhow!("Application or device not found")),
            (Ok(false), true) => Ok(Outcome::Success),
            (Err(e), _) => Err(e.into()),
        }
    }

    async fn read(&self, config: &Context) -> Result<Outcome<Device>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        match client
            .get_device(&self.app, &self.device.as_ref().unwrap())
            .await
        {
            Ok(Some(dev)) => Ok(SuccessWithJsonData(dev)),
            Ok(None) => Err(anyhow!("Device or application not found")),
            Err(e) => Err(e.into()),
        }
    }

    async fn create(&self, config: &Context) -> Result<Outcome<()>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        Ok(client
            .create_device(&self.payload.as_ref().unwrap())
            .await
            .map(|_| Outcome::SuccessWithMessage(format!("Device created")))?)
    }

    async fn edit(&self, config: &Context) -> Result<Outcome<()>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let op = match (&self.device, &self.payload) {
            (None, Some(d)) => client.update_device(d).await,
            (Some(id), None) => {
                //read device data
                let data = client
                    .get_device(&self.app, &self.device.as_ref().unwrap())
                    .await?;
                match data {
                    Some(dev) => {
                        let edited = util::editor(dev)?;
                        client.update_app(&edited).await
                    }
                    None => Ok(false),
                }
            }
            // Clap is making sure the arguments are mutually exclusive.
            _ => unreachable!(),
        };

        match op {
            Ok(true) => Ok(Outcome::SuccessWithMessage(format!("Device updated"))),
            Ok(false) => Err(anyhow!(format!("Device or application not found"))),
            Err(e) => Err(e.into()),
        }
    }

    async fn list(
        &self,
        config: &Context,
        labels: Option<Values<'_>>,
    ) -> Result<Outcome<Vec<Device>>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let labels = util::clap_values_to_labels(labels);

        match client.list_devices(&self.app, labels).await {
            Ok(Some(devices)) => Ok(SuccessWithJsonData(devices)),
            Ok(None) => Err(anyhow!("Application not found")),
            Err(e) => Err(e.into()),
        }
    }

    async fn set_gateway(&self, config: &Context, gateway_id: String) -> Result<Outcome<()>> {
        // prepare json data to merge
        let data = json!({"spec": {
        "gatewaySelector": {
            "matchNames": [gateway_id]
        }
        }});

        self.merge_in(data, config).await
    }

    async fn set_password(
        &self,
        config: &Context,
        password: String,
        username: Option<&str>,
    ) -> Result<Outcome<()>> {
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

    async fn add_alias(&self, config: &Context, new_alias: String) -> Result<Outcome<()>> {
        // prepare json data to merge
        let data = json!({"spec": {
        "alias": [
              new_alias
            ]
        }});

        self.merge_in(data, config).await
    }

    async fn add_labels(&self, config: &Context, args: Values<'_>) -> Result<Outcome<()>> {
        let data = util::process_labels(&args);
        self.merge_in(data, config).await
    }

    /// todo merge that with the same method in apps ?
    /// merges a serde Value into the device object that exist on the server
    async fn merge_in(&self, data: Value, config: &Context) -> Result<Outcome<()>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        //retrieve device
        let op = match client
            .get_device(
                &self.app,
                self.device
                    .as_ref()
                    .ok_or(anyhow!("No device name provided"))?,
            )
            .await
        {
            Ok(Some(device)) => {
                serde_json::to_value(&device)?.merge(data);
                client.update_device(&device).await
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        };

        match op {
            Ok(true) => Ok(Outcome::Success),
            Ok(false) => Err(anyhow!("Device or application not found")),
            Err(e) => Err(e.into()),
        }
    }
}

// todo the firmware status section is not part of the core types. If we see a use case arise
// where there is a need for a generic schema extension mechanism that the CLI tool can handle,
// this part needs to be refactored.

fn pretty_list(data: Vec<Device>, wide: bool) {
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
        let name = dev.metadata.name;
        let creation = dev.metadata.creation_timestamp;

        let mut row = Row::new()
            .with_cell(name)
            .with_cell(util::age_from_timestamp(creation));

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

pub fn name_from_json_or_file(param: Option<String>, file: Option<&str>) -> Result<String> {
    match (param, file) {
        (Some(id), None) => Ok(id),
        (None, Some(file)) => {
            let f: Value = util::get_data_from_file(file)?;
            let id = f["metadata"]["name"]
                .as_str()
                .context("Misisng `name` property in device definition file")?
                .to_string();
            Ok(id)
        }
        // we must have id or file, not both, not neither.
        _ => unreachable!(),
    }
}
