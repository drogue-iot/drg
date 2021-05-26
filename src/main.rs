mod apps;
mod arguments;
mod config;
mod devices;
mod openid;
mod util;

use arguments::{Context_subcommands, Other_commands, Parameters, Resources, Verbs};

use crate::config::{Config, ContextId};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use std::process::exit;
use std::str::FromStr;

type AppId = String;
type DeviceId = String;

fn main() -> Result<()> {
    let matches = arguments::parse_arguments();
    let config_path = matches.value_of(Parameters::config);
    let (command, submatches) = matches.subcommand();
    let context_arg = matches.value_of(Parameters::context).map(|s| s.to_string());

    simple_logger::SimpleLogger::new()
        .with_level(util::log_level(&matches))
        .init()
        .unwrap();

    // load the config file
    let config_result: Result<Config> =
        Config::from(config_path).context("Error loading config file");

    if command == Other_commands::login.as_ref() {
        let url = util::url_validation(submatches.unwrap().value_of(Parameters::url).unwrap())?;
        let refresh_token_val = submatches.unwrap().value_of(Other_commands::token);

        let mut config = config_result.unwrap_or_else(|_| Config::empty());
        let context = openid::login(
            url.clone(),
            refresh_token_val,
            context_arg.unwrap_or("default".to_string() as ContextId),
        )?;

        println!("\nSuccessfully authenticated to drogue cloud : {}", url);
        let name = context.name.clone();
        config.add_context(context)?;

        if !submatches.unwrap().is_present(Parameters::keep_current) {
            config.set_active_context(name)?;
        }

        config.write(config_path)?;
        exit(0);
    } else if command == Other_commands::version.as_ref() {
        util::print_version(&config_result);
        exit(0);
    }

    let mut config: Config = config_result?;

    if command == Other_commands::context.as_ref() {
        let cmd = submatches.unwrap();
        let (v, c) = cmd.subcommand();
        let verb = Context_subcommands::from_str(v);

        let ctx_id = c
            .unwrap()
            .value_of(Parameters::context_name)
            .map(|s| s.to_string());

        match verb? {
            Context_subcommands::create => {
                println!("To create a new context use drg login");
            }
            Context_subcommands::list => {
                config.list_contexts();
            }
            Context_subcommands::show => {
                config.show()?;
            }
            Context_subcommands::set_active => {
                config.set_active_context(ctx_id.unwrap())?;
                config.write(config_path)?;
            }
            Context_subcommands::delete => {
                let id = ctx_id.unwrap();
                config.delete_context(&id)?;
                config.write(config_path)?;
            }
            Context_subcommands::set_default_app => {
                let id = c.unwrap().value_of(Parameters::id).unwrap().to_string();
                let context = config.get_context_mut(&ctx_id)?;

                context.set_default_app(id);
                config.write(config_path)?;
            }
            Context_subcommands::rename => {
                let new_ctx = c.unwrap().value_of("new_context_id").unwrap().to_string();

                config.rename_context(ctx_id.unwrap(), new_ctx)?;
                config.write(config_path)?;
            }
        }
        exit(0);
    }

    // The following commands needs a context and a valid token
    if openid::verify_token_validity(config.get_context_mut(&context_arg)?)? {
        config.write(config_path)?;
    }
    let context = config.get_context(&context_arg)?;

    if command == Other_commands::token.as_ref() {
        openid::print_token(&context);
        exit(0);
    } else if command == Other_commands::whoami.as_ref() {
        let (_, submatches) = matches.subcommand();
        if submatches.unwrap().is_present("token") {
            openid::print_token(&context);
        } else {
            openid::print_whoami(&context);
            util::print_version(&Ok(config));
        }
        exit(0)
    }

    log::warn!("Using context: {}", context.name);
    let verb = Verbs::from_str(command);
    let cmd = submatches.unwrap();

    match verb? {
        Verbs::create => {
            let (res, command) = cmd.subcommand();
            let data = util::json_parse(command.unwrap().value_of(Parameters::spec))?;
            let id = command
                .unwrap()
                .value_of(Parameters::id)
                .unwrap()
                .to_string();

            let resource = Resources::from_str(res);
            let file = command.unwrap().value_of(Parameters::filename);

            match resource? {
                Resources::app => apps::create(&context, id, data, file),
                Resources::device => {
                    let app_id = arguments::get_app_id(&command.unwrap(), &context)?;
                    devices::create(&context, id, data, app_id, file)
                }
                // ignore apps and devices keywords
                _ => Err(anyhow!("Cannot create multiple resources")),
            }?;
        }
        Verbs::delete => {
            let (res, command) = cmd.subcommand();
            let id = command
                .unwrap()
                .value_of(Parameters::id)
                .unwrap()
                .to_string();
            let resource = Resources::from_str(res);

            match resource? {
                Resources::app => apps::delete(&context, id),
                Resources::device => {
                    let app_id = arguments::get_app_id(&command.unwrap(), &context)?;
                    devices::delete(&context, app_id, id)
                }
                // ignore apps and devices keywords
                _ => Err(anyhow!("Cannot delete multiple resources")),
            }?;
        }
        Verbs::edit => {
            let (res, command) = cmd.subcommand();
            let id = command
                .unwrap()
                .value_of(Parameters::id)
                .unwrap()
                .to_string();
            let file = command.unwrap().value_of(Parameters::filename);
            let resource = Resources::from_str(res);

            match resource? {
                Resources::app => apps::edit(&context, id, file),
                Resources::device => {
                    let app_id = arguments::get_app_id(&command.unwrap(), &context)?;
                    devices::edit(&context, app_id, id, file)
                }
                // ignore apps and devices keywords
                _ => Err(anyhow!("Cannot edit multiple resources")),
            }?;
        }
        Verbs::get => {
            let (res, command) = cmd.subcommand();

            let resource = Resources::from_str(res)?;

            let id = command
                .unwrap()
                .value_of(Parameters::id)
                .map(|s| s.to_string());

            let labels = command
                .unwrap()
                .values_of(Parameters::labels)
                .map(|v| v.collect::<Vec<&str>>().join(","));

            match resource {
                Resources::app | Resources::apps => {
                    match id {
                        Some(id) => apps::read(&context, id as AppId),
                        None => apps::list(&context, labels),
                    }?;
                }
                Resources::device | Resources::devices => {
                    let app_id = arguments::get_app_id(&command.unwrap(), &context)?;
                    match id {
                        Some(id) => devices::read(&context, app_id, id as DeviceId),
                        None => devices::list(&context, app_id, labels),
                    }?;
                }
            }
        }
    }

    Ok(())
}
