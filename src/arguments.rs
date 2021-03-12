use crate::util;

use clap::{Arg, App, SubCommand, ArgMatches};
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
}

pub fn parse_arguments() -> ArgMatches<'static> {

let resource_id_arg = Arg::with_name(Parameters::id.as_ref())
    .required(true)
    .help("The unique id of the resource.");

let url_arg = Arg::with_name(Parameters::url.as_ref())
    .short("u")
    .required(true)
    .takes_value(true)
    .help("The url of the registry endpoint");

let app_id_arg = Arg::with_name(Resources::app.as_ref())
    .long("app")
    .required(true)
    .takes_value(true)
    .help("The app owning the device.");

let data_arg = Arg::with_name(Parameters::data.as_ref())
    .short("d")
    .long("data")
    .takes_value(true)
    .help("The data for the resource.");


App::new("Drogue Command Line Tool")
    .version(util::VERSION)
    .author("Jb Trystram <jbtrystram@redhat.com>")
    .about("Allows to manage drogue apps and devices in a drogue-cloud instance")
    .subcommand(SubCommand::with_name(Verbs::create.as_ref())
            .about("create a resource in the drogue-cloud registry")
            .arg(url_arg.clone())
            .subcommand(
                SubCommand::with_name(Resources::device.as_ref())
                    .about("create a device.")
                    .arg(resource_id_arg.clone())
                    .arg(app_id_arg.clone())
                    .arg(data_arg.clone())
            ).subcommand(SubCommand::with_name(Resources::app.as_ref())
                .about("create an app.")
                .arg(resource_id_arg.clone())
                .arg(data_arg.clone())
            )
    ).subcommand(
        SubCommand::with_name(Verbs::delete.as_ref())
            .about("delete a resource in the drogue-cloud registry")
            .arg(url_arg.clone())
            .subcommand(
                SubCommand::with_name(Resources::device.as_ref())
                    .about("delete a device.")
                    .arg(resource_id_arg.clone())
                    .arg(app_id_arg.clone())
            ).subcommand(SubCommand::with_name(Resources::app.as_ref())
            .about("create an app.")
            .arg(resource_id_arg.clone())
            )
    ).subcommand(
    SubCommand::with_name(Verbs::get.as_ref())
        .about("Read a resource from the drogue-cloud registry")
        .arg(url_arg.clone())
        .subcommand(
            SubCommand::with_name(Resources::device.as_ref())
                .about("Retrieve a device data.")
                .arg(resource_id_arg.clone())
                .arg(app_id_arg.clone())
        ).subcommand(SubCommand::with_name(Resources::app.as_ref())
        .about("retrieve an app data.")
        .arg(resource_id_arg.clone())
        )
    ).subcommand(
    SubCommand::with_name(Verbs::edit.as_ref())
        .about("Edit a resource from the drogue-cloud registry")
        .arg(url_arg.clone())
        .subcommand(
            SubCommand::with_name(Resources::device.as_ref())
                .about("Edit a device data.")
                .arg(resource_id_arg.clone())
                .arg(app_id_arg.clone())
        ).subcommand(SubCommand::with_name(Resources::app.as_ref())
        .about("Edit an app data.")
        .arg(resource_id_arg.clone())
        )
    ).subcommand(
            SubCommand::with_name("version")
                .about("Print version information.")
    ).get_matches()
}