mod admin;
pub mod applications;
pub mod arguments;
pub mod command;
pub mod config;
pub mod devices;
pub mod openid;
pub mod stream;
pub mod util;

use admin::tokens;
use applications::ApplicationOperation;
use arguments::cli::{Action, Parameters, ResourceId, ResourceType, Transfer};
use config::{AccessToken, Config, Context};
use devices::DeviceOperation;
use openid::login;
use stream::stream_app;
use util::{display, display_simple, DrogueError};

use anyhow::{Context as AnyhowContext, Result};
use clap::ArgMatches;
use std::io::Write;
use std::str::FromStr;
use url::Url;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn start_wasm(refresh_token: String, api_url: Url) -> Result<()> {
    // first we log in
    let context = login(api_url, Some(&refresh_token), "default".to_string()).await?;
    interactive_mode()
}

pub async fn interactive_mode() -> Result<()> {
    loop {
        let matches = match arguments::cli::app_arguments().try_get_matches_from(prompt()) {
            Ok(matches) => matches,
            Err(e) => {
                let _ = e.print();
                continue;
            }
        };

        if matches.subcommand_name() == Some("exit") {
            break;
        }

        process_arguments(matches).await?;
    }
    Ok(())
}

fn prompt() -> Vec<String> {
    let mut line = String::new();
    print!("drg 🚀 ");
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    line = line.trim().to_string();
    line.insert_str(0, "drg ");

    return line
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
}

//todo avoid config reloads and writes

pub async fn process_arguments(matches: ArgMatches) -> Result<i32> {
    // load the config
    let config_path = matches.value_of(Parameters::config.as_ref());
    let config_result = Config::from(config_path);

    let (command, submatches) = matches.subcommand().unwrap();
    let context_arg = matches
        .value_of(ResourceId::contextId.as_ref())
        .map(|s| s.to_string());

    if command == Action::login.as_ref() {
        let mut config = config_result.unwrap_or_else(|_| Config::empty());
        arguments::login::subcommand(submatches, &mut config, &context_arg).await?;

        config.write(config_path)?;
        return Ok(0);
    } else if command == Action::version.as_ref() {
        util::print_version(config_result.ok().as_ref()).await;
        return Ok(0);
    }

    let mut config = config_result?;

    if command == Action::config.as_ref() {
        //fixme handle the pretty print: issue #107
        let code = arguments::config::subcommand(submatches, &mut config, &context_arg)?;
        config.write(config_path)?;
        return Ok(code);
    }

    // The following commands needs a context and a valid token
    openid::verify_token_validity(config.get_context_mut(&context_arg)?).await?;

    let context = config.get_context(&context_arg)?;

    if command == Action::whoami.as_ref() {
        let (_, submatches) = matches.subcommand().unwrap();
        if submatches.is_present(Parameters::token.as_ref()) {
            util::print_token(context);
        } else if let Some((_, endpoints_matches)) = submatches.subcommand() {
            let service = match endpoints_matches.value_of(Parameters::endpoints.as_ref()) {
                Some("*") => None,
                s => s,
            };
            util::print_endpoints(context, service).await?;
        } else {
            util::print_whoami(context);
            util::print_version(Some(&config)).await;
        }
        config.write(config_path)?;
        return Ok(0);
    }

    log::warn!("Using context: {}", context.name);
    let verb = Action::from_str(command);
    let cmd = submatches;

    let json_output = cmd
        .value_of(Parameters::output.as_ref())
        .map(|s| s == "json")
        .unwrap_or(false);

    let exit_code = match verb? {
        Action::create => arguments::create::subcommand(cmd, context, json_output).await?,
        Action::delete => arguments::delete::subcommand(cmd, context, json_output).await?,
        Action::edit => arguments::edit::subcommand(cmd, context, json_output).await?,
        Action::get => arguments::get::subcommand(cmd, context, json_output).await?,
        Action::set => {
            let (target, command) = cmd.subcommand().unwrap();
            let app_id = arguments::get_app_id(command, context)?;

            let id = command
                .value_of(ResourceId::deviceId.as_ref())
                .map(|s| s.to_string());

            let op = DeviceOperation::new(app_id.clone(), id, None, None)?;

            match ResourceType::from_str(target)? {
                ResourceType::gateway => {
                    let gateway_id = command
                        .value_of(ResourceId::gatewayId.as_ref())
                        .unwrap()
                        .to_string();
                    display_simple(op.set_gateway(context, gateway_id).await, json_output)
                }
                ResourceType::password => {
                    let password = command
                        .value_of(Parameters::password.as_ref())
                        .unwrap()
                        .to_string();
                    let username = command.value_of(ResourceId::username.as_ref());
                    display_simple(
                        op.set_password(context, password, username).await,
                        json_output,
                    )
                }
                ResourceType::alias => {
                    let alias = command
                        .value_of(Parameters::alias.as_ref())
                        .unwrap()
                        .to_string();

                    display_simple(op.add_alias(context, alias).await, json_output)
                }
                // The other enum variants are not exposed by clap
                _ => unreachable!(),
            }?
        }
        Action::command => {
            let command = cmd.value_of(Parameters::command.as_ref()).unwrap();
            let app_id = arguments::get_app_id(cmd, context)?;
            let device = cmd.value_of(ResourceId::deviceId.as_ref()).unwrap();

            let body = match cmd.value_of(Parameters::filename.as_ref()) {
                Some(f) => util::get_data_from_file(f)?,
                None => {
                    let data = cmd.value_of(Parameters::payload.as_ref()).unwrap();
                    serde_json::from_str(data)
                        .context(format!("Can't parse data args: \'{data}\' into json",))?
                }
            };

            display_simple(
                command::send_command(context, app_id.as_str(), device, command, body).await,
                json_output,
            )?
        }
        Action::transfer => {
            let task = Transfer::from_str(command);

            match task? {
                Transfer::init => {
                    let user = cmd.value_of(Parameters::username.as_ref()).unwrap();
                    let id = arguments::get_app_id(cmd, context)?;
                    display(
                        admin::transfer_app(context, id.as_str(), user).await,
                        json_output,
                        admin::app_transfer_guide,
                    )?
                }
                Transfer::accept => {
                    let id = cmd.value_of(ResourceId::applicationId.as_ref()).unwrap();
                    display_simple(admin::accept_transfer(context, id).await, json_output)?
                }
                Transfer::cancel => {
                    let id = cmd.value_of(ResourceId::applicationId.as_ref()).unwrap();
                    display_simple(admin::cancel_transfer(context, id).await, json_output)?
                }
            }
        }
        Action::stream => {
            let (_, matches) = matches.subcommand().unwrap();
            let app_id = arguments::get_app_id(matches, context)?;
            let count = matches
                .value_of(Parameters::count.as_ref())
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap_or(usize::MAX);
            let device = matches.value_of(Parameters::device.as_ref());

            stream::stream_app(context, &app_id, device, count).await?;
            0
        }

        Action::label => {
            let (target, command) = cmd.subcommand().unwrap();
            let labels = command.values_of(Parameters::label.as_ref()).unwrap();

            match ResourceType::from_str(target)? {
                ResourceType::application => {
                    let app = command
                        .value_of(ResourceId::applicationId.as_ref())
                        .unwrap()
                        .to_string();

                    display_simple(
                        ApplicationOperation::new(Some(app), None, None)?
                            .add_labels(context, &labels)
                            .await,
                        json_output,
                    )
                }
                ResourceType::device => {
                    let app_id = arguments::get_app_id(command, context)?;

                    let device = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .unwrap()
                        .to_string();

                    display_simple(
                        DeviceOperation::new(app_id, Some(device), None, None)?
                            .add_labels(context, &labels)
                            .await,
                        json_output,
                    )
                }
                _ => unreachable!(),
            }?
        }
        _ => unimplemented!(),
    };

    // if the config was changed, save it
    config.write(config_path)?;
    Ok(exit_code)
}
