mod admin;
mod apps;
mod arguments;
mod command;
mod config;
mod devices;
mod openid;
mod stream;
mod tokens;
mod trust;
mod util;

use arguments::{Action, Parameters, ResourceId, ResourceType};

use crate::arguments::Transfer;
use crate::config::{AccessToken, Config, Context, ContextId, Token};
use anyhow::{anyhow, Context as AnyhowContext, Result};
use json_value_merge::Merge;
use serde_json::json;
use std::process::exit;
use std::str::FromStr;

type AppId = String;
type DeviceId = String;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = arguments::app_arguments().get_matches();
    let config_path = matches.value_of(Parameters::config.as_ref());
    let (command, submatches) = matches.subcommand().unwrap();
    let context_arg = matches
        .value_of(ResourceId::contextId.as_ref())
        .map(|s| s.to_string());

    simple_logger::SimpleLogger::new()
        .with_level(util::log_level(&matches))
        .init()
        .unwrap();

    // load the config file
    let config_result: Result<Config> =
        Config::from(config_path).context("Error loading config file");

    if command == Action::login.as_ref() {
        let url = util::url_validation(submatches.value_of(Parameters::url.as_ref()).unwrap())?;

        let access_token_val = submatches.value_of(Parameters::access_token.as_ref());
        let mut config = config_result.unwrap_or_else(|_| Config::empty());
        let context = if let Some(access_token) = access_token_val {
            if let Some((id, token)) = access_token.split_once(':') {
                let (sso_url, registry_url) = util::get_drogue_services_endpoints(url.clone())?;
                let (auth_url, token_url) = util::get_auth_and_tokens_endpoints(sso_url)?;
                Ok(Context {
                    name: context_arg.unwrap_or("default".to_string() as ContextId),
                    drogue_cloud_url: url.clone(),
                    default_app: None,
                    default_algo: None,
                    token: Token::AccessToken(AccessToken {
                        id: id.to_string(),
                        token: token.to_string(),
                    }),
                    token_url,
                    auth_url,
                    registry_url,
                    token_exp_date: chrono::MAX_DATETIME,
                })
            } else {
                Err(anyhow!(
                    "Invalid access token. Format should be username:token"
                ))
            }
        } else {
            let refresh_token_val = submatches.value_of(Parameters::token.as_ref());
            openid::login(
                url.clone(),
                refresh_token_val,
                context_arg.unwrap_or("default".to_string() as ContextId),
            )
            .await
        }?;

        println!("\nSuccessfully authenticated to drogue cloud : {}", url);
        let name = context.name.clone();
        config.add_context(context)?;

        if !submatches.is_present(Parameters::keep_current.as_ref()) {
            config.set_active_context(name)?;
        }

        config.write(config_path)?;
        exit(0);
    } else if command == Action::version.as_ref() {
        util::print_version(&config_result);
        exit(0);
    }

    let mut config: Config = config_result?;

    if command == Action::config.as_ref() {
        let cmd = submatches;
        let (v, c) = cmd.subcommand().unwrap();

        let ctx_id = c
            .value_of(ResourceId::contextId.as_ref())
            .map(|s| s.to_string());

        match v {
            "create" => {
                println!("To create a new context use drg login");
            }
            "list" => {
                config.list_contexts();
            }
            "show" => {
                println!("{}", config);
            }
            "default-context" => {
                config.set_active_context(ctx_id.unwrap())?;
                config.write(config_path)?;
            }
            "delete" => {
                let id = ctx_id.unwrap();
                config.delete_context(&id)?;
                config.write(config_path)?;
            }
            "default-app" => {
                let id = c
                    .value_of(ResourceId::applicationId.as_ref())
                    .unwrap()
                    .to_string();
                let context = config.get_context_mut(&ctx_id)?;

                context.set_default_app(id);
                config.write(config_path)?;
            }
            "rename" => {
                let new_ctx = c.value_of("new_context_id").unwrap().to_string();

                config.rename_context(ctx_id.unwrap(), new_ctx)?;
                config.write(config_path)?;
            }
            "default-algo" => {
                let algo = c
                    .value_of(&Parameters::algo.as_ref())
                    .map(|a| trust::SignAlgo::from_str(a).unwrap())
                    .unwrap();
                let context = config.get_context_mut(&ctx_id)?;

                context.set_default_algo(algo);
                config.write(config_path)?;
            }
            _ => {
                println!("forgot to route config subcommand : {}", v);
            }
        }
        exit(0);
    }

    // The following commands needs a context and a valid token
    if openid::verify_token_validity(config.get_context_mut(&context_arg)?).await? {
        config.write(config_path)?;
    }
    let context = config.get_context(&context_arg)?;

    if command == Action::whoami.as_ref() {
        let (_, submatches) = matches.subcommand().unwrap();
        if submatches.is_present(Parameters::token.as_ref()) {
            openid::print_token(context);
        } else if let Some((_, endpoints_matches)) = submatches.subcommand() {
            let service = match endpoints_matches.value_of(Parameters::endpoints.as_ref()) {
                Some("*") => None,
                s => s,
            };
            util::print_endpoints(context, service)?;
        } else {
            openid::print_whoami(context);
            util::print_version(&Ok(config));
        }
        exit(0)
    }

    log::warn!("Using context: {}", context.name);
    let verb = Action::from_str(command);
    let cmd = submatches;

    match verb? {
        Action::create => {
            let (res, command) = cmd.subcommand().unwrap();
            let resource = ResourceType::from_str(res)?;

            match resource {
                ResourceType::application => {
                    let data = util::json_parse(command.value_of(Parameters::spec.as_ref()))?;
                    let file = command.value_of(Parameters::filename.as_ref());
                    let app_id = command
                        .value_of(ResourceId::applicationId.as_ref())
                        .map(|s| s.to_string());

                    apps::create(context, app_id, data, file).await
                }
                ResourceType::device => {
                    let app_id = arguments::get_app_id(command, context)?;
                    let mut data = util::json_parse(command.value_of(Parameters::spec.as_ref()))?;
                    let file = command.value_of(Parameters::filename.as_ref());
                    let dev_id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .map(|s| s.to_string());
                    let dev_id = devices::name_from_json_or_file(dev_id, file)?;

                    // add an alias with the correct subject dn.
                    if command.is_present(Parameters::cert.as_ref()) {
                        let alias = format!("CN={}, O=Drogue IoT, OU={}", &dev_id, app_id);
                        let alias_spec = json!([alias]);
                        data.merge_in("/alias", alias_spec)
                    }

                    devices::create(context, dev_id, data, app_id, file)
                }
                ResourceType::member => {
                    let app_id = arguments::get_app_id(command, context)?;
                    let role = command
                        .value_of(Parameters::role.as_ref())
                        .map(|r| admin::Roles::from_str(r).unwrap())
                        .unwrap();

                    let user = command.value_of(ResourceType::member.as_ref()).unwrap();

                    admin::member_add(context, &app_id, user, role)
                }
                ResourceType::token => {
                    let description = command.value_of(Parameters::description.as_ref());
                    tokens::create_api_key(context, description).await
                }
                //TODO verify appcert
                ResourceType::app_cert | ResourceType::device_cert => {
                    let app_id = arguments::get_app_id(command, context)?;
                    let days = command.value_of(&Parameters::days.as_ref());
                    let key_pair_algorithm = command
                        .value_of(&Parameters::algo.as_ref())
                        .or_else(|| {
                            context.default_algo.as_deref().map(|a| {
                                println!("Using default signature algorithm: {}", a);
                                a
                            })
                        })
                        .map(|algo| trust::SignAlgo::from_str(algo).unwrap());

                    let (key_input, key_pair_algorithm) = match command
                        .value_of(&Parameters::key_input.as_ref())
                    {
                        Some(f) => trust::verify_input_key(f).map(|s| (Some(s.0), Some(s.1)))?,
                        _ => (None, key_pair_algorithm),
                    };

                    let keyout = command.value_of(&Parameters::key_output.as_ref());

                    let ca_key = &command
                        .value_of(&Parameters::ca_key.as_ref())
                        .unwrap()
                        .to_string();

                    let device_cert = command.value_of(&Parameters::cert_output.as_ref());

                    let device_key = command.value_of(&Parameters::key_output.as_ref());

                    let cert = apps::get_trust_anchor(context, &app_id).await?;

                    if resource == ResourceType::app_cert {
                        apps::add_trust_anchor(
                            context,
                            &app_id,
                            keyout,
                            key_pair_algorithm,
                            days,
                            key_input,
                        )
                        .await
                    } else {
                        // Safe unwrap because clap makes sure the argument is provided
                        let dev_id = command.value_of(ResourceId::deviceId.as_ref()).unwrap();

                        trust::create_device_certificate(
                            &app_id,
                            dev_id,
                            ca_key,
                            &cert,
                            device_key,
                            device_cert,
                            key_pair_algorithm,
                            days,
                            key_input,
                        )
                        .and_then(|_| {
                            let alias = format!("CN={}, O=Drogue IoT, OU={}", dev_id, app_id);
                            devices::add_alias(
                                context,
                                app_id.to_string(),
                                dev_id.to_string(),
                                alias,
                            )
                        })
                    }
                }
                // The other enum variants are not exposed by clap
                _ => unreachable!(),
            }?;
        }
        Action::delete => {
            let (res, command) = cmd.subcommand().unwrap();
            let resource = ResourceType::from_str(res);

            let ignore_missing = command.is_present(Parameters::ignore_missing.as_ref());

            match resource? {
                ResourceType::application => {
                    let id = command
                        .value_of(ResourceId::applicationId.as_ref())
                        .unwrap()
                        .to_string();
                    apps::delete(context, id, ignore_missing).await
                }
                ResourceType::device => {
                    let app_id = arguments::get_app_id(command, context)?;
                    let id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .unwrap()
                        .to_string();
                    devices::delete(context, app_id, id, ignore_missing)
                }
                ResourceType::member => {
                    let app_id = arguments::get_app_id(command, context)?;
                    let user = command.value_of(ResourceType::member.as_ref()).unwrap();

                    admin::member_delete(context, app_id.as_str(), user)
                }
                ResourceType::token => {
                    let prefix = command.value_of(ResourceId::tokenPrefix.as_ref()).unwrap();
                    tokens::delete_api_key(context, prefix).await
                }
                // The other enum variants are not exposed by clap
                _ => unreachable!(),
            }?;
        }
        Action::edit => {
            let (res, command) = cmd.subcommand().unwrap();
            let resource = ResourceType::from_str(res);

            match resource? {
                ResourceType::application => {
                    let file = command.value_of(Parameters::filename.as_ref());
                    let id = command
                        .value_of(ResourceId::applicationId.as_ref())
                        .map(|s| s.to_string())
                        .unwrap();
                    apps::edit(context, id, file).await
                }
                ResourceType::device => {
                    let dev_id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .map(|s| s.to_string());
                    let file = command.value_of(Parameters::filename.as_ref());
                    let app_id = arguments::get_app_id(command, context)?;

                    devices::edit(context, app_id, dev_id, file)
                }
                ResourceType::member => {
                    let app_id = arguments::get_app_id(command, context)?;
                    admin::member_edit(context, &app_id)
                }
                // The other enum variants are not exposed by clap
                _ => unreachable!(),
            }?;
        }
        Action::get => {
            let (res, command) = cmd.subcommand().unwrap();
            let resource = ResourceType::from_str(res)?;

            match resource {
                ResourceType::application => {
                    let app_id = command
                        .value_of(ResourceId::applicationId.as_ref())
                        .map(|s| s.to_string());
                    let labels = command.values_of(Parameters::labels.as_ref());

                    match app_id {
                        Some(id) => apps::read(context, id as AppId).await,
                        None => apps::list(context, labels).await,
                    }?;
                }
                ResourceType::device => {
                    let wide = arguments::get_wide(command);
                    let app_id = arguments::get_app_id(command, context)?;
                    let labels = command
                        .values_of(Parameters::labels.as_ref())
                        .map(|v| v.collect::<Vec<&str>>().join(","));
                    let dev_id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .map(|s| s.to_string());
                    match dev_id {
                        Some(id) => devices::read(context, app_id, id as DeviceId),
                        None => devices::list(context, app_id, labels, wide),
                    }?;
                }
                ResourceType::member => {
                    let app_id = arguments::get_app_id(command, context)?;
                    admin::member_list(context, &app_id)?;
                }
                ResourceType::token => {
                    tokens::get_api_keys(context).await?;
                }
                // The other enum variants are not exposed by clap
                _ => unreachable!(),
            }
        }
        Action::set => {
            let (target, command) = cmd.subcommand().unwrap();
            let app_id = arguments::get_app_id(command, context)?;

            match ResourceType::from_str(target)? {
                ResourceType::gateway => {
                    let id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .unwrap()
                        .to_string();
                    let gateway_id = command
                        .value_of(ResourceId::gatewayId.as_ref())
                        .unwrap()
                        .to_string();
                    devices::set_gateway(context, app_id, id as DeviceId, gateway_id)?;
                }
                ResourceType::password => {
                    let id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .unwrap()
                        .to_string();
                    let password = command
                        .value_of(Parameters::password.as_ref())
                        .unwrap()
                        .to_string();
                    let username = command.value_of(ResourceId::username.as_ref());
                    devices::set_password(context, app_id, id as DeviceId, password, username)?;
                }
                ResourceType::alias => {
                    let id = command
                        .value_of(ResourceId::deviceId.as_ref())
                        .unwrap()
                        .to_string();
                    let alias = command
                        .value_of(Parameters::alias.as_ref())
                        .unwrap()
                        .to_string();
                    devices::add_alias(context, app_id, id as DeviceId, alias)?;
                }
                ResourceType::label => {
                    let labels = command.values_of(ResourceType::label.as_ref()).unwrap();

                    match command.value_of("dev-flag") {
                        Some(dev_id) => {
                            devices::add_labels(context, app_id, dev_id.to_string(), labels)
                        }
                        None => apps::add_labels(context, app_id, &labels).await,
                    }?;
                }
                // The other enum variants are not exposed by clap
                _ => unreachable!(),
            }
        }
        Action::command => {
            let command = cmd.value_of(Parameters::command.as_ref()).unwrap();
            let app_id = arguments::get_app_id(cmd, context)?;
            let device = cmd.value_of(ResourceId::deviceId.as_ref()).unwrap();

            let body = match cmd.value_of(Parameters::filename.as_ref()) {
                Some(f) => util::get_data_from_file(f)?,
                None => util::json_parse(cmd.value_of(Parameters::payload.as_ref()))?,
            };

            command::send_command(context, app_id.as_str(), device, command, body)?;
        }
        Action::transfer => {
            let task = Transfer::from_str(command);

            match task? {
                Transfer::init => {
                    let user = cmd.value_of(Parameters::username.as_ref()).unwrap();
                    let id = arguments::get_app_id(cmd, context)?;
                    admin::transfer_app(context, id.as_str(), user)?;
                }
                Transfer::accept => {
                    let id = cmd.value_of(ResourceId::applicationId.as_ref()).unwrap();
                    admin::accept_transfer(context, id)?
                }
                Transfer::cancel => {
                    let id = cmd.value_of(ResourceId::applicationId.as_ref()).unwrap();
                    admin::cancel_transfer(context, id)?
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

            stream::stream_app(context, &app_id, device, count)?;
            exit(0)
        }
        // todo implement the other Actions variants?
        _ => unimplemented!(),
    }

    Ok(())
}
