mod apps;
mod arguments;
mod config;
mod devices;
mod openid;
mod util;

use arguments::{Context_subcommands, Other_commands, Parameters, Resources, Verbs};

use crate::config::Config;
use anyhow::{Context as AnyhowContext, Result};
use std::process::exit;
use std::str::FromStr;

type AppId = String;
type DeviceId = String;

fn main() -> Result<()> {
    let matches = arguments::parse_arguments();
    let config_path = matches.value_of(Parameters::config);
    let context_arg = matches.value_of(Parameters::context).map(|s| s.to_string());

    simple_logger::SimpleLogger::new()
        .with_level(util::log_level(&matches))
        .init()
        .unwrap();

    // load the config file
    let config_result: Result<Config> =
        Config::from(config_path).context("Error loading config file");

    if matches.is_present(Other_commands::login) {
        let (_, submatches) = matches.subcommand();
        let url = util::url_validation(submatches.unwrap().value_of(Parameters::url).unwrap())?;

        let refresh_token_val = submatches.unwrap().value_of(Other_commands::token);

        let mut config = config_result.unwrap_or(Config::empty());
        let context = openid::login(url.clone(), refresh_token_val)?;

        println!("\nSuccessfully authenticated to drogue cloud : {}", url);
        config.add_context(context)?;
        config.write(config_path)?;
        exit(0);
    }

    if matches.is_present(Other_commands::version) {
        util::print_version(&config_result);
        exit(0);
    }

    let mut config: Config = config_result?;

    if matches.is_present(Other_commands::token) {
        let context = config.get_context(&context_arg)?;
        openid::print_token(&context);
        exit(0);
    }

    if matches.is_present(Other_commands::context) {
        match matches.subcommand() {
            (_cmd_name, sub_cmd) => {
                let cmd = sub_cmd.unwrap();
                let (v, c) = cmd.subcommand();
                let verb = Context_subcommands::from_str(v);

                match verb? {
                    Context_subcommands::create => {
                        println!("To create a new context use drg login");
                        exit(1);
                    }
                    Context_subcommands::list => {
                        config.list_contexts();
                        exit(0);
                    }
                    Context_subcommands::show => {
                        config.show()?;
                        exit(0);
                    }
                    Context_subcommands::set_active => {
                        let id = c
                            .unwrap()
                            .value_of(Parameters::context_id)
                            .unwrap()
                            .to_string();
                        config.set_active_context(id)?;
                        config.write(config_path)?;
                        exit(0);
                    }
                    Context_subcommands::delete => {
                        let id = c
                            .unwrap()
                            .value_of(Parameters::context_id)
                            .unwrap()
                            .to_string();
                        config.delete_context(&id)?;
                        config.write(config_path)?;
                        exit(0);
                    }
                    Context_subcommands::set_default_app => {
                        let id = c.unwrap().value_of(Parameters::id).unwrap().to_string();
                        let ctx_name = matches.value_of(Parameters::context).map(|s| s.to_string());

                        let context = config.get_context_mut(&ctx_name)?;
                        context.set_default_app(id);
                        config.write(config_path)?;
                        exit(0);
                    }
                    Context_subcommands::rename => {
                        let ctx = c
                            .unwrap()
                            .value_of(Parameters::context_id)
                            .unwrap()
                            .to_string();
                        let new_ctx = c.unwrap().value_of("new_context_id").unwrap().to_string();

                        config.rename_context(ctx, new_ctx)?;
                        config.write(config_path)?;
                        exit(0);
                    }
                }
            }
        }
    }

    if openid::verify_token_validity(config.get_context_mut(&context_arg)?)? {
        config.write(config_path)?;
    }

    let context = config.get_context(&context_arg)?;
    match matches.subcommand() {
        (cmd_name, sub_cmd) => {
            let verb = Verbs::from_str(cmd_name);
            let cmd = sub_cmd.unwrap();

            match verb? {
                Verbs::create => match cmd.subcommand() {
                    (res, command) => {
                        let data = util::json_parse(command.unwrap().value_of(Parameters::spec))?;
                        let id = command
                            .unwrap()
                            .value_of(Parameters::id)
                            .unwrap()
                            .to_string();

                        let resource = Resources::from_str(res);
                        let file = command.unwrap().value_of(Parameters::filename);

                        match resource? {
                            Resources::app => apps::create(&context, id, data, file)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id =
                                    arguments::get_app_id(&command.unwrap(), &context)?.to_string();
                                devices::create(&context, id, data, app_id, file)
                                    .map_err(|e| {
                                        log::error!("{:?}", e);
                                        exit(3)
                                    })
                                    .unwrap();
                            }
                        }
                    }
                },
                Verbs::delete => match cmd.subcommand() {
                    (res, command) => {
                        let id = command
                            .unwrap()
                            .value_of(Parameters::id)
                            .unwrap()
                            .to_string();
                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::delete(&context, id)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id =
                                    arguments::get_app_id(&command.unwrap(), &context)?.to_string();
                                devices::delete(&context, app_id, id)
                                    .map_err(|e| {
                                        log::error!("{:?}", e);
                                        exit(3)
                                    })
                                    .unwrap()
                            }
                        }
                    }
                },
                Verbs::edit => match cmd.subcommand() {
                    (res, command) => {
                        let id = command
                            .unwrap()
                            .value_of(Parameters::id)
                            .unwrap()
                            .to_string();
                        let file = command.unwrap().value_of(Parameters::filename);
                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::edit(&context, id, file)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id =
                                    arguments::get_app_id(&command.unwrap(), &context)?.to_string();
                                devices::edit(&context, app_id, id, file)
                                    .map_err(|e| {
                                        log::error!("{:?}", e);
                                        exit(3)
                                    })
                                    .unwrap()
                            }
                        }
                    }
                },
                Verbs::get => match cmd.subcommand() {
                    (res, command) => {
                        let id = command
                            .unwrap()
                            .value_of(Parameters::id)
                            .unwrap()
                            .to_string();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::read(&context, id)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id =
                                    arguments::get_app_id(&command.unwrap(), &context)?.to_string();
                                devices::read(&context, app_id, id)
                                    .map_err(|e| {
                                        log::error!("{:?}", e);
                                        exit(3)
                                    })
                                    .unwrap()
                            }
                        }
                    }
                },
            }
        }
    }

    Ok(())
}
