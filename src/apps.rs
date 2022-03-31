use crate::config::Context;
use crate::{trust, util, AppId};
use anyhow::{anyhow, Result};
use clap::Values;
use json_value_merge::Merge;
use serde_json::{json, Value};
use std::process::exit;
use tabular::{Row, Table};

use drogue_client::registry::v1::Application;
use drogue_client::registry::v1::Client;

pub async fn create(
    config: &Context,
    app: Option<AppId>,
    data: serde_json::Value,
    file: Option<&str>,
) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let app: Application = match (file, app) {
        (Some(f), None) => {
            let app: Application = util::get_data_from_file(f)?;
            app
        }
        (None, Some(a)) => {
            let app: Application = serde_json::from_value(json!({
            "metadata": {
                "name": a,
            },
            "spec": data,
            }))
            .unwrap();
            app
        }
        // a file AND an app name should not be allowed by clap.
        _ => unreachable!(),
    };

    match client.create_app(&app).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn read(config: &Context, app: AppId) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.get_app(&app).await {
        Ok(Some(app)) => Ok(util::show_json(serde_json::to_string(&app)?)),
        Ok(None) => Ok(println!("Application {} not found", app)),
        Err(e) => Err(e.into()),
    }
}

pub async fn delete(config: &Context, app: AppId, ignore_missing: bool) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.delete_app(&app).await {
        Ok(res) => {
            if res {
                println!("App {} deleted", &app);
                Ok(())
            } else {
                if !ignore_missing {
                    Err(anyhow!("The application does not exist."))
                } else {
                    Ok(())
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn edit(config: &Context, app: AppId, file: Option<&str>) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match file {
        Some(f) => {
            let data: Application = util::get_data_from_file(f)?;

            client.update_app(&data).await
        }
        None => {
            //read app data
            let data = client.get_app(&app).await?;

            match data {
                Some(app) => {
                    let edited = util::editor(app)?;
                    client.update_app(&edited).await
                }
                None => Ok(false),
            }
        }
    };

    match op {
        Ok(true) => Ok(println!("Application {} was successfully updated", app)),
        Ok(false) => Ok(println!("Application {} does not exist", app)),
        Err(e) => Err(e.into()),
    }
}

pub async fn list(config: &Context, labels: Option<Values<'_>>) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let labels = labels.map(|mut labels| {
        let mut labels_vec: Vec<&str> = Vec::new();
        while let Some(l) = labels.next() {
            labels_vec.push(l)
        }
        labels_vec
    });

    match client.list_apps(labels).await {
        Ok(Some(apps)) => Ok(pretty_list(apps)),
        Ok(None) => Ok(println!("No applications")),
        Err(e) => Err(e.into()),
    }
}

pub async fn add_trust_anchor(
    config: &Context,
    app: &str,
    keyout: Option<&str>,
    key_pair_algorithm: Option<trust::SignAlgo>,
    days: Option<&str>,
    key_input: Option<rcgen::KeyPair>,
) -> Result<()> {
    let trust_anchor =
        trust::create_trust_anchor(app, keyout, key_pair_algorithm, days, key_input)?;

    let data = json!({"spec": {"trustAnchors": [ trust_anchor ]}} );

    merge_in(app, data, config).await
}

pub async fn get_trust_anchor(config: &Context, app: &str) -> Result<String> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    //read app data
    match client.get_app(app).await {
        Ok(Some(application)) => match application.spec.get("trustAnchors") {
            Some(anchor) => Ok(anchor.to_string().replace("\"", "")),
            None => {
                log::error!("No trust anchor found in this application.");
                exit(1);
            }
        },
        Ok(None) => {
            log::error!("Application not found.");
            exit(1)
        }
        Err(e) => {
            log::error!("Error : could not retrieve app");
            exit(2);
        }
    }
}

pub async fn add_labels(config: &Context, app: AppId, args: &Values<'_>) -> Result<()> {
    let data = util::process_labels(&args);
    merge_in(app, data, config).await
}

// merges a serde Value into the application object that exist on the server
async fn merge_in<A>(app: A, data: Value, config: &Context) -> Result<()>
where
    A: AsRef<str>,
{
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    //read app data
    let op = match client.get_app(app.as_ref()).await {
        Ok(Some(application)) => {
            serde_json::to_value(&application)?.merge(data);
            client.update_app(&application).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e.into()),
    };

    match op {
        Ok(true) => Ok(println!(
            "Application {} was successfully updated",
            app.as_ref()
        )),
        Ok(false) => Ok(println!("Application {} does not exist", app.as_ref())),
        Err(e) => Err(e.into()),
    }
}

fn pretty_list(apps: Vec<Application>) {
    let mut table = Table::new("{:<} {:<}");
    table.add_row(Row::new().with_cell("NAME").with_cell("AGE"));

    for app in apps {
        let name = app.metadata.name;
        let creation = app.metadata.creation_timestamp;

        table.add_row(
            Row::new()
                .with_cell(name)
                .with_cell(util::age_from_timestamp(creation)),
        );
    }

    print!("{}", table);
}
