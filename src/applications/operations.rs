use crate::applications::ApplicationOperation;
use crate::config::Context;
use crate::handle_operation;
use crate::util::{self, DrogueError, Outcome};

use anyhow::Result;
use clap::Values;
use json_value_merge::Merge;
use serde_json::{json, Value};
use tabular::{Row, Table};

use drogue_client::registry::v1::Client;
use drogue_client::registry::v1::{Application, ApplicationSpecTrustAnchors};
use drogue_client::Translator;

impl ApplicationOperation {
    pub async fn create(&self, config: &Context) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        Ok(client
            .create_app(&self.payload)
            .await
            .map(|_| Outcome::SuccessWithMessage("Application created".to_string()))?)
    }

    pub async fn read(&self, config: &Context) -> Result<Outcome<Application>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let op = client.get_app(self.name.as_ref().unwrap()).await;
        handle_operation!(op)
    }

    pub async fn delete(&self, config: &Context, ignore_missing: bool) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        match (
            client.delete_app(&self.name.as_ref().unwrap()).await,
            ignore_missing,
        ) {
            (Ok(true), _) => Ok(Outcome::SuccessWithMessage(
                "Application deleted".to_string(),
            )),
            (Ok(false), false) => Err(DrogueError::NotFound.into()),
            (Ok(false), true) => Ok(Outcome::SuccessWithMessage(
                "No application to delete, ignoring.".to_string(),
            )),
            (Err(e), _) => Err(e.into()),
        }
    }

    pub async fn edit(&self, config: &Context) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let op = match &self.name {
            None => client.update_app(&self.payload).await,
            Some(name) => {
                //read app data
                match client.get_app(name).await? {
                    Some(app) => {
                        let edited = util::editor(app)?;
                        client.update_app(&edited).await
                    }
                    None => Ok(false),
                }
            }
        };

        handle_operation!(op, "Application updated")
    }

    pub async fn list(
        &self,
        config: &Context,
        labels: Option<Values<'_>>,
    ) -> Result<Outcome<Vec<Application>>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        let labels = util::clap_values_to_labels(labels);

        handle_operation!(client.list_apps(labels).await)
    }

    pub async fn add_trust_anchor(
        &self,
        config: &Context,
        keyout: Option<&str>,
        key_pair_algorithm: Option<util::SignAlgo>,
        days: Option<&str>,
        key_input: Option<rcgen::KeyPair>,
    ) -> Result<Outcome<String>> {
        let trust_anchor = util::create_trust_anchor(
            self.name.as_ref().unwrap(),
            keyout,
            key_pair_algorithm,
            days,
            key_input,
        )?;

        let anchors = ApplicationSpecTrustAnchors {
            anchors: vec![trust_anchor],
        };
        let data = json!({"spec": {"trustAnchors": anchors }} );
        self.merge_in(data, config).await
    }

    pub async fn get_trust_anchor(&self, config: &Context) -> Result<ApplicationSpecTrustAnchors> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        match client.get_app(&self.name.as_ref().unwrap()).await {
            Ok(Some(application)) => {
                match application
                    .section::<ApplicationSpecTrustAnchors>()
                    .transpose()?
                {
                    Some(anchors) => Ok(anchors),
                    None => {
                        Err(DrogueError::User("No trust anchors for this app".to_string()).into())
                    }
                }
            }
            Ok(None) => Err(DrogueError::NotFound.into()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn add_labels(&self, config: &Context, args: &Values<'_>) -> Result<Outcome<String>> {
        let data = util::process_labels(args);
        self.merge_in(data, config).await
    }

    // merges a serde Value into the application object that exist on the server
    async fn merge_in(&self, data: Value, config: &Context) -> Result<Outcome<String>> {
        let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

        //read app data
        let op = match client.get_app(&self.name.as_ref().unwrap()).await {
            Ok(Some(app)) => {
                let mut new = serde_json::to_value(&app)?;
                new.merge(data);
                client.update_app(&serde_json::from_value(new)?).await
            }
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        };

        handle_operation!(op, "Application updated")
    }
}

pub fn pretty_list(apps: &Vec<Application>) {
    let mut table = Table::new("{:<} {:<}");
    table.add_row(Row::new().with_cell("NAME").with_cell("AGE"));

    for app in apps {
        let name = app.metadata.name.clone();
        let creation = app.metadata.creation_timestamp;

        table.add_row(
            Row::new()
                .with_cell(name)
                .with_cell(util::age_from_timestamp(&creation)),
        );
    }

    print!("{}", table);
}
