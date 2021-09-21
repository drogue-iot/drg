mod apps;
mod arguments;
mod command;
mod config;
mod devices;
mod openid;
mod stream;
mod trust;
mod util;

use arguments::{
    Context_subcommands, Other_commands, Other_flags, Parameters, Resources, Set_args, Set_targets,
    Trust_subcommands, Verbs,
};

use crate::config::{Config, ContextId};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use json_value_merge::Merge;
use serde_json::json;
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
                println!("{}", config);
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
            Context_subcommands::set_default_algo => {
                let algo = c
                    .unwrap()
                    .value_of(&Parameters::algo)
                    .map(|a| trust::SignAlgo::from_str(a).unwrap())
                    .unwrap();
                let context = config.get_context_mut(&ctx_id)?;

                context.set_default_algo(algo);
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

    if command == Other_commands::whoami.as_ref() {
        let (_, submatches) = matches.subcommand();
        let (_, endpoints_matches) = submatches.map(|s| s.subcommand()).unwrap_or(("", None));
        if submatches.unwrap().is_present(Other_commands::token) {
            openid::print_token(&context);
        } else if let Some(endpoints_matches) = endpoints_matches {
            let service = match endpoints_matches.value_of(Other_commands::endpoints) {
                Some("*") => None,
                s => s,
            };
            util::print_endpoints(&context, service)?;
        } else {
            openid::print_whoami(&context);
            util::print_version(&Ok(config));
        }
        exit(0)
    }

    if command == Other_commands::stream.as_ref() {
        let (_, matches) = matches.subcommand();
        let app_id = arguments::get_app_id(&matches.unwrap(), &context)?;

        stream::stream_app(&context, &app_id)?;
        exit(0)
    }

    if command == Other_commands::trust.as_ref() {
        let (v, command) = submatches.unwrap().subcommand();
        let verb = Trust_subcommands::from_str(v);
        let id = command
            .unwrap()
            .value_of(Parameters::id)
            .map(|s| s.to_string());
        let days = command.unwrap().value_of(&Parameters::days);
        let key_pair_algorithm = command
            .unwrap()
            .value_of(&Parameters::algo)
            .or_else(|| {
                context.default_algo.as_deref().map(|a| {
                    println!("Using default signature algorithm: {}", a);
                    a
                })
            })
            .map(|algo| trust::SignAlgo::from_str(algo).unwrap());

        let (key_input, key_pair_algorithm) =
            match command.unwrap().value_of(&Parameters::key_input) {
                Some(f) => trust::verify_input_key(f).map(|s| (Some(s.0), Some(s.1)))?,
                _ => (None, key_pair_algorithm),
            };

        match verb? {
            Trust_subcommands::create => {
                let keyout = command.unwrap().value_of(&Parameters::key_output);
                let app_id = id.unwrap_or_else(|| {
                    arguments::get_app_id(&command.unwrap(), &context)
                        .map_err(|e| {
                            log::error!("{}", e);
                            exit(1)
                        })
                        .unwrap()
                });

                apps::add_trust_anchor(
                    &context,
                    &app_id,
                    keyout,
                    key_pair_algorithm,
                    days,
                    key_input,
                )
            }
            Trust_subcommands::enroll => {
                let app_id = arguments::get_app_id(&command.unwrap(), &context)?;
                let device_id = &id.unwrap();

                let ca_key = &command
                    .unwrap()
                    .value_of(&Parameters::ca_key)
                    .unwrap()
                    .to_string();

                let device_cert = command.unwrap().value_of(&Parameters::out);

                let device_key = command.unwrap().value_of(&Parameters::key_output);

                let cert = apps::get_trust_anchor(&context, &app_id)?;

                trust::create_device_certificate(
                    &app_id,
                    &device_id,
                    ca_key,
                    &cert,
                    device_key,
                    device_cert,
                    key_pair_algorithm,
                    days,
                    key_input,
                )
                .and_then(|_| {
                    let alias = format!("CN={}, O=Drogue IoT, OU={}", device_id, app_id);
                    devices::add_alias(&context, app_id, device_id.to_string(), alias)
                })
            }
        }?;
        exit(0);
    }

    log::warn!("Using context: {}", context.name);
    let verb = Verbs::from_str(command);
    let cmd = submatches.unwrap();

    match verb? {
        Verbs::create => {
            let (res, command) = cmd.subcommand();
            let mut data = util::json_parse(command.unwrap().value_of(Parameters::spec))?;
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

                    // add an alias with the correct subject dn.
                    if command.unwrap().is_present(&Other_flags::cert) {
                        let alias = format!("CN={}, O=Drogue IoT, OU={}", id, app_id);
                        let alias_spec = json!([alias]);
                        data.merge_in("/alias", alias_spec)
                    }

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
            let ignore_missing = command.unwrap().is_present(Other_flags::ignore_missing);

            match resource? {
                Resources::app => apps::delete(&context, id, ignore_missing),
                Resources::device => {
                    let app_id = arguments::get_app_id(&command.unwrap(), &context)?;
                    devices::delete(&context, app_id, id, ignore_missing)
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
        Verbs::set => {
            let (target, command) = cmd.subcommand();
            let args: Vec<&str> = command.unwrap().values_of(Verbs::set).unwrap().collect();

            // clap already makes sure vals contains two values
            let (device, value) = (args[0].to_string(), args[1].to_string());
            let app_id = arguments::get_app_id(&command.unwrap(), &context)?;

            match Set_targets::from_str(target)? {
                Set_targets::gateway => {
                    devices::set_gateway(&context, app_id, device as DeviceId, value)?;
                }
                Set_targets::password => {
                    let username = command.unwrap().value_of(Set_args::username);
                    devices::set_password(&context, app_id, device as DeviceId, value, username)?;
                }
                Set_targets::alias => {
                    devices::add_alias(&context, app_id, device as DeviceId, value)?;
                }
            }
        }
        Verbs::cmd => {
            let args: Vec<&str> = cmd.values_of(Verbs::cmd).unwrap().collect();
            let app_id = arguments::get_app_id(&cmd, &context)?;
            let (command, device) = (args[0], args[1]);

            let body = match cmd.value_of(Parameters::filename) {
                Some(f) => util::get_data_from_file(f)?,
                None => util::json_parse(cmd.value_of(Parameters::payload))?,
            };

            command::send_command(&context, app_id.as_str(), device, command, body)?;
        }
    }

    Ok(())
}
