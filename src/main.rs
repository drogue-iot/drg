mod apps;
mod arguments;
mod config;
mod devices;
mod openid;
mod util;

extern crate reqwest;

use arguments::{Other_commands, Parameters, Resources, Verbs};

use anyhow::{Context, Result};
use std::process::exit;
use std::str::FromStr;
type AppId = str;
type DeviceId = str;

fn main() -> Result<()> {
    let matches = arguments::parse_arguments();
    let mut config;
    simple_logger::SimpleLogger::new()
        .with_level(util::log_level(&matches))
        .init()
        .unwrap();

    if matches.is_present(Other_commands::login) {
        let (_, submatches) = matches.subcommand();
        let url = util::url_validation(submatches.unwrap().value_of(Parameters::url).unwrap())?;

        config = openid::login(url.clone())?;

        println!("\nSuccessfully authenticated to drogue cloud : {}", url);
        config::save_config(&config)?;
        exit(0);
    }
    
    // load the config file
    let rst_config = config::load_config(matches.value_of(Parameters::config));
    
    if matches.is_present(Other_commands::version) {
        util::print_version(&rst_config);
    }
    
    config = rst_config.context(
        "Error opening the configuration file. Did you log into a drogue cloud cluster ?",
    )?;
    
    config = openid::verify_token_validity(config)?;
    
    if matches.is_present(Other_commands::sendcommand) {
        let (_, submatches) = matches.subcommand();
        let mut url =config.drogue_cloud_url.to_string();
        let appid = submatches.unwrap().value_of(Parameters::app_id).unwrap();
        let device_id = submatches.unwrap().value_of(Parameters::device_id).unwrap();
        let command = submatches.unwrap().value_of(Parameters::command).unwrap();
        let payload = submatches.unwrap().value_of(Parameters::payload).unwrap();
        url = url  +"/command?application=" + &appid + "/&device_id=" + &device_id + "/&command=" + &command + "/&payload=" + &payload;
        println!("Final Url for the get request {}",  url);
        let response_text = reqwest::get(&url)
        .expect("Could not send the command")
        .text().expect("Could not read the response text");
        println!("Response Text : {}", response_text);
        exit(0);
    } 
    
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
                            Resources::app => apps::create(&config, id, data)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::create(&config, id, data, app_id)
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
                        let id = command.unwrap().value_of(Parameters::id).unwrap();
                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::delete(&config, id)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::delete(&config, app_id, id)
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
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::edit(&config, id)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::edit(&config, app_id, id)
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
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::read(&config, id)
                                .map_err(|e| {
                                    log::error!("{:?}", e);
                                    exit(3)
                                })
                                .unwrap(),
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::read(&config, app_id, id)
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
