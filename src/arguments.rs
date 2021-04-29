use crate::{util, AppId};

use crate::config::Context;
use anyhow::{anyhow, Result};
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
    spec,
    config,
    filename,
    context,
    context_id,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Other_commands {
    login,
    token,
    version,
    whoami,
    context,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Context_subcommands {
    list,
    show,
    #[strum(serialize = "set-active")]
    set_active,
    delete,
    create,
    #[strum(serialize = "set-default-app")]
    set_default_app,
    rename,
}

pub fn parse_arguments() -> ArgMatches<'static> {
    let resource_id_arg = Arg::with_name(Parameters::id.as_ref())
        .required(true)
        .help("The unique id of the resource.");

    let url_arg = Arg::with_name(Parameters::url.as_ref())
        .required(true)
        .value_name("URL")
        .help("The url of the drogue cloud api endpoint");

    let app_id_arg = Arg::with_name(Resources::app.as_ref())
        .short("a")
        .long(Resources::app.as_ref())
        .takes_value(true)
        .env("DRG_APP")
        .help("The app owning the device. Can be set with DRG_APP environment variable.");

    let spec_arg = Arg::with_name(Parameters::spec.as_ref())
        .short("s")
        .long(Parameters::spec.as_ref())
        .alias("data")
        .takes_value(true)
        .help("The spec for the resource. --data is deprecated");

    let file_arg = Arg::with_name(Parameters::filename.as_ref())
        .short("f")
        .long(Parameters::filename.as_ref())
        .takes_value(true)
        .value_name("FILE")
        .conflicts_with(Parameters::spec.as_ref())
        .help("file that contains the spec to update the resource with.");

    let token_arg = Arg::with_name(Other_commands::token.as_ref())
        .short("t")
        .takes_value(true)
        .long(Other_commands::token.as_ref())
        .help("Refresh token for authentication.");

    let config_file_arg = Arg::with_name(Parameters::config.as_ref())
        .long(Parameters::config.as_ref())
        .short("C")
        .takes_value(true)
        .global(true)
        .value_name("FILE")
        .help("Path to the drgconfig file. If not specified, reads $DRGCFG environment variable or defaults to XDG config directory for drg_config.json");

    let verbose = Arg::with_name("verbose")
        .short("v")
        .takes_value(false)
        .multiple(true)
        .global(true)
        .help("Enable verbose output. Multiple occurences increase verbosity.");

    let context_arg = Arg::with_name(Parameters::context.as_ref())
        .long(Parameters::context.as_ref())
        .short("c")
        .takes_value(true)
        .global(true)
        .env("DRG_CONTEXT")
        .help("The name of the context to use. Can be set with DRG_CONTEXT environment variable.");

    let context_id_arg = Arg::with_name(Parameters::context_id.as_ref())
        .required(true)
        .help("The id of the context");
    let token_arg = Arg::with_name(Other_commands::token.as_ref())
        .short("t")
        .long(Other_commands::token.as_ref())
        .takes_value(false)
        .multiple(false)
        .global(false)
        .help("Print a valid bearer token for the drogue cloud instance.");

    App::new("Drogue Command Line Tool")
        .version(util::VERSION)
        .author("Jb Trystram <jbtrystram@redhat.com>")
        .about("Allows to manage drogue apps and devices in a drogue-cloud instance")
        .arg(config_file_arg)
        .arg(verbose)
        .arg(context_arg)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name(Verbs::create.as_ref())
                .alias("add")
                .about("create a resource in the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("create a device.")
                        .arg(&resource_id_arg)
                        .arg(&app_id_arg)
                        .arg(&spec_arg)
                        .arg(&file_arg),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("create an app.")
                        .arg(&resource_id_arg)
                        .arg(&spec_arg)
                        .arg(&file_arg),
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
                        .arg(&resource_id_arg)
                        .arg(&app_id_arg),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("create an app.")
                        .arg(&resource_id_arg),
                ),
        )
        .subcommand(
            SubCommand::with_name(Verbs::get.as_ref())
                .about("Read a resource from the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("Retrieve a device spec.")
                        .arg(&resource_id_arg)
                        .arg(&app_id_arg),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("retrieve an app spec.")
                        .arg(&resource_id_arg),
                ),
        )
        .subcommand(
            SubCommand::with_name(Verbs::edit.as_ref())
                .alias("update")
                .about("Update a resource from the drogue-cloud registry")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Resources::device.as_ref())
                        .about("Edit a device spec.")
                        .arg(&resource_id_arg)
                        .arg(&app_id_arg)
                        .arg(&file_arg),
                )
                .subcommand(
                    SubCommand::with_name(Resources::app.as_ref())
                        .about("Edit an app spec.")
                        .arg(&resource_id_arg)
                        .arg(&file_arg),
                ),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::version.as_ref())
                .about("Print version information."),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::login.as_ref())
                .arg(&token_arg)
                .about("Log into a drogue cloud installation.")
                .arg(&url_arg),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::token.as_ref())
                .about("Print a valid bearer token for the drogue cloud instance."),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::whoami.as_ref())
                .about("Print cluster adress, version and default app(if any)")
                .arg(token_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name(Other_commands::context.as_ref())
                .about("Manage contexts in the configuration file.")
                .alias("config")
                .setting(AppSettings::ArgRequiredElseHelp)
                .subcommand(
                    SubCommand::with_name(Context_subcommands::create.as_ref())
                        .setting(AppSettings::Hidden)
                        .arg(&context_id_arg),
                )
                .subcommand(
                    SubCommand::with_name(Context_subcommands::list.as_ref())
                        .about("List existing contexts names in configuration file"),
                )
                .subcommand(
                    SubCommand::with_name(Context_subcommands::show.as_ref())
                        .about("Show full configuration file"),
                )
                .subcommand(
                    SubCommand::with_name("set-active")
                        .about("Set a context as the active context")
                        .arg(&context_id_arg),
                )
                .subcommand(
                    SubCommand::with_name(Context_subcommands::delete.as_ref())
                        .alias("remove")
                        .about("Set a context as the active context")
                        .arg(&context_id_arg),
                )
                .subcommand(
                    SubCommand::with_name("set-default-app")
                        .about("Set a default-app for a context.")
                        .arg(&resource_id_arg),
                )
                .subcommand(
                    SubCommand::with_name(Context_subcommands::rename.as_ref())
                        .about("Rename a context.")
                        .arg(&context_id_arg)
                        .arg(
                            Arg::with_name("new_context_id")
                                .required(true)
                                .help("The new context name")
                                .conflicts_with(Parameters::context.as_ref()),
                        ),
                ),
        )
        .get_matches()
}

pub fn get_app_id<'a>(matches: &'a ArgMatches, config: &'a Context) -> Result<AppId> {
    match matches.value_of(Resources::app) {
        Some(a) => Ok(a.to_string()),
        None => config
            .default_app
            .as_ref()
            .map(|v| {
                println!("Using default app \"{}\".", &v);
                v.to_string()
            })
            .ok_or_else(|| {
                anyhow!("Missing app argument and no default app specified in config file.")
            }),
    }
}
