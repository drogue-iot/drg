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
) -> Vec<Result<Outcome<String>, DrogueError>> {
    let mut resources: Vec<Resource> = Vec::new();
    let mut results: Vec<Result<Outcome<String>, DrogueError>> = Vec::new();

    // explore directories and load every file there
    for p in paths {
        if p.is_dir() {
            match read_dir(p) {
                Err(e) => results.push(Err(InvalidInput(e.to_string()))),
                Ok(files) => {
                    for file in files {
                        match load_json(&file.unwrap().path()) {
                            Ok(r) => resources.push(r),
                            Err(e) => results.push(Err(e)),
                        }
                    }
                }
            }
        } else if p == &PathBuf::from("-") {
            match std_in() {
                Ok(r) => resources.push(r),
                Err(e) => results.push(Err(e)),
            }
        } else {
            match load_json(p) {
                Ok(r) => resources.push(r),
                Err(e) => results.push(Err(InvalidInput(format!(
                    "Cannot read file {:?} -> {e}",
                    p
                )))),
            }
        }
    }

    for mut r in resources {
        match r {
            Resource::Device(ref mut dev) => {
                if ignore_resource_version {
                    // an empty string will skip the field serialization
                    dev.metadata.resource_version = String::default();
                }
                results.push(apply_device(config, dev).await);
            }
            Resource::Application(ref mut app) => {
                if ignore_resource_version {
                    // an empty string will skip the field serialization
                    app.metadata.resource_version = String::default();
                }
                results.push(apply_app(config, app).await);
            }
        }
    }

    results
}

async fn apply_device(config: &Context, dev: &Device) -> Result<Outcome<String>, DrogueError> {
    match check_existence(
        config,
        ResourceName::Device(dev.metadata.application.clone(), dev.metadata.name.clone()),
    )
    .await?
    {
        ExistenceOutcome::Update => DeviceOperation::from_device(dev.clone()).edit(config).await,
        ExistenceOutcome::Create => {
            DeviceOperation::from_device(dev.clone())
                .create(config)
                .await
        }
        ExistenceOutcome::NoApp => Err(InvalidInput(format!(
            "Cannot apply device {} : application {} does not exist",
            dev.metadata.name, dev.metadata.application
        ))),
    }
}

async fn apply_app(config: &Context, app: &Application) -> Result<Outcome<String>, DrogueError> {
    match check_existence(config, ResourceName::Application(app.metadata.name.clone())).await? {
        ExistenceOutcome::Update => {
            ApplicationOperation::from_application(app.clone())
                .edit(config)
                .await
        }
        ExistenceOutcome::Create => {
            ApplicationOperation::from_application(app.clone())
                .create(config)
                .await
        }
        ExistenceOutcome::NoApp => unreachable!(),
    }
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
    if let Some(metadata) = json.get("metadata") {
        if metadata.get("application").is_some() {
            let dev: Device = serde_json::from_value(json)?;
            Ok(Resource::Device(dev))
        } else {
            let app: Application = serde_json::from_value(json)?;
            Ok(Resource::Application(app))
        }
    } else {
        Err(DrogueError::InvalidInput(
            "Input data does not represent a valid drogue resource".to_string(),
        ))
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
                        "Device {} does not exist, verifying if application {} exists.",
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
