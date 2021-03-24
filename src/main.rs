mod apps;
mod arguments;
mod config;
mod devices;
mod util;
mod openid;

use arguments::{Parameters, Resources, Verbs};

use anyhow::{Context, Result};
use std::str::FromStr;

type AppId = str;
type DeviceId = str;

fn main() -> Result<()> {
    let matches = arguments::parse_arguments();

    if matches.is_present("version") {
        util::print_version();
    }

    let url;
    //todo default app is not used
    let _default_app: Option<String>;
    // url arg preempts config file.
    if matches.is_present(Parameters::url) {
        url = util::url_validation(matches.value_of(Parameters::url))?;
        _default_app = None;
    } else {
        let conf = config::load_config_file(matches.value_of(Parameters::config))
            .context("No URL arg provided and DRGCTL config file was not found.")?;
        url = util::url_validation(Some(conf.drogue_cloud_url.as_str()))?;
        _default_app = conf.default_app;
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
                            Resources::app => apps::create(&url, id, data)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::create(&url, id, data, app_id)?
                            }
                        }
                    }
                },
                Verbs::delete => match cmd.subcommand() {
                    (res, command) => {
                        let id = command.unwrap().value_of(Parameters::id).unwrap();
                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::delete(&url, id)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::delete(&url, app_id, id)?
                            }
                        }
                    }
                },
                Verbs::edit => match cmd.subcommand() {
                    (res, command) => {
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::edit(&url, id),
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::edit(&url, app_id, id)
                            }
                        }
                    }
                },
                Verbs::get => match cmd.subcommand() {
                    (res, command) => {
                        let id = command.unwrap().value_of(Parameters::id).unwrap();

                        let resource = Resources::from_str(res);

                        match resource? {
                            Resources::app => apps::read(&url, id)?,
                            Resources::device => {
                                let app_id = command.unwrap().value_of(Resources::app).unwrap();
                                devices::read(&url, app_id, id)?
                            }
                        }
                    }
                },
            }
        }
    }

    Ok(())
}
