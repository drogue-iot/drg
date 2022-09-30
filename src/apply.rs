use crate::DrogueError::InvalidInput;
use crate::{ApplicationOperation, Context, DeviceOperation, DrogueError, Outcome};
use drogue_client::registry::v1::{Application, Device};
use serde_json::Value;
use std::fs::{read_dir, File};
use std::io;
use std::io::BufReader;
use std::path::PathBuf;

enum Resource {
    Device(Device),
    Application(Application),
}

enum ResourceName {
    // app, dev
    Device(String, String),
    Application(String),
}

enum ExistenceOutcome {
    Update,
    Create,
    NoApp,
}

pub async fn apply(
    config: &Context,
    paths: Vec<&PathBuf>,
    ignore_resource_version: bool,
) -> Result<Outcome<String>, DrogueError> {
    let mut resources: Vec<Resource> = Vec::new();

    // explore directories and load every file there
    for p in paths {
        if p.is_dir() {
            for file in read_dir(p)? {
                match load_json(&file.unwrap().path()) {
                    Ok(r) => resources.push(r),
                    Err(e) => log::error!("{e}"),
                }
            }
        } else if p == &PathBuf::from("-") {
            match std_in() {
                Ok(r) => resources.push(r),
                Err(e) => log::error!("{e}"),
            }
        } else {
            match load_json(p) {
                Ok(r) => resources.push(r),
                Err(e) => log::error!("Cannot read file {:?} -> {e}", p),
            }
        }
    }

    for mut r in resources {
        match r {
            Resource::Device(ref mut dev) => {
                match check_existence(
                    config,
                    ResourceName::Device(
                        dev.metadata.application.clone(),
                        dev.metadata.name.clone(),
                    ),
                )
                .await?
                {
                    ExistenceOutcome::Update => {
                        if ignore_resource_version {
                            // an empty string will skip the field serialization
                            dev.metadata.resource_version = String::default();
                        }
                        match DeviceOperation::from_device(dev.clone()).edit(config).await {
                            Ok(_) => {
                                println!("Success updating device {}", dev.metadata.name.clone())
                            }
                            Err(e) => println!("Error updating device {}: {e}", dev.metadata.name),
                        }
                    }
                    ExistenceOutcome::Create => {
                        match DeviceOperation::from_device(dev.clone())
                            .create(config)
                            .await
                        {
                            Ok(_) => {
                                println!("Success creating device {}", dev.metadata.name.clone())
                            }
                            Err(e) => println!("Error creating device {}: {e}", dev.metadata.name),
                        }
                    }
                    ExistenceOutcome::NoApp => log::error!(
                        "Cannot apply device {} : application {} does not exist",
                        dev.metadata.name,
                        dev.metadata.application
                    ),
                }
            }
            Resource::Application(ref mut app) => {
                match check_existence(config, ResourceName::Application(app.metadata.name.clone()))
                    .await?
                {
                    ExistenceOutcome::Update => {
                        if ignore_resource_version {
                            // an empty string will skip the field serialization
                            app.metadata.resource_version = String::default();
                        }
                        match ApplicationOperation::from_application(app.clone())
                            .edit(config)
                            .await
                        {
                            Ok(_) => println!("Success updating app {}", app.metadata.name.clone()),
                            Err(e) => println!("Error updating app {}: {e}", app.metadata.name),
                        }
                    }
                    ExistenceOutcome::Create => {
                        match ApplicationOperation::from_application(app.clone())
                            .create(config)
                            .await
                        {
                            Ok(_) => println!("Success creating app {}", app.metadata.name.clone()),
                            Err(e) => println!("Error creating app {}: {e}", app.metadata.name),
                        }
                    }
                    ExistenceOutcome::NoApp => unreachable!(),
                }
            }
        }
    }

    //fixme drg don't really handles multiple operations yet :)
    Ok(Outcome::SuccessWithMessage("Finished apply".to_string()))
}

fn std_in() -> Result<Resource, DrogueError> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    let json: Value = serde_yaml::from_reader(reader)?;
    deser(json)
}

fn load_json(path: &PathBuf) -> Result<Resource, DrogueError> {
    if path.is_dir() {
        log::debug!("path {:?} is a subdirectory, skipping.", path);
        Err(InvalidInput("Ignored subdirectory".to_string()))
    } else if path.file_name().unwrap().to_str().unwrap().starts_with('.') {
        log::debug!("path {:?} is a hidden file, skipping.", path);
        Err(InvalidInput("Ignored hidden file".to_string()))
    } else {
        log::debug!("reading {:?}", path);
        let f = File::open(path)?;
        let reader = BufReader::new(f);

        let json: Value = serde_yaml::from_reader(reader)?;
        deser(json)
    }
}

fn deser(json: Value) -> Result<Resource, DrogueError> {
    if json.get("metadata").unwrap().get("application").is_some() {
        let dev: Device = serde_json::from_value(json)?;
        Ok(Resource::Device(dev))
    } else {
        let app: Application = serde_json::from_value(json)?;
        Ok(Resource::Application(app))
    }
}

async fn check_existence(
    context: &Context,
    resource: ResourceName,
) -> Result<ExistenceOutcome, DrogueError> {
    // here the two unwraps are safe because operation::new can only yield an error when trying to read a file
    match resource {
        ResourceName::Device(app, dev) => {
            let op = DeviceOperation::new(app.clone(), Some(dev.clone()), None, None).unwrap();
            match op.read(context).await {
                Ok(_) => Ok(ExistenceOutcome::Update),
                // 404 response, let's try if the app even exist
                Err(DrogueError::NotFound) => {
                    log::info!(
                        "Device {} does not exist, verify if application {} exists.",
                        &dev,
                        &app
                    );
                    let op = ApplicationOperation::new(Some(app), None, None).unwrap();
                    match op.read(context).await {
                        Ok(_) => Ok(ExistenceOutcome::Create),
                        Err(DrogueError::NotFound) => Ok(ExistenceOutcome::NoApp),
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
        }
        ResourceName::Application(app) => {
            let op = ApplicationOperation::new(Some(app), None, None).unwrap();
            match op.read(context).await {
                Ok(_) => Ok(ExistenceOutcome::Update),
                Err(DrogueError::NotFound) => Ok(ExistenceOutcome::Create),
                Err(e) => Err(e),
            }
        }
    }
}
