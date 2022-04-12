use crate::{trust, util, AppId};

use crate::config::Context;
use anyhow::{anyhow, Result};
use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches};
use std::convert::AsRef;
use strum_macros::{AsRefStr, EnumString};

/// Drg CLI follows a "action resourceType resourceId options" pattern.
/// Rarely, the resource Id is optional

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Action {
    create,
    delete,
    edit,
    get,
    set,
    command,
    stream,
    login,
    transfer,
    version,
    whoami,
    config,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
// the transfer action subcommands
pub enum Transfer {
    init,
    accept,
    cancel,
}

#[derive(AsRefStr, EnumString, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum ResourceType {
    device,
    application,
    member,
    #[strum(serialize = "app-cert")]
    app_cert,
    #[strum(serialize = "device-cert")]
    device_cert,
    token,
    context,

    // resources for the set command
    gateway,
    password,
    alias,
    label,
    #[strum(serialize = "default-app")]
    default_app,
    #[strum(serialize = "default-context")]
    default_context,

    // for the login command
    url,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum ResourceId {
    deviceId,
    applicationId,
    username,
    role,
    contextId,
    tokenPrefix,
    gatewayId,
}

#[derive(AsRefStr, EnumString)]
#[allow(non_camel_case_types)]
pub enum Parameters {
    // global flags
    verbose,
    config,
    context,

    // login command
    url,
    #[strum(serialize = "keep-current")]
    keep_current,

    // specific to CRUD commands
    cert,
    spec,
    filename,
    labels,
    #[strum(serialize = "ignore-missing")]
    ignore_missing,

    // specific to certificate commands (create app-cert & create device-cert)
    #[strum(serialize = "key-input")]
    key_input,
    #[strum(serialize = "key-output")]
    key_output,
    #[strum(serialize = "ca-key")]
    ca_key,
    #[strum(serialize = "output")]
    cert_output,
    days,
    algo,

    // send command
    command,

    // specific to set command
    alias,
    password,
    payload,
    role,
    username,
    label,

    // stream command
    count,
    device,

    // login & whoami command
    token,
    endpoints,
    description,
    #[strum(serialize = "access-token")]
    access_token,
}

pub fn app_arguments() -> clap::App<'static> {
    let device_id = Arg::new(ResourceId::deviceId.as_ref()).help("The id of the device.");

    let app_id = Arg::new(ResourceId::applicationId.as_ref()).help("The id of the application.");

    let wide_flag = Arg::new("wide-flag")
        .short('w')
        .long("wide")
        .takes_value(false)
        .help("Wide display output format");

    let app_flag = Arg::new("app-flag")
        .short('a')
        .long("application")
        .takes_value(true)
        .env("DRG_APP")
        .value_name("applicationId")
        .help("Application id. Can be set with DRG_APP environment variable.");

    let spec = Arg::new(Parameters::spec.as_ref())
        .alias("data")
        .short('s')
        .long(Parameters::spec.as_ref())
        .takes_value(true)
        .help("The spec for the resource. --data is deprecated.");

    let file = Arg::new(Parameters::filename.as_ref())
        .short('f')
        .long(Parameters::filename.as_ref())
        .takes_value(true)
        .value_name("FILE")
        .help("File containing the data to create or update the resource with.")
        .long_help("File containing the data to create or update the resource with. \
            Note: unlike the --spec argument which cover only the spec section of the resource, \
            the file provided with --file must contains the complete resource object, including metadata.");

    let member = Arg::new(ResourceType::member.as_ref())
        .required(true)
        .help("Username to which roles are assigned.");

    let role = Arg::new(Parameters::role.as_ref())
        .long(Parameters::role.as_ref())
        .takes_value(true)
        .required(true)
        .help("Role assigned to this member")
        .possible_values(["admin", "manager", "reader"]);

    let ca_key = Arg::new(Parameters::ca_key.as_ref())
        .long(Parameters::ca_key.as_ref())
        .takes_value(true)
        .required(true)
        .help("Private key of the CA i.e application.");

    let keyout = Arg::new(Parameters::key_output.as_ref())
        .takes_value(true)
        .required(false)
        .long(Parameters::key_output.as_ref())
        .help("Generate and Output file containing the private key. Later to be used to sign device certificates, or device authentication.");

    let key_input = Arg::new(Parameters::key_input.as_ref())
        .long(Parameters::key_input.as_ref())
        .takes_value(true)
        .required(false)
        .help("Input private key to be used to sign CA/device certificates.");

    let device_name_subj = Arg::new(Parameters::cert.as_ref())
        .long(Parameters::cert.as_ref())
        .takes_value(false)
        .help("Creates device with the same name as the subject of device certificate.")
        .long_help(
            "X.509 authentication requires that the name of the device should \
            to be equal to the subject of the device's certificate. This flag \
            converts the given device name into the required format.",
        );

    let cert_out = Arg::new(Parameters::cert_output.as_ref())
        .long(Parameters::cert_output.as_ref())
        .short('o')
        .takes_value(true)
        .required(false)
        .help("Output device certificate to file.");

    // Default value comes from trust::CERT_VALIDITY_DAYS
    let cert_valid_days = Arg::new(Parameters::days.as_ref())
        .long(Parameters::days.as_ref())
        .takes_value(true)
        .required(false)
        .help("Number of days the certificate should be valid for. [default: 365]")
        .validator(|n| match n.parse::<u64>() {
            Err(_) => Err(String::from("The value is not an integer")),
            Ok(_) => Ok(()),
        });

    let algo_param = Arg::new(Parameters::algo.as_ref())
        .required(true)
        .help("Algorithm used to generate key pair.")
        .possible_values([
            trust::SignAlgo::ECDSA.as_ref(),
            trust::SignAlgo::EdDSA.as_ref(),
            trust::SignAlgo::RSA.as_ref(),
        ]);

    let key_pair_algorithm = algo_param
        .clone()
        .required(false)
        .takes_value(true)
        .long(Parameters::algo.as_ref());

    let access_token_description = Arg::new(Parameters::description.as_ref())
        .long(Parameters::description.as_ref())
        .help("Description to attach to the access token.")
        .takes_value(true);

    // create subcommand
    let create = App::new(Action::create.as_ref())
        .visible_alias("add")
        .about("Create a resource.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new(ResourceType::device.as_ref())
                .about("Create a device in Drogue Cloud")
                .arg(&device_id)
                .arg(&spec.clone().conflicts_with(Parameters::filename.as_ref()))
                .arg(&app_flag)
                .arg(&file)
                .arg(&device_name_subj)
                .group(
                    ArgGroup::new("name")
                        .required(true)
                        .args(&[ResourceId::deviceId.as_ref(), Parameters::filename.as_ref()]),
                ),
        )
        .subcommand(
            App::new(ResourceType::application.as_ref())
                .alias("app")
                .about("Create an application in Drogue Cloud")
                .arg(app_id.clone())
                .arg(&spec.clone().conflicts_with(Parameters::filename.as_ref()))
                .arg(
                    file.clone()
                        .required_unless_present(ResourceId::applicationId.as_ref()),
                )
                .group(ArgGroup::new("name").required(true).args(&[
                    ResourceId::applicationId.as_ref(),
                    Parameters::filename.as_ref(),
                ])),
        )
        .subcommand(
            App::new(ResourceType::member.as_ref())
                .about("Allow a member to access an application")
                .arg(&app_flag)
                .arg(&member)
                .arg(&role),
        )
        .subcommand(
            App::new(ResourceType::app_cert.as_ref())
                .about("Create a trust-anchor for an application.")
                .arg(&app_flag)
                .arg(&key_pair_algorithm)
                .arg(&cert_valid_days)
                .arg(&key_input)
                .arg(&keyout),
        )
        .subcommand(
            App::new(ResourceType::device_cert.as_ref())
                .about("Generate and sign a device certificate using application's private key.")
                .arg(&device_id)
                .arg(&app_flag)
                .arg(&ca_key)
                .arg(&cert_out)
                .arg(&keyout)
                .arg(&key_pair_algorithm)
                .arg(&cert_valid_days)
                .arg(&key_input),
        )
        .subcommand(
            App::new(ResourceType::token.as_ref())
                .about("Generate a new API access token")
                .alias("tokens")
                .arg(&access_token_description),
        );

    // edit subcommand
    let edit = App::new(Action::edit.as_ref())
        .about("Modify an existing resource.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new(ResourceType::device.as_ref())
                .about("Edit a device in Drogue Cloud")
                .arg(&device_id)
                .arg(&app_flag)
                .arg(&file)
                .group(
                    ArgGroup::new("name")
                        .required(true)
                        .args(&[ResourceId::deviceId.as_ref(), Parameters::filename.as_ref()]),
                ),
        )
        .subcommand(
            App::new(ResourceType::application.as_ref())
                .about("Edit an application in Drogue Cloud")
                .arg(&app_id)
                .arg(&spec)
                .arg(&file)
                .group(ArgGroup::new("name").required(true).args(&[
                    ResourceId::applicationId.as_ref(),
                    Parameters::filename.as_ref(),
                ])),
        )
        .subcommand(
            App::new(ResourceType::member.as_ref())
                .about("Edit application members")
                .arg(&app_flag),
        );

    let labels = Arg::new(Parameters::labels.as_ref())
        .required(false)
        .short('l')
        .long(Parameters::labels.as_ref())
        .use_delimiter(true)
        .multiple_values(true)
        .help("A comma separated list of the label filters to filter the list with.");

    // get subcommand
    let get = App::new(Action::get.as_ref())
        .about("Display one or multiple resources from the drogue-cloud registry")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new(ResourceType::device.as_ref())
                .alias("devices")
                .about("Retrieve a device spec. If no device ID is passed, list all devices for the app.")
                .arg(&device_id)
                .arg(&app_flag)
                .arg(&labels)
                .arg(&wide_flag)
        )
        .subcommand(
            App::new(ResourceType::application.as_ref())
                .aliases(&["applications", "app", "apps"])
                .about("Retrieve application details. If no application ID is passed, list all apps the user have access to.")
                .arg(&app_id)
                .arg(&labels)
        )
        .subcommand(
            App::new(ResourceType::member.as_ref())
                .alias("members")
                .about("List all members of the application")
                .arg(&app_flag)
        )
        .subcommand(
            App::new(ResourceType::token.as_ref())
                .alias("tokens")
                .about("List created access tokens for this account")
        );

    let ignore_missing = Arg::new(Parameters::ignore_missing.as_ref())
        .long(Parameters::ignore_missing.as_ref())
        .takes_value(false)
        .global(false)
        .help("Silence the error if the resource does not exist.");

    let token_prefix = Arg::new(ResourceId::tokenPrefix.as_ref())
        .required(true)
        .help("The token prefix.");

    // delete subcommand
    let delete = App::new(Action::delete.as_ref())
        .about("Delete resources in Drogue Cloud")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(&ignore_missing.global(true))
        .subcommand(
            App::new(ResourceType::application.as_ref())
                .alias("app")
                .about("Delete an application")
                .arg(&app_id.clone().required(true)),
        )
        .subcommand(
            App::new(ResourceType::device.as_ref())
                .about("Delete a device from an application.")
                .arg(&device_id.clone().required(true))
                .arg(&app_flag),
        )
        .subcommand(
            App::new(ResourceType::member.as_ref())
                .about("Remove a user from the members list for this application")
                .arg(&app_flag)
                .arg(&member),
        )
        .subcommand(
            App::new(ResourceType::token.as_ref())
                .about("Delete an API access token")
                .arg(&token_prefix),
        );

    let username = Arg::new(ResourceId::username.as_ref())
        .help("Username of the recipient of the application transfer request")
        .required(true);

    // transfer subcommand
    let transfer = App::new(Action::transfer.as_ref())
        .about("Transfer ownership of an application to another member")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new(Transfer::init.as_ref())
                .about("Initiate the application transfer")
                .arg(&app_flag)
                .arg(&username),
        )
        .subcommand(
            App::new(Transfer::accept.as_ref())
                .about("Accept an application transfer")
                .arg(app_id.clone().required(true)),
        )
        .subcommand(
            App::new(Transfer::cancel.as_ref())
                .about("Cancel an application transfer")
                .arg(app_id.clone().required(true)),
        );

    let gateway_id = Arg::new(ResourceId::gatewayId.as_ref())
        .required(true)
        .help("The id of the gateway device");

    let password = Arg::new(Parameters::password.as_ref())
        .required(true)
        .help("The credential password value");

    let set_password_username = Arg::new(Parameters::username.as_ref())
        .short('u')
        .long("username")
        .takes_value(true)
        .value_name("username")
        .help("The credential username value");

    let alias_id = Arg::new(ResourceType::alias.as_ref())
        .required(true)
        .help("The alias id for the device");

    let label = Arg::new(Parameters::label.as_ref())
        .required(true)
        .multiple_values(true)
        .help("The labels and values must be separated by an equal sign:'='")
        .long_help("The labels and values must be separated by an equal sign:'='. Multiples labels are accepted.")
        .value_name("key=value");

    let dev_flag = Arg::new("dev-flag")
        .long("device")
        .takes_value(true)
        .value_name("DeviceId")
        .help("Device to attach the label(s). If omitted, the label will be applied to the app.");

    // set subcommand
    let set = App::new(Action::set.as_ref())
        .about("Shortcuts to configure properties for apps or devices")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(app_flag.clone().global(true))
        .subcommand(
            App::new(ResourceType::gateway.as_ref())
                .about("Allow another device to act as gateway for a device")
                .arg(device_id.clone().required(true))
                .arg(&gateway_id),
        )
        .subcommand(
            App::new(ResourceType::password.as_ref())
                .about("Set a password credentials for a device")
                .arg(&password)
                .arg(device_id.clone().required(true))
                .arg(&set_password_username),
        )
        .subcommand(
            App::new(ResourceType::alias.as_ref())
                .about("Add an alias for a device")
                .arg(device_id.clone().required(true))
                .arg(alias_id),
        )
        .subcommand(
            App::new(ResourceType::label.as_ref())
                .about("Set a label to a device or application")
                .arg(&label)
                .arg(dev_flag)
                .arg(&app_flag),
        );

    let count = Arg::new(Parameters::count.as_ref())
        .required(false)
        .short('n')
        .takes_value(true)
        .global(true)
        .help("The number of messages to stream before exiting.");

    let stream = App::new(Action::stream.as_ref())
        .about("Stream all events going through drogue cloud")
        .arg(&app_flag)
        .arg(&count)
        .arg(
            Arg::new(Parameters::device.as_ref())
                .long("device")
                .takes_value(true)
                .value_name("deviceId")
                .help("Filter events comming from this device."),
        );

    let context_id = Arg::new(ResourceId::contextId.as_ref())
        //.conflicts_with(Parameters::context.as_ref())
        .required(true)
        .help("The context Id");

    let config = App::new(Action::config.as_ref())
        .about("Manage the configuration file")
        .alias("context")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new(Action::create.as_ref())
                .setting(AppSettings::Hidden)
                .about("This subcommand is invalid. To create a new context use drg login."),
        )
        .subcommand(App::new("list").about("List existing contexts names in configuration file"))
        .subcommand(App::new("show").about("Show full configuration file"))
        .subcommand(
            App::new("default-context")
                .about("Set a context as the active context")
                .arg(&context_id),
        )
        .subcommand(
            App::new("delete")
                .alias("remove")
                .about("Delete a context")
                .arg(&context_id),
        )
        .subcommand(
            App::new("default-app")
                .about("Set a default-app for a context.")
                .arg(&app_id),
        )
        .subcommand(
            App::new("rename")
                .about("Rename a context.")
                .arg(&context_id)
                .arg(
                    Arg::new("new_context_id")
                        .required(true)
                        .help("The new context name"),
                ),
        )
        .subcommand(
            App::new("default-algo")
                .about("Set a default key generation algorithm for a context.")
                .arg(&algo_param),
        );

    let command = Arg::new(Parameters::command.as_ref())
        .required(true)
        .help("The name of the command to send to the device");

    let url = Arg::new(Parameters::url.as_ref())
        .required(true)
        .value_name("URL")
        .help("The url of the drogue cloud api endpoint");

    let payload_arg = Arg::new(Parameters::payload.as_ref())
        .short('p')
        .long(Parameters::payload.as_ref())
        .takes_value(true)
        .required(false)
        .help("The command body, as a JSON value.");

    let token_arg = Arg::new(Parameters::token.as_ref())
        .short('t')
        .takes_value(true)
        .long(Parameters::token.as_ref())
        .help("Refresh token for authentication. This flag is deprecated. Please use access-token with an API access token.");

    let access_token_arg = Arg::new(Parameters::access_token.as_ref())
        .takes_value(true)
        .long(Parameters::access_token.as_ref())
        .conflicts_with(Parameters::token.as_ref())
        .value_name("username:token")
        .help("Access token for authentication.");

    let config_file_arg = Arg::new(Parameters::config.as_ref())
        .long(Parameters::config.as_ref())
        .short('C')
        .takes_value(true)
        .global(true)
        .value_name("FILE")
        .help("Path to the drgconfig file. If not specified, reads $DRGCFG environment variable or defaults to XDG config directory for drg_config.json");

    let verbose = Arg::new(Parameters::verbose.as_ref())
        .short('v')
        .takes_value(false)
        .multiple_occurrences(true)
        .global(true)
        .help("Enable verbose output. Multiple occurrences increase verbosity.");

    let context_arg = Arg::new(ResourceId::contextId.as_ref())
        .long(Parameters::context.as_ref())
        .short('c')
        .takes_value(true)
        .global(true)
        .env("DRG_CONTEXT")
        .help("The name of the context to use. Can be set with DRG_CONTEXT environment variable.");

    let login_keep_current = Arg::new(Parameters::keep_current.as_ref())
        .short('k')
        .help("Do not activate the new context.");

    App::new("Drogue Command Line Tool")
        .version(util::VERSION)
        .author("Jb Trystram <jbtrystram@redhat.com>")
        .about("Allows to manage drogue apps and devices in a drogue-cloud instance")
        .arg(config_file_arg)
        .arg(verbose)
        .arg(&context_arg)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(create)
        .subcommand(delete)
        .subcommand(edit)
        .subcommand(get)
        .subcommand(set)
        .subcommand(stream)
        .subcommand(config)
        .subcommand(transfer)
        .subcommand(
            App::new(Action::command.as_ref())
                .alias("cmd")
                .about("Send a command to a device")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(&device_id.required(true))
                .arg(&command)
                .arg(&app_flag)
                .arg(&payload_arg)
                .arg(
                    &file
                        .clone()
                        .help("File containing the command payload as a JSON object."),
                )
                .group(
                    ArgGroup::new("data")
                        .required(true)
                        .args(&[Parameters::payload.as_ref(), Parameters::filename.as_ref()]),
                ),
        )
        .subcommand(App::new(Action::version.as_ref()).about("Print version information."))
        .subcommand(
            App::new(Action::login.as_ref())
                .arg(&token_arg)
                .arg(&access_token_arg)
                .about("Log into a drogue cloud installation.")
                .arg(&url)
                .arg(&login_keep_current),
        )
        .subcommand(
            App::new(Action::whoami.as_ref())
                .about("Print cluster adress, version and default app(if any)")
                .arg(token_arg.clone().takes_value(false).help(
                    "Pulls an valid token from the context to authenticate against drogue cloud.",
                ))
                .subcommand(
                    App::new(Parameters::endpoints.as_ref())
                        .about("List drogue-cloud available endpoints.")
                        .aliases(&["-e", "endpoint", "--endpoints"])
                        .arg(
                            Arg::new(Parameters::endpoints.as_ref())
                                .takes_value(true)
                                .required(false)
                                .help("Specify an endpoint name to get only it's URI.")
                                .value_name("endpoint_name"),
                        ),
                ),
        )
}

pub fn get_wide<'a>(matches: &'a ArgMatches) -> bool {
    matches.is_present("wide-flag")
}

pub fn get_app_id<'a>(matches: &'a ArgMatches, config: &'a Context) -> Result<AppId> {
    match matches.value_of("app-flag") {
        Some(a) => Ok(a.to_string()),
        None => config
            .default_app
            .as_ref()
            .map(|v| {
                log::debug!("Using default app \"{}\".", &v);
                v.to_string()
            })
            .ok_or_else(|| {
                anyhow!("Missing app argument and no default app specified in config file.")
            }),
    }
}

#[test]
fn verify_app() {
    app_arguments().debug_assert();
}
