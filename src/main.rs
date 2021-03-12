mod apps;
mod arguments;
mod devices;
mod util;

use arguments::{Parameters, Verbs, Resources};

use anyhow::Result;
use reqwest::Url;
use std::str::FromStr;

type AppId = str;
type DeviceId = str;

fn main() -> Result<()> {
    let matches = arguments::parse_arguments();

    if matches.is_present("version") {
        util::print_version();
    }

    let (cmd_name, cmd) = matches.subcommand();
    //deserialize the command into enum to take advantage of rust exhaustive match
    let verb = Verbs::from_str(cmd_name).unwrap();
    let (sub_cmd_name, sub_cmd) = cmd.unwrap().subcommand();
    let resource = Resources::from_str(sub_cmd_name).unwrap();
    let id = sub_cmd.unwrap().value_of(Parameters::id).unwrap();

    //TODO : The error is not nice to read.
    let url = Url::parse(matches.value_of(Parameters::url).unwrap()).expect("Invalid URL.");

    match verb {
        Verbs::create => {
            //TODO nicer panic message
            let data = util::json_parse(sub_cmd.unwrap().value_of(Parameters::data)).expect("Invalid Json in data args");
            match resource {
                Resources::app => apps::create(&url, id, data),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    devices::create(&url, id, data, app_id)
                },
            }
        }
        Verbs::delete => {
            match resource {
                Resources::app => apps::delete(&url, id),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    devices::delete(&url, app_id, id)
                },
            }
        }
        Verbs::edit => {
            match resource {
                Resources::app => apps::edit(&url, id),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    devices::edit(&url, app_id, id)
                },
            }
        }
        Verbs::get => {
            match resource {
                Resources::app => apps::read(&url, id),
                Resources::device => {
                    let app_id = sub_cmd.unwrap().value_of(Resources::app).unwrap();
                    devices::read(&url, app_id, id)
                },
            }
        }
    }

    Ok(())
}