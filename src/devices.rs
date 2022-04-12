use crate::config::Context;
use crate::{util, AppId, DeviceId};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use clap::Values;
use json_value_merge::Merge;

use serde_json::{json, Value};
use sha_crypt::sha512_simple;
use tabular::{Row, Table};

use drogue_client::registry::v1::Password::Sha512;
use drogue_client::registry::v1::{Client, Credential, Device};

pub async fn delete(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    ignore_missing: bool,
) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match (client.delete_device(app, &device_id).await, ignore_missing) {
        (Ok(true), _) => {
            println!("Device {} deleted", &device_id);
            Ok(())
        }
        (Ok(false), false) => {
            println!("Device {} not found", &device_id);
            Ok(())
        }
        (Ok(false), true) => Ok(()),
        (Err(e), _) => Err(e.into()),
    }
}

pub async fn read(config: &Context, app: AppId, device_id: DeviceId) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.get_device(app, &device_id).await {
        Ok(Some(dev)) => {
            util::show_json(serde_json::to_string(&dev)?);
            Ok(())
        }
        Ok(None) => {
            println!("Device {} not found", &device_id);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn create(
    config: &Context,
    device_id: DeviceId,
    data: serde_json::Value,
    app_id: AppId,
    file: Option<&str>,
) -> Result<()> {
    let device: Device = match file {
        Some(f) => util::get_data_from_file(f)?,
        None => {
            let mut device = Device::new(app_id, device_id);
            if let Some(spec) = data.as_object() {
                device.spec = spec.clone();
            }
            device
        }
    };

    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);
    match client.create_device(&device).await {
        Ok(_) => {
            println!("Device {} created", device.metadata.name);
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn edit(
    config: &Context,
    app: AppId,
    device_id: Option<DeviceId>,
    file: Option<&str>,
) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match (device_id, file) {
        (None, Some(f)) => {
            let data: Device = util::get_data_from_file(f)?;
            client.update_device(&data).await
        }
        (Some(id), None) => {
            //read device data
            let data = client.get_device(app, &id).await?;
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
        Ok(true) => {
            println!("Device updated");
            Ok(())
        }
        Ok(false) => {
            println!("Device or application not found");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn list(
    config: &Context,
    app: AppId,
    labels: Option<Values<'_>>,
    wide: bool,
) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let labels = util::clap_values_to_labels(labels);

    match client.list_devices(app, labels).await {
        Ok(Some(apps)) => {
            pretty_list(apps, wide);
            Ok(())
        }
        Ok(None) => {
            println!("No applications");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn set_gateway(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    gateway_id: DeviceId,
) -> Result<()> {
    // prepare json data to merge
    let data = json!({"spec": {
    "gatewaySelector": {
        "matchNames": [gateway_id]
    }
    }});

    merge_in(app, device_id, data, config).await
}

pub async fn set_password(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    password: String,
    username: Option<&str>,
) -> Result<()> {
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

    merge_in(app, device_id, data, config).await
}

pub async fn add_alias(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    new_alias: String,
) -> Result<()> {
    // prepare json data to merge
    let data = json!({"spec": {
    "alias": [
          new_alias
        ]
    }});

    merge_in(app, device_id, data, config).await
}

pub async fn add_labels(
    config: &Context,
    app: AppId,
    device_id: DeviceId,
    args: Values<'_>,
) -> Result<()> {
    let data = util::process_labels(&args);
    merge_in(app, device_id, data, config).await
}

//todo merge that with the same method in apps ?
/// merges a serde Value into the device object that exist on the server
async fn merge_in<A, D>(app: A, device: D, data: Value, config: &Context) -> Result<()>
where
    A: AsRef<str>,
    D: AsRef<str>,
{
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    //read app data
    let op = match client.get_device(app.as_ref(), device.as_ref()).await {
        Ok(Some(device)) => {
            serde_json::to_value(&device)?.merge(data);
            client.update_device(&device).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    };

    match op {
        Ok(true) => {
            println!("Device {} was successfully updated", device.as_ref());
            Ok(())
        }
        Ok(false) => {
            println!("Device or application does not exist");
            Ok(())
        }
        Err(e) => Err(e.into()),
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
