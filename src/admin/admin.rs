use crate::config::Context;
use crate::util;

use crate::outcome::Outcome::SuccessWithMessage;
use crate::outcome::{DrogueError, Outcome};
use anyhow::Result;
use drogue_client::admin::v1::{Client, MemberEntry, Members, Role};
use tabular::{Row, Table};

pub async fn member_list(config: &Context, app: &str) -> Result<Outcome<Members>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.get_members(app).await {
        Ok(Some(members)) => Ok(Outcome::SuccessWithJsonData(members)),
        Ok(None) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
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

    match op {
        Ok(true) => Ok(SuccessWithMessage(
            "Application members updated".to_string(),
        )),
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
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

    match op {
        Ok(true) => Ok(SuccessWithMessage(
            "Application members updated".to_string(),
        )),
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
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

    match op {
        Ok(true) => Ok(SuccessWithMessage(
            "Application members updated".to_string(),
        )),
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
}

pub async fn transfer_app(config: &Context, app: &str, user: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    //TODO : the long message should be a pretty print with the URL
    match client.initiate_app_transfer(app, user).await {
        Ok(true) => {
            let msg = format!("Application transfer initated\nThe new user can accept the transfer with \"drg admin transfer accept {}\"",
                                  app
                );
            let msg = if let Ok(console) = util::get_drogue_console_endpoint(config).await {
                format!(
                    "{}\nAlternatively you can share this link with the new owner :\n{}transfer/{}",
                    msg,
                    console.as_str(),
                    urlencoding::encode(app)
                )
            } else {
                msg
            };
            Ok(Outcome::SuccessWithMessage(msg))
        }
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
}

pub async fn cancel_transfer(config: &Context, app: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.cancel_app_transfer(app).await {
        Ok(true) => Ok(Outcome::SuccessWithMessage(
            "Application transfer canceled".to_string(),
        )),
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
}

pub async fn accept_transfer(config: &Context, app: &str) -> Result<Outcome<String>> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.accept_app_transfer(app).await {
        Ok(true) => Ok(Outcome::SuccessWithMessage(
            "Application transfer completed. \n You are now the owner of the application"
                .to_string(),
        )),
        Ok(false) => Err(DrogueError::NotFound.into()),
        Err(e) => Err(e.into()),
    }
}

pub fn members_table(members: Members) {
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
