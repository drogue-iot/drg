use crate::util::DrogueError;
use crate::{
    config::pretty_list, display, display_simple, util, Config, Outcome, Parameters, ResourceId,
};
use anyhow::Result;

use clap::ArgMatches;
use std::str::FromStr;

pub fn subcommand(
    matches: &ArgMatches,
    config: &mut Config,
    ctx_name: &Option<String>,
    json: bool,
) -> Result<i32> {
    let (v, c) = matches.subcommand().unwrap();

    match v {
        "create" => display_simple::<String>(
            Err(DrogueError::InvalidInput(
                "To create a new context use drg login".to_string(),
            )),
            json,
        ),
        "list" => display(config.list_contexts(), json, |c| {
            pretty_list(c, &config.active_context)
        }),
        "show" => {
            if c.is_present("active") {
                let c = config
                    .get_context(ctx_name)
                    .map(Outcome::SuccessWithJsonData)
                    .map_err(|e| DrogueError::ConfigIssue(e.to_string()));

                display(c, json, |c| println!("{}", c))
            } else {
                display(
                    Ok(Outcome::SuccessWithJsonData(config.clone())),
                    json,
                    |c| println!("{}", c),
                )
            }
        }
        "default-context" => {
            display_simple(config.set_active_context(ctx_name.clone().unwrap()), json)
        }
        "delete" => {
            let id = ctx_name.clone().unwrap();
            display_simple(config.delete_context(&id), json)
        }
        "default-app" => {
            let id = c
                .value_of(ResourceId::applicationId.as_ref())
                .unwrap()
                .to_string();
            let context = config.get_context_mut(ctx_name)?;
            let outcome = context.set_default_app(id);
            config.changed(true);

            display_simple(Ok(outcome), json)
        }
        "rename" => {
            let new_ctx = c.value_of("new_context_id").unwrap().to_string();

            display_simple(
                config.rename_context(ctx_name.clone().unwrap(), new_ctx),
                json,
            )
        }
        "default-algo" => {
            let algo = c
                .value_of(&Parameters::algo.as_ref())
                .map(|a| util::SignAlgo::from_str(a).unwrap())
                .unwrap();
            let context = config.get_context_mut(ctx_name)?;
            let outcome = context.set_default_algo(algo);
            config.changed(true);
            display_simple(Ok(outcome), json)
        }
        _ => {
            unreachable!("forgot to route config subcommand : {}", v);
        }
    }
}
