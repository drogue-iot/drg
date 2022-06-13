use crate::{
    admin, arguments, display_simple, tokens, ApplicationOperation, Context, DeviceOperation,
    Parameters, ResourceId, ResourceType,
};
use anyhow::Result;
use clap::ArgMatches;
use std::str::FromStr;

pub async fn subcommand(matches: &ArgMatches, context: &Context, json_output: bool) -> Result<i32> {
    let (res, command) = matches.subcommand().unwrap();
    let resource = ResourceType::from_str(res);

    let ignore_missing = matches.is_present(Parameters::ignore_missing.as_ref());

    match resource? {
        ResourceType::application => {
            let id = command
                .value_of(ResourceId::applicationId.as_ref())
                .unwrap()
                .to_string();
            display_simple(
                ApplicationOperation::new(Some(id), None, None)?
                    .delete(context, ignore_missing)
                    .await,
                json_output,
            )
        }
        ResourceType::device => {
            let app_id = arguments::get_app_id(command, context)?;
            let id = command
                .value_of(ResourceId::deviceId.as_ref())
                .unwrap()
                .to_string();

            display_simple(
                DeviceOperation::new(app_id, Some(id), None, None)?
                    .delete(context, ignore_missing)
                    .await,
                json_output,
            )
        }
        ResourceType::member => {
            let app_id = arguments::get_app_id(command, context)?;
            let user = command.value_of(ResourceType::member.as_ref()).unwrap();

            display_simple(
                admin::member_delete(context, app_id.as_str(), user).await,
                json_output,
            )
        }
        ResourceType::token => {
            let prefix = command.value_of(ResourceId::tokenPrefix.as_ref()).unwrap();
            display_simple(tokens::delete(context, prefix).await, json_output)
        }
        // The other enum variants are not exposed by clap
        _ => unreachable!(),
    }
}
