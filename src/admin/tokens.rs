use crate::config::Context;
use crate::handle_operation;
use crate::util;

use tabular::{Row, Table};

use crate::util::{DrogueError, Outcome};
use drogue_client::tokens::v1::{AccessToken, Client, CreatedAccessToken};

pub async fn get_api_keys(config: &Context) -> Result<Outcome<Vec<AccessToken>>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.drogue_cloud_url.clone(),
        config.token.clone(),
    );
    handle_operation!(client.get_tokens().await)
}

pub async fn create(
    config: &Context,
    description: Option<&str>,
) -> Result<Outcome<CreatedAccessToken>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.drogue_cloud_url.clone(),
        config.token.clone(),
    );

    handle_operation!(client.create_token(description).await)
}

pub async fn delete(config: &Context, prefix: &str) -> Result<Outcome<String>, DrogueError> {
    let client = Client::new(
        reqwest::Client::new(),
        config.drogue_cloud_url.clone(),
        config.token.clone(),
    );

    handle_operation!(client.delete_token(prefix).await, "Access token deleted")
}

pub fn tokens_table(tokens: &Vec<AccessToken>) {
    let mut table = Table::new("{:<} | {:<} | {:<}");
    table.add_row(
        Row::new()
            .with_cell("TOKEN PREFIX")
            .with_cell("AGE")
            .with_cell("DESCRIPTION"),
    );

    for token in tokens {
        table.add_row(
            Row::new()
                .with_cell(&token.prefix)
                .with_cell(util::age_from_timestamp(&token.created))
                .with_cell(&token.description.clone().unwrap_or_default()),
        );
    }
    print!("{}", table);
}

pub fn created_token_print(token: &CreatedAccessToken) {
    println!("A new API Token was created:\n");
    println!("{}", token.token);
    println!("Make sure you save it, as you will not be able to display it again.");
}
