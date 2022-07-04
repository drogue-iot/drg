use crate::{openid, util, Config, DrogueError, Outcome, Parameters};
use clap::ArgMatches;

pub async fn subcommand(
    matches: &ArgMatches,
    config: &mut Config,
    ctx_name: &Option<String>,
) -> Result<Outcome<String>, DrogueError> {
    let url = util::url_validation(matches.value_of(Parameters::url.as_ref()).unwrap())?;
    let access_token_val = matches.value_of(Parameters::access_token.as_ref());
    let ctx_name = ctx_name.clone().unwrap_or_else(|| "default".to_string());

    let context = if let Some(access_token) = access_token_val {
        if let Some((id, token)) = access_token.split_once(':') {
            util::context_from_access_token(ctx_name, url.clone(), id, token)
                .await
                .map_err(|e| DrogueError::InvalidInput(format!("{e}")))
        } else {
            Err(DrogueError::InvalidInput(
                "Invalid access token. Format should be username:token".to_string(),
            ))
        }
    } else {
        let refresh_token_val = matches.value_of(Parameters::token.as_ref());
        openid::login(url.clone(), refresh_token_val, ctx_name)
            .await
            .map_err(|e| DrogueError::InvalidInput(format!("{e}")))
    }?;

    let mut message = format!(
        "Successfully authenticated to drogue cloud : {}\nSaved context: {}",
        url, context.name
    );

    let name = context.name.clone();
    config.add_context(context)?;

    if !matches.is_present(Parameters::keep_current.as_ref()) {
        let _ = config.set_active_context(name.clone())?;
        message = format!("{}\nSwitched active context to: {}", message, name);
    }

    Ok(Outcome::SuccessWithMessage(message))
}
