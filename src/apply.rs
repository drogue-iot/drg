use crate::DrogueError::InvalidInput;
use crate::{ApplicationOperation, Context, DeviceOperation, DrogueError, Outcome};
use drogue_client::registry::v1::{Application, Device};
use serde_json::Value;
use std::fs::{read_dir, File};
use std::io::BufReader;
use std::path::PathBuf;

enum Resource {
    Device(Device),
    Application(Application),
}

pub async fn apply(
    config: &Context,
    mut paths: Vec<&PathBuf>,
) -> Result<Outcome<String>, DrogueError> {
    let mut resources: Vec<Resource> = Vec::new();

    // explore directories and load every file there
    for p in paths {
        if p.is_dir() {
            for file in read_dir(p)? {
                match load_json(&file.unwrap().path()) {
                    Ok(r) => resources.push(r),
                    Err(e) => log::warn!("{e}"),
                }
            }
        } else {
            match load_json(p) {
                Ok(r) => resources.push(r),
                Err(e) => log::warn!("{e}"),
            }
        }
    }

    for mut r in resources {
        let version = get_resource_version(config, &r).await;
        match r {
            Resource::Device(mut dev) => {
                // The device exists
                if let Some(version) = version {
                    dev.metadata.resource_version = version;
                    match DeviceOperation::from_device(dev.clone()).edit(config).await {
                        Ok(_) => println!("Success updating device {}", dev.metadata.name.clone()),
                        Err(e) => println!("Error updating device {}: {e}", dev.metadata.name),
                    }
                }
                // the device do not exist, create it
                else {
                    match DeviceOperation::from_device(dev.clone())
                        .create(config)
                        .await
                    {
                        Ok(_) => println!("Success creating device {}", dev.metadata.name.clone()),
                        Err(e) => println!("Error creating device {}: {e}", dev.metadata.name),
                    }
                }
            }
            Resource::Application(mut app) => {
                // The app exists
                if let Some(version) = version {
                    app.metadata.resource_version = version;
                    match ApplicationOperation::from_application(app.clone())
                        .edit(config)
                        .await
                    {
                        Ok(_) => println!("Success updating app {}", app.metadata.name.clone()),
                        Err(e) => println!("Error updating app {}: {e}", app.metadata.name),
                    }
                }
                // the application do not exist, create it
                else {
                    match ApplicationOperation::from_application(app.clone())
                        .create(config)
                        .await
                    {
                        Ok(_) => println!("Success creating app {}", app.metadata.name.clone()),
                        Err(e) => println!("Error creating app {}: {e}", app.metadata.name),
                    }
                }
            }
        }
    }

    //fixme drg don't really handles multiple operations yet :)
    Ok(Outcome::SuccessWithMessage("Finished apply".to_string()))
}

// fixme if a file cannot be loaded (invalid), simply display a warning and carry on ?
fn load_json(path: &PathBuf) -> Result<Resource, DrogueError> {
    if path.is_dir() {
        log::debug!("path {:?} is a subdirectory, skipping.", path);
        Err(InvalidInput("Ignored subdirectory".to_string()))
    } else if path.file_name().unwrap().to_str().unwrap().starts_with(".") {
        log::debug!("path {:?} is a hidden file, skipping.", path);
        Err(InvalidInput("Ignored hidden file".to_string()))
    } else {
        log::debug!("reading {:?}", path);
        let f = File::open(path)?;
        let reader = BufReader::new(f);

        let json: Value = serde_json::from_reader(reader)?;

        if json.get("metadata").unwrap().get("application").is_some() {
            let dev: Device = serde_json::from_value(json)?;
            Ok(Resource::Device(dev))
        } else {
            let app: Application = serde_json::from_value(json)?;
            Ok(Resource::Application(app))
        }
    }
}

//TODO, check for app existence before reading the device
async fn get_resource_version(context: &Context, resource: &Resource) -> Option<String> {
    // here the two unwraps are safe because operation::new can only yield an error when trying to read a file
    match resource {
        Resource::Device(dev) => {
            let op = DeviceOperation::new(
                dev.metadata.application.clone(),
                Some(dev.metadata.name.clone()),
                None,
                None,
            )
            .unwrap();
            op.read(context)
                .await
                .and_then(|o| o.inner())
                .map(|d| d.metadata.resource_version)
                .ok()
        }
        Resource::Application(app) => {
            let op =
                ApplicationOperation::new(Some(app.metadata.name.clone()), None, None).unwrap();
            op.read(context)
                .await
                .and_then(|o| o.inner())
                .map(|a| a.metadata.resource_version)
                .ok()
        }
    }
}
