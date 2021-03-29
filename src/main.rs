mod apps;
mod arguments;
mod config;
mod devices;
mod openid;
mod util;

use arguments::{Parameters, Resources, Verbs, Other_commands};

use anyhow::{Context, Result};
use std::process::exit;
use std::str::FromStr;

type AppId = str;
type DeviceId = str;

fn main() -> Result<()> {
    let matches = arguments::parse_arguments();
    let mut config;

    if matches.is_present(Other_commands::version) {
        util::print_version();
    } else if matches.is_present(Other_commands::login) {
        let (_, submatches) = matches.subcommand();
        let url = util::url_validation(submatches.unwrap().value_of(Parameters::url).unwrap())?;

        config = openid::login(url.clone())?;

        println!("\nSuccessfully authenticated to drogue cloud : {}", url);
        config::save_config(&config)?;
        exit(0);
    }

    // try to load the config file
    config = config::load_config(matches.value_of(Parameters::config)).context(
        "Error opening the configuration file. Did you log into a drogue cloud cluster ?",
    )?;

    config = openid::verify_token_validity(config)?;

    if matches.is_present(Other_commands::token) {
        openid::print_token(&config);
        exit(0);
    }

    match matches.subcommand() {
        (cmd_name, sub_cmd) => {
            let verb = Verbs::from_str(cmd_name);
            let cmd = sub_cmd.unwrap();

            match verb? {
                Verbs::create => match cmd.subcommand() {
                    (res, command) => {
                        let data = util::json_parse(command.unwrap().value_of(Parameters::data))?;
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::create(&config, id, data)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::create(&config, id, data, app_id)?
                            }
                        }
                    }
                },
                Verbs::delete => match cmd.subcommand() {
                    (res, command) => {
                        let id = command.unwrap().value_of(Parameters::id).unwrap();
                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::delete(&config, id)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::delete(&config, app_id, id)?
                            }
                        }
                    }
                },
                Verbs::edit => match cmd.subcommand() {
                    (res, command) => {
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::edit(&config, id)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::edit(&config, app_id, id)?
                            }
                        }
                    }
                },
                Verbs::get => match cmd.subcommand() {
                    (res, command) => {
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::read(&config, id)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::read(&config, app_id, id)?
                            }
                        }
                    }
                },
            }
        }
    }

    Ok(())
}
