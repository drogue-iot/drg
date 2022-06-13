use crate::{
    admin, applications, arguments, devices, display, tokens, ApplicationOperation, Context,
    DeviceOperation, Parameters, ResourceId, ResourceType,
};
use anyhow::Result;
use clap::ArgMatches;
use std::str::FromStr;

pub async fn subcommand(matches: &ArgMatches, context: &Context, json_output: bool) -> Result<i32> {
    let (res, command) = matches.subcommand().unwrap();
    let resource = ResourceType::from_str(res)?;

    match resource {
        ResourceType::application => {
            let app_id = command
                .value_of(ResourceId::applicationId.as_ref())
                .map(|s| s.to_string());
            let labels = command.values_of(Parameters::labels.as_ref());

            let op = ApplicationOperation::new(app_id.clone(), None, None)?;
            match app_id {
                Some(_) => display(op.read(context).await, json_output, |app| {
                    applications::pretty_list(&vec![app.clone()])
                }),
                None => display(
                    op.list(context, labels).await,
                    json_output,
                    applications::pretty_list,
                ),
            }
        }
        ResourceType::device => {
            let wide = command
                .value_of(Parameters::output.as_ref())
                .map(|v| v == "wide")
                .unwrap_or(false);
            let app_id = arguments::get_app_id(command, context)?;
            let labels = command.values_of(Parameters::labels.as_ref());
            let dev_id = command
                .value_of(ResourceId::deviceId.as_ref())
                .map(|s| s.to_string());

            let op = DeviceOperation::new(app_id, dev_id.clone(), None, None)?;
            match dev_id {
                //fixme : add a pretty print for a single device ?
                Some(_) => display(op.read(context).await, json_output, |d| {
                    devices::pretty_list(&vec![d.clone()], wide)
                }),
                None => display(op.list(context, labels).await, json_output, |d| {
                    devices::pretty_list(d, wide)
                }),
            }
        }
        ResourceType::member => {
            let app_id = arguments::get_app_id(command, context)?;
            display(
                admin::member_list(context, &app_id).await,
                json_output,
                admin::members_table,
            )
        }
        ResourceType::token => display(
            tokens::get_api_keys(context).await,
            json_output,
            tokens::tokens_table,
        ),
        // The other enum variants are not exposed by clap
        _ => unreachable!(),
    }
}
