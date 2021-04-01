use crate::util;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use std::convert::AsRef;
use strum_macros::{AsRefStr, EnumString};

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Verbs {
    create,
    delete,
    edit,
    get,
    
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Resources {
    device,
    app,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Parameters {
    url,
    id,
    data,
    config,
    app_id,
    device_id,
    command,
    payload,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Other_commands {
    login,
    token,
    version,
    sendcommand,
}

pub fn parse_arguments() -> ArgMatches<'static> {
    let resource_id_arg = Arg::with_name(Parameters::id.as_ref())
        .required(true)
        .help("The unique id of the resource.");
        
    let _app_id_arg = Arg::with_name(Parameters::app_id.as_ref())
        .required(true)
        .help("The app id");
        
    let device_id_arg = Arg::with_name(Parameters::device_id.as_ref())
        .required(true)
        .help("The device id");
        
    let command_arg = Arg::with_name(Parameters::command.as_ref())
        .required(true)
        .help("The command");
        
    let payload_arg = Arg::with_name(Parameters::payload.as_ref())
        .required(true)
        .help("The payload");

    let url_arg = Arg::with_name(Parameters::url.as_ref())
        .required(true)
        .help("The url of the drogue cloud api endpoint");

    let app_id_arg = Arg::with_name(Resources::app.as_ref())
        .required(true)
        .short("a")
        .long(Resources::app.as_ref())
        .takes_value(true)
        .help("The app owning the device.");

    let data_arg = Arg::with_name(Parameters::data.as_ref())
        .short("d")
        .long(Parameters::data.as_ref())
        .takes_value(true)
        .help("The data for the resource.");

    let config_file_arg = Arg::with_name(Parameters::config.as_ref())
        .long(Parameters::config.as_ref())
        .takes_value(true)
        .conflicts_with(Parameters::url.as_ref())
        .help("Path to the drgconfig file. If not specified, reads $DRGCFG environment variable or defaults to XDG config directory for drg_config.json");

    let verbose = Arg::with_name("verbose")
        .short("v")
        .takes_value(false)
        .multiple(true)
        .global(true)
        .help("Enable verbose output. Multiple occurences increase verbosity.");

    App::new("Drogue Command Line Tool")
        .version(util::VERSION)
        .author("Jb Trystram <jbtrystram@redhat.com>")
        .about("Allows to manage drogue apps and devices in a drogue-cloud instance")
        .arg(config_file_arg)
        .arg(verbose)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name(Verbs::create.as_ref())
                .alias("add")
                .about("create a resource in the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("create a device.")
                        .arg(resource_id_arg.clone())
                        .arg(app_id_arg.clone())
                        .arg(data_arg.clone()),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("create an app.")
                        .arg(resource_id_arg.clone())
                        .arg(data_arg.clone()),
                ),
        )
        .subcommand(
            SubCommand::with_name(Verbs::delete.as_ref())
                .alias("remove")
                .about("delete a resource in the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("delete a device.")
                        .arg(resource_id_arg.clone())
                        .arg(app_id_arg.clone()),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("create an app.")
                        .arg(resource_id_arg.clone()),
                ),
        )
        .subcommand(
            SubCommand::with_name(Verbs::get.as_ref())
                .about("Read a resource from the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("Retrieve a device data.")
                        .arg(resource_id_arg.clone())
                        .arg(app_id_arg.clone()),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("retrieve an app data.")
                        .arg(resource_id_arg.clone()),
                ),
        )
        .subcommand(
            SubCommand::with_name(Verbs::edit.as_ref())
                .about("Edit a resource from the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("Edit a device data.")
                        .arg(resource_id_arg.clone())
                        .arg(app_id_arg.clone()),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("Edit an app data.")
                        .arg(resource_id_arg.clone()),
                ),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::sendcommand.as_ref())
                .about("send command to the drogue-cloud url")
                .arg(_app_id_arg.clone())
                .arg(device_id_arg.clone())
                .arg(command_arg.clone())
                .arg(payload_arg.clone())
        )
        .subcommand(
            SubCommand::with_name(Other_commands::version.as_ref())
                .about("Print version information."),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::login.as_ref())
                .about("Log into a drogue cloud installation.")
                .arg(url_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::token.as_ref())
                .about("Print a valid bearer token for the drogue cloud instance."),
        )
        .get_matches()
}
