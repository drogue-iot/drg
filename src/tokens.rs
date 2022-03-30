use crate::config::Context;
use crate::util;

use anyhow::Result;
use reqwest::StatusCode;
use tabular::{Row, Table};

use drogue_client::tokens::v1::Client;

pub async fn get_api_keys(config: &Context) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let res = client.get_tokens().await?;
    let mut table = Table::new("{:<} | {:<} | {:<}");
    table.add_row(
        Row::new()
            .with_cell("TOKEN PREFIX")
            .with_cell("AGE")
            .with_cell("DESCRIPTION"),
    );

    if let Some(tokens) = res {
        for token in tokens {
            table.add_row(
                Row::new()
                    .with_cell(token.prefix)
                    .with_cell(util::age_from_timestamp(token.created)?)
                    .with_cell(token.description.unwrap_or_default()),
            );
        }
    }
    print!("{}", table);
    Ok(())
}

pub async fn create_api_key(config: &Context, description: Option<&str>) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let res = client.create_token(description).await?;
    if let Some(token) = res {
        println!("A new API Token was created:\n");
        println!("{}", token.token);
        println!("Make sure you save it, as you will not be able to display it again.");
    } else {
        println!("There was an error creating the API token")
    }
    Ok(())
}

pub async fn delete_api_key(config: &Context, prefix: &str) -> Result<()> {
    let client = Client::new(reqwest::Client::new(), config.registry_url.clone(), config);

    let res = client.delete_token(prefix).await?;

    if res {
        println!("Access token with prefix {} deleted", prefix);
    } else {
        println!("Access token with prefix {} was not found", prefix);
        util::exit_with_code(StatusCode::NOT_FOUND);
    }
    Ok(())
}
