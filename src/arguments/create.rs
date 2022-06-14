use crate::{
    admin, arguments, display, display_simple, tokens, util, ApplicationOperation, Context,
    DeviceOperation, Parameters, ResourceId, ResourceType,
};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use drogue_client::admin::v1::Role;
use json_value_merge::Merge;
use serde_json::json;
use std::str::FromStr;

pub async fn subcommand(matches: &ArgMatches, context: &Context, json_output: bool) -> Result<i32> {
    let (res, command) = matches.subcommand().unwrap();
    let resource = ResourceType::from_str(res)?;

    match resource {
        ResourceType::application => {
            let data = util::json_parse_option(command.value_of(Parameters::spec.as_ref()))?;
            let file = command.value_of(Parameters::filename.as_ref());
            let app_id = command
                .value_of(ResourceId::applicationId.as_ref())
                .map(|s| s.to_string());

            display_simple(
                ApplicationOperation::new(app_id, file, data)?
                    .create(context)
                    .await,
                json_output,
            )
        }
        ResourceType::device => {
            let app_id = arguments::get_app_id(command, context)?;
            let mut data = util::json_parse_option(command.value_of(Parameters::spec.as_ref()))?;
            let file = command.value_of(Parameters::filename.as_ref());
            let dev_id = command
                .value_of(ResourceId::deviceId.as_ref())
                .map(|s| s.to_string());

            // add an alias with the correct subject dn.
            if command.is_present(Parameters::cert.as_ref()) {
                let alias = format!(
                    "CN={}, O=Drogue IoT, OU={}",
                    &util::name_from_json_or_file(dev_id.clone(), file)?,
                    app_id
                );
                let alias_spec = json!([alias]);

                data = match data {
                    Some(mut d) => {
                        d.merge_in("/alias", alias_spec);
                        Some(d)
                    }
                    None => Some(alias_spec),
                };
            }

            let op = DeviceOperation::new(app_id, dev_id.clone(), file, data)?;
            display_simple(op.create(context).await, json_output)
        }
        ResourceType::member => {
            let app_id = arguments::get_app_id(command, context)?;
            let role = command
                .value_of(Parameters::role.as_ref())
                .map(|r| Role::from_str(r).unwrap())
                .unwrap();

            let user = command.value_of(ResourceType::member.as_ref()).unwrap();

            display_simple(
                admin::member_add(context, &app_id, user, role).await,
                json_output,
            )
        }
        ResourceType::token => {
            let description = command.value_of(Parameters::description.as_ref());
            display(
                tokens::create(context, description).await,
                json_output,
                tokens::created_token_print,
            )
        }
        ResourceType::app_cert | ResourceType::device_cert => {
            let app_id = arguments::get_app_id(command, context)?;
            let days = command.value_of(&Parameters::days.as_ref());
            let key_pair_algorithm = command
                .value_of(&Parameters::algo.as_ref())
                .or_else(|| {
                    context.default_algo.as_deref().map(|a| {
                        log::debug!("Using default signature algorithm: {}", a);
                        a
                    })
                })
                .map(|algo| util::SignAlgo::from_str(algo).unwrap());

            let (key_input, key_pair_algorithm) =
                match command.value_of(&Parameters::key_input.as_ref()) {
                    Some(f) => util::verify_input_key(f).map(|s| (Some(s.0), Some(s.1)))?,
                    _ => (None, key_pair_algorithm),
                };

            let keyout = command.value_of(&Parameters::key_output.as_ref());

            let device_key = command.value_of(&Parameters::key_output.as_ref());

            if resource == ResourceType::app_cert {
                display_simple(
                    ApplicationOperation::new(Some(app_id), None, None)?
                        .add_trust_anchor(context, keyout, key_pair_algorithm, days, key_input)
                        .await,
                    json_output,
                )
            } else {
                // Safe unwraps because clap makes sure the argument is provided
                let dev_id = command.value_of(ResourceId::deviceId.as_ref()).unwrap();
                let ca_key = command.value_of(&Parameters::ca_key.as_ref()).unwrap();
                let device_cert = command.value_of(&Parameters::cert_output.as_ref());

                let cert = ApplicationOperation::new(Some(app_id.clone()), None, None)?
                    .get_trust_anchor(context)
                    .await?;

                match util::create_device_certificate(
                    &app_id,
                    dev_id,
                    ca_key,
                    cert.anchors[0].certificate.as_slice(),
                    device_key,
                    device_cert,
                    key_pair_algorithm,
                    days,
                    key_input,
                ) {
                    Ok(_) => {
                        let alias = format!("CN={}, O=Drogue IoT, OU={}", dev_id, app_id);

                        display_simple(
                            DeviceOperation::new(app_id, Some(dev_id.to_string()), None, None)?
                                .add_alias(context, alias)
                                .await,
                            json_output,
                        )
                    }
                    //fixme use drogueError
                    Err(e) => Err(anyhow!("Cannot create trust anchor : {e:?}")),
                }
            }
        }
        // The other enum variants are not exposed by clap
        _ => unreachable!(),
    }
}
