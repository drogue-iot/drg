use crate::config::Context;
use crate::util;

use crate::util::{handle_operation, DrogueError, Outcome};
use anyhow::Result;
use drogue_client::admin::v1::{Client, MemberEntry, Members, Role};
use serde::Serialize;
use tabular::{Row, Table};
use url::Url;

pub async fn member_list(config: &Context, app: &str) -> Result<Outcome<Members>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    handle_operation!(client.get_members(app).await)
}
pub async fn member_delete(config: &Context, app: &str, username: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match client.get_members(app).await {
        Ok(Some(mut members)) => {
            members.members.remove(&username.to_string());

            client.update_members(app, members).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    };

    handle_operation!(op, "Application members updated")
}

pub async fn member_edit(config: &Context, app: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match client.get_members(app).await {
        Ok(Some(members)) => {
            let data = util::editor(members)?;
            client.update_members(app, data).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e.into()),
    };

    handle_operation!(op, "Application members updated")
}

pub async fn member_add(
    config: &Context,
    app: &str,
    user: &str,
    role: Role,
) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match client.get_members(app).await {
        Ok(Some(mut members)) => {
            members
                .members
                .insert(user.to_string(), MemberEntry { role });

            client.update_members(app, members).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e.into()),
    };

    handle_operation!(op, "Application members updated")
}

pub async fn transfer_app(config: &Context, app: &str, user: &str) -> Result<Outcome<AppTransfer>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.initiate_app_transfer(app, user).await {
        Ok(true) => {
            let console = util::get_drogue_console_endpoint(config).await.ok();
            Ok(Outcome::SuccessWithJsonData(AppTransfer {
                console,
                app: app.to_string(),
            }))
        }
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
}

pub async fn cancel_transfer(config: &Context, app: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    handle_operation!(
        client.cancel_app_transfer(app).await,
        "Application transfer canceled"
    )
}

pub async fn accept_transfer(config: &Context, app: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    handle_operation!(
        client.accept_app_transfer(app).await,
        "Application transfer completed. \n You are now the owner of the application"
    )
}

pub fn members_table(members: &Members) {
    let mut table = Table::new("{:<} | {:<}");
    table.add_row(Row::new().with_cell("USER").with_cell("ROLE"));

    if !members.members.is_empty() {
        for (user, entry) in members.members.iter() {
            table.add_row(Row::new().with_cell(user).with_cell(entry.role));
        }
        println!("{}", table);
    } else {
        println!("The member list for this application is empty.");
    }
}

#[derive(Serialize)]
pub struct AppTransfer {
    console: Option<Url>,
    app: String,
}

pub fn app_transfer_guide(transfer: &AppTransfer) {
    println!("Application transfer initated.");
    println!(
        "The new user can accept the transfer with \"drg admin transfer accept {}\"",
        transfer.app
    );

    if let Some(console) = &transfer.console {
        println!(
            "Alternatively you can share this link with the new owner :\n{}transfer/{}",
            console.as_str(),
            urlencoding::encode(&transfer.app)
        )
    }
}
