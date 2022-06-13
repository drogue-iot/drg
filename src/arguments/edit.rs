use crate::{
    admin, arguments, display_simple, util, ApplicationOperation, Context, DeviceOperation,
    Parameters, ResourceId, ResourceType,
};
use anyhow::Result;
use clap::ArgMatches;
use std::str::FromStr;

pub async fn subcommand(matches: &ArgMatches, context: &Context, json_output: bool) -> Result<i32> {
    let (res, command) = matches.subcommand().unwrap();
    let resource = ResourceType::from_str(res);

    match resource? {
        ResourceType::application => {
            let file = command.value_of(Parameters::filename.as_ref());
            let id = command
                .value_of(ResourceId::applicationId.as_ref())
                .map(|s| s.to_string());
            let spec = util::json_parse_option(command.value_of(Parameters::spec.as_ref()))?;

            display_simple(
                ApplicationOperation::new(id, file, spec)?
                    .edit(context)
                    .await,
                json_output,
            )
        }
        ResourceType::device => {
            let dev_id = command
                .value_of(ResourceId::deviceId.as_ref())
                .map(|s| s.to_string());
            let file = command.value_of(Parameters::filename.as_ref());
            let app_id = arguments::get_app_id(command, context)?;

            display_simple(
                DeviceOperation::new(app_id, dev_id.clone(), file, None)?
                    .edit(context)
                    .await,
                json_output,
            )
        }
        ResourceType::member => {
            let app_id = arguments::get_app_id(command, context)?;
            display_simple(admin::member_edit(context, &app_id).await, json_output)
        }
        // The other enum variants are not exposed by clap
        _ => unreachable!(),
    }
}
