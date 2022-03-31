use crate::config::Context;
use crate::util;

use anyhow::Result;
pub use drogue_client::admin::v1::Role;
use drogue_client::admin::v1::{Client, MemberEntry};
use tabular::{Row, Table};

pub async fn member_list(config: &Context, app: &str) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let res = client.get_members(app).await?;

    let mut table = Table::new("{:<} | {:<}");
    table.add_row(Row::new().with_cell("USER").with_cell("ROLE"));

    match res {
        Some(members) => {
            for (user, entry) in members.members.iter() {
                table.add_row(Row::new().with_cell(user).with_cell(entry.role));
            }
            println!("{}", table);
        }
        None => {
            println!("No members found for this application.");
        }
    };

    Ok(())
}
pub async fn member_delete(config: &Context, app: &str, username: &str) -> Result<()> {
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
        Ok(true) => {
            println!("Application members updated");
            Ok(())
        }
        Ok(false) => {
            println!("Application not found");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn member_edit(config: &Context, app: &str) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match client.get_members(app).await {
        Ok(Some(members)) => {
            let data = util::editor(members)?;
            client.update_members(app, data).await
        }
        Ok(None) => {
            println!("Application {} not found", app);
            Ok(false)
        }
        Err(e) => Err(e),
    };

    match op {
        Ok(true) => {
            println!("Application members updated");
            Ok(())
        }
        Ok(false) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub async fn member_add(config: &Context, app: &str, username: &str, role: Role) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let op = match client.get_members(app).await {
        Ok(Some(mut members)) => {
            println!("{:?}", members);
            members
                .members
                .insert(username.to_string(), MemberEntry { role });

            println!("{:?}", members);

            client.update_members(app, members).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    };

    match op {
        Ok(true) => {
            println!("Application members updated");
            Ok(())
        }
        Ok(false) => {
            println!("Application not found");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn transfer_app(config: &Context, app: &str, username: &str) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.initiate_app_transfer(app, username).await {
        Ok(true) => {
            println!("Application transfer initated");
            println!(
                "The new user can accept the transfer with \"drg admin transfer accept {}\"",
                app
            );
            if let Ok(console) = util::get_drogue_console_endpoint(config) {
                println!("Alternatively you can share this link with the new owner :");
                println!("{}transfer/{}", console.as_str(), urlencoding::encode(app));
            }
            Ok(())
        }
        Ok(false) => {
            println!("The application does not exist");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn cancel_transfer(config: &Context, app: &str) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.cancel_app_transfer(app).await {
        Ok(true) => {
            println!("Application transfer canceled");
            Ok(())
        }
        Ok(false) => {
            println!("The application does not exist");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn accept_transfer(config: &Context, app: &str) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    match client.accept_app_transfer(app).await {
        Ok(true) => {
            println!("Application transfer completed.");
            println!("You are now the owner of application {}", app);
            Ok(())
        }
        Ok(false) => {
            println!("The application does not exist");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
