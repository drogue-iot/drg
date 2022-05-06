use crate::config::Context;
use crate::handle_operation;
use crate::util::{self, DrogueError, Outcome};

use drogue_client::admin::v1::{Client, MemberEntry, Members, Role};
use tabular::{Row, Table};

pub async fn member_list(config: &Context, app: &str) -> Result<Outcome<Members>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    handle_operation!(client.get_members(app).await)
}
pub async fn member_delete(
    config: &Context,
    app: &str,
    username: &str,
) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

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

pub async fn member_edit(config: &Context, app: &str) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    let op = match client.get_members(app).await {
        Ok(Some(members)) => {
            let data = util::editor(members)?;
            client.update_members(app, data).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    };

    handle_operation!(op, "Application members updated")
}

pub async fn member_add(
    config: &Context,
    app: &str,
    user: &str,
    role: Role,
) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.registry_url.clone(),
        config.token.clone(),
    );

    let op = match client.get_members(app).await {
        Ok(Some(mut members)) => {
            members
                .members
                .insert(user.to_string(), MemberEntry { role });

            client.update_members(app, members).await
        }
        Ok(None) => Ok(false),
        Err(e) => Err(e),
    };

    handle_operation!(op, "Application members updated")
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
