mod admin;
mod applications;
mod arguments;
mod command;
mod config;
mod devices;
mod openid;
mod stream;
mod util;

use arguments::cli::{Action, Parameters, ResourceId, ResourceType, Transfer};
use std::io::Write;

use crate::admin::tokens;
use crate::applications::ApplicationOperation;
use crate::config::{AccessToken, Config, Context};
use crate::devices::DeviceOperation;
use crate::util::{display, display_simple, DrogueError, Outcome};

use anyhow::{Context as AnyhowContext, Result};
use clap::ArgMatches;
use std::process::exit;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = arguments::cli::app_arguments().get_matches();

    simple_logger::SimpleLogger::new()
        .with_level(util::log_level(&matches))
        .init()
        .unwrap();

    let code = if matches.is_present(Parameters::interactive.as_ref()) {
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
        0
    } else {
        process_arguments(matches).await?
    };

    exit(code)
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

async fn process_arguments(matches: ArgMatches) -> Result<i32> {
    // load the config
    let config_path = matches.value_of(Parameters::config.as_ref());
    let config_result = Config::from(config_path);

    let (command, submatches) = matches.subcommand().unwrap();
    let context_arg = matches
        .value_of(ResourceId::contextId.as_ref())
        .map(|s| s.to_string());

    let json_output = submatches
        .value_of(Parameters::output.as_ref())
        .map(|s| s == "json")
        .unwrap_or(false);

    if command == Action::login.as_ref() {
        let mut config = config_result.unwrap_or_else(|_| Config::empty());
        let code = display_simple(
            arguments::login::subcommand(submatches, &mut config, &context_arg).await,
            json_output,
        );

        config.write(config_path)?;
        return code;
    } else if command == Action::version.as_ref() {
        util::print_version(config_result.ok().as_ref()).await;
        return Ok(0);
    }

    let mut config = config_result?;

    if command == Action::config.as_ref() {
        //fixme handle the pretty print: issue #107
        let code =
            arguments::config::subcommand(submatches, &mut config, &context_arg, json_output)?;
        config.write(config_path)?;
        return Ok(code);
    }

    // The following commands needs a context and a valid token
    openid::verify_token_validity(config.get_context_mut(&context_arg)?).await?;

    let context = config.get_context_mut(&context_arg)?;

    if command == Action::whoami.as_ref() {
        let (_, submatches) = matches.subcommand().unwrap();
        let code = if submatches.is_present(Parameters::token.as_ref()) {
            display_simple(Ok(openid::print_token(context)), json_output)?
        } else if let Some((_, endpoints_matches)) = submatches.subcommand() {
            let service = match endpoints_matches.value_of(Parameters::endpoints.as_ref()) {
                Some("*") => None,
                s => s,
            };
            let endpoints = util::get_drogue_endpoints_authenticated(context)
                .await
                .map(Outcome::SuccessWithJsonData);
            display(endpoints, json_output, |data| {
                util::endpoints_pretty_print(data, service)
            })?
        } else {
            openid::print_whoami(context, json_output)?
        };
        config.write(config_path)?;
        return Ok(code);
    }

    log::warn!("Using context: {}", context.name);
    let verb = Action::from_str(command);
    let cmd = submatches;

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
