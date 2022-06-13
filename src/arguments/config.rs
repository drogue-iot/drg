use crate::util::DrogueError;
use crate::{util, Config, Parameters, ResourceId};

use clap::ArgMatches;
use std::str::FromStr;

pub fn subcommand(
    matches: &ArgMatches,
    config: &mut Config,
    ctx_name: &Option<String>,
) -> Result<i32, DrogueError> {
    let (v, c) = matches.subcommand().unwrap();

    let ctx_id = c
        .value_of(ResourceId::contextId.as_ref())
        .map(|s| s.to_string());

    match v {
        "create" => {
            println!("To create a new context use drg login");
        }
        "list" => {
            config.list_contexts();
        }
        "show" => {
            if c.is_present("active") {
                config.get_context(ctx_name).map(|c| println!("{}", c))?;
            } else {
                println!("{}", config);
            }
        }
        "default-context" => {
            config.set_active_context(ctx_id.unwrap())?;
        }
        "delete" => {
            let id = ctx_id.unwrap();
            config.delete_context(&id)?;
        }
        "default-app" => {
            let id = c
                .value_of(ResourceId::applicationId.as_ref())
                .unwrap()
                .to_string();
            let context = config.get_context_mut(&ctx_id)?;

            context.set_default_app(id);
        }
        "rename" => {
            let new_ctx = c.value_of("new_context_id").unwrap().to_string();

            config.rename_context(ctx_id.unwrap(), new_ctx)?;
        }
        "default-algo" => {
            let algo = c
                .value_of(&Parameters::algo.as_ref())
                .map(|a| util::SignAlgo::from_str(a).unwrap())
                .unwrap();
            let context = config.get_context_mut(&ctx_id)?;

            context.set_default_algo(algo);
        }
        _ => {
            unreachable!("forgot to route config subcommand : {}", v);
        }
    }
    return Ok(0);
}
