use crate::util::SignAlgo;

use anyhow::{anyhow, Context as AnyhowContext, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::{env, fs::create_dir_all, fs::write, fs::File, path::Path, process::exit};

use async_trait::async_trait;
use drogue_client::openid::{Credentials, TokenProvider};

use crate::Outcome::{SuccessWithJsonData, SuccessWithMessage};
use crate::{DrogueError, Outcome};
use chrono::{DateTime, Utc};
use core::fmt;
use dirs::config_dir;
use drogue_client::error::ClientError;
use oauth2::basic::BasicTokenResponse;
use oauth2::TokenResponse;
use tabular::{Row, Table};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub active_context: Option<String>,
    pub contexts: Vec<Context>,
    #[serde(skip, default)]
    changed: bool,
    //todo : when loading, put a ref to the active context for faster access
    // to avoid looping through the contexts each time.
    //#[serde(skip)]
    //pub active_ctx_ref: Option<&'a Context>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub name: String,
    pub drogue_cloud_url: Url,
    pub default_app: Option<String>,
    pub default_algo: Option<String>,
    pub auth_url: Url,
    pub token_url: Url,
    pub registry_url: Url,
    pub token_exp_date: DateTime<Utc>,
    pub token: Token,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Token {
    TokenResponse(BasicTokenResponse),
    AccessToken(AccessToken),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessToken {
    pub id: String,
    pub token: String,
}

pub trait RequestBuilderExt {
    fn auth(self, token: &Token) -> Self;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    fn auth(self, token: &Token) -> Self {
        match token {
            Token::TokenResponse(token) => self.bearer_auth(&token.access_token().secret()),
            Token::AccessToken(auth) => self.basic_auth(&auth.id, Some(&auth.token)),
        }
    }
}

impl RequestBuilderExt for tungstenite::http::Request<()> {
    fn auth(mut self, token: &Token) -> Self {
        match token {
            Token::TokenResponse(token) => {
                let bearer_header = format!("Bearer {}", &token.access_token().secret());
                let mut bearer_header =
                    tungstenite::http::HeaderValue::from_str(&bearer_header).unwrap();
                bearer_header.set_sensitive(true);
                self.headers_mut()
                    .insert(tungstenite::http::header::AUTHORIZATION, bearer_header);
                self
            }
            Token::AccessToken(auth) => {
                let encoded = general_purpose::STANDARD
                    .encode(&format!("{}:{}", auth.id, auth.token).as_bytes());
                let basic_header = format!("Basic {}", encoded);
                let mut basic_header =
                    tungstenite::http::HeaderValue::from_str(&basic_header).unwrap();
                basic_header.set_sensitive(true);
                self.headers_mut()
                    .insert(tungstenite::http::header::AUTHORIZATION, basic_header);
                self
            }
        }
    }
}

impl Config {
    pub fn empty() -> Config {
        Config {
            active_context: None,
            contexts: Vec::new(),
            changed: true,
            //active_ctx_ref: None,
        }
    }
    pub fn from(path: Option<&str>) -> Result<Config, DrogueError> {
        let path = eval_config_path(path);
        log::info!("Loading configuration file: {}", &path);

        let file = File::open(path).map_err(|e| {
            DrogueError::ConfigIssue(format!(
                "Cannot open config file. Did you log in into a drogue-cloud instance? {e}"
            ))
        })?;
        let config: Config = serde_yaml::from_reader(file)
            .map_err(|_| DrogueError::ConfigIssue("Cannot deserialize config file.".to_string()))?;

        // let active_ref = config.get_active_context()?;
        // config.active_ctx_ref = Some(active_ref);
        Ok(config)
    }

    pub fn add_context(&mut self, context: Context) -> Result<()> {
        let name = &context.name;
        if self.contains_context(name) {
            self.replace_context(context)?;
        } else {
            if self.contexts.is_empty() {
                self.active_context = Some(name.clone());
            }
            self.contexts.push(context);
        }
        self.changed = true;
        Ok(())
    }

    fn replace_context(&mut self, context: Context) -> Result<()> {
        let name = &context.name;
        self.delete_context(name)?;
        self.contexts.push(context);
        self.changed = true;
        Ok(())
    }

    pub fn get_context(&self, name: &Option<String>) -> Result<&Context> {
        match name {
            Some(n) => self.get_context_as_ref(n),
            None => self.get_active_context(),
        }
    }

    pub fn get_context_mut(&mut self, name: &Option<String>) -> Result<&mut Context> {
        match name {
            Some(n) => self.get_context_as_mut(n),
            None => self.get_active_context_mut(),
        }
    }
    fn get_active_context(&self) -> Result<&Context> {
        if let Some(default) = &self.active_context.clone() {
            self.get_context_as_ref(default)
        } else {
            Err(anyhow!(
                "The config file is empty. Please log in to drogue-cloud first."
            ))
        }
    }
    fn get_active_context_mut(&mut self) -> Result<&mut Context> {
        // todo : avoid the clone ?
        if let Some(default) = &self.active_context.clone() {
            self.get_context_as_mut(default)
        } else {
            Err(anyhow!(
                "The config file is empty. Please log in to drogue-cloud first."
            ))
        }
    }
    fn get_context_as_ref(&self, name: &str) -> Result<&Context> {
        for c in &self.contexts {
            if c.name == name {
                return Ok(c);
            }
        }
        Err(anyhow!("Context \"{}\" not found in config file.", name))
    }

    fn get_context_as_mut(&mut self, name: &str) -> Result<&mut Context> {
        for c in &mut self.contexts {
            if c.name == name {
                return Ok(c);
            }
        }
        Err(anyhow!("Context \"{}\" not found in config file.", name))
    }
    fn contains_context(&self, name: &str) -> bool {
        for config in &self.contexts {
            if config.name == name {
                return true;
            }
        }
        false
    }
    pub fn list_contexts(&self) -> Result<Outcome<Vec<Context>>, DrogueError> {
        Ok(SuccessWithJsonData(self.contexts.clone()))
    }

    pub fn set_active_context(&mut self, name: String) -> Result<Outcome<String>, DrogueError> {
        if self.contains_context(&name) {
            self.active_context = Some(name.clone());
            self.changed = true;
            Ok(Outcome::SuccessWithMessage(format!(
                "Switched active context to: {}",
                &name
            )))
        } else {
            Err(DrogueError::InvalidInput(format!(
                "Context {} does not exist in config file.",
                name
            )))
        }
    }

    pub fn write(&self, path: Option<&str>) -> Result<()> {
        if self.changed {
            let path = eval_config_path(path);
            if let Some(parent) = Path::new(&path).parent() {
                create_dir_all(parent)
                    .context("Failed to create parent directory of configuration")?;
            }

            log::info!("Saving config file: {}", &path);
            write(&path, serde_yaml::to_string(&self)?)
                .context(format!("Unable to write config file :{}", path))?;
        }
        Ok(())
    }

    pub fn delete_context(&mut self, name: &str) -> Result<Outcome<String>, DrogueError> {
        if self.contains_context(name) {
            self.contexts.retain(|c| c.name != name);

            if self.active_context == Some(name.to_string()) {
                if !self.contexts.is_empty() {
                    self.active_context = Some(self.contexts[0].name.clone());
                } else {
                    self.active_context = None;
                }
            }
            self.changed = true;
            Ok(Outcome::SuccessWithMessage(format!(
                "Context {name} deleted"
            )))
        } else {
            Err(DrogueError::InvalidInput(format!(
                "Context {} does not exist in config file.",
                name
            )))
        }
    }

    pub fn rename_context(
        &mut self,
        name: String,
        new_name: String,
    ) -> Result<Outcome<String>, DrogueError> {
        if self.contains_context(&new_name) {
            Err(DrogueError::InvalidInput(format!(
                "Context {} already exists in config file.",
                new_name
            )))
        } else if self.contains_context(&name) {
            let ctx = self.get_context_as_mut(&name)?;
            ctx.rename(new_name.clone());

            if self.active_context.is_some() && self.active_context.as_ref().unwrap() == &name {
                self.active_context = Some(new_name.clone());
            }
            self.changed = true;
            Ok(SuccessWithMessage(format!(
                "Context {name} renamed to {new_name}"
            )))
        } else {
            Err(DrogueError::InvalidInput(format!(
                "Context {} does not exist in config file.",
                name
            )))
        }
    }

    pub fn changed(&mut self, changed: bool) {
        self.changed = changed;
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_yaml::to_string(&self).unwrap_or_else(|_| {
                "Cannot deserialize the config. The file may be corrupted.".to_string()
            })
        )
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_yaml::to_string(&self)
                .unwrap_or_else(|_| { "Cannot deserialize the context.".to_string() })
        )
    }
}

impl Context {
    pub fn init_with_access_token(name: String, api: Url, auth: AccessToken) -> Self {
        let dummy_url = Url::parse("https://example.net").unwrap();
        Context {
            name,
            drogue_cloud_url: api,
            token: Token::AccessToken(auth),

            default_app: None,
            default_algo: None,
            auth_url: dummy_url.clone(),
            token_url: dummy_url.clone(),
            registry_url: dummy_url,
            token_exp_date: chrono::DateTime::<Utc>::MAX_UTC,
        }
    }

    fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn set_default_app(&mut self, app: String) -> Outcome<String> {
        self.default_app = Some(app.clone());
        SuccessWithMessage(format!(
            "{} set as default application for context {}",
            &app, self.name
        ))
    }

    pub fn set_default_algo(&mut self, algo: SignAlgo) -> Outcome<String> {
        self.default_algo = Some(algo.as_ref().to_string());
        SuccessWithMessage(format!(
            "{} set as default certificate algorithm for context {}",
            algo.as_ref(),
            self.name
        ))
    }

    pub fn fill_urls(&mut self, auth: Url, registry: Url, token: Url) {
        self.token_url = token;
        self.registry_url = registry;
        self.auth_url = auth;
    }
}

#[async_trait]
impl TokenProvider for Token {
    async fn provide_access_token(&self) -> std::result::Result<Option<Credentials>, ClientError> {
        match self {
            Token::AccessToken(basic) => Ok(Some(Credentials::Basic(
                basic.id.clone(),
                Some(basic.token.clone()),
            ))),
            Token::TokenResponse(token) => Ok(Some(Credentials::Bearer(
                token.access_token().secret().clone(),
            ))),
        }
    }
}

// use the provided config path or `$DRGCFG` value if set
// otherwise will default to $XDG_CONFIG_HOME
// fall back to `$HOME/.config` if XDG var is not set.
fn eval_config_path(path: Option<&str>) -> String {
    match path {
        Some(p) => p.to_string(),
        None => env::var("DRGCFG").unwrap_or_else(|_| {
            let xdg = match config_dir() {
                Some(path) => path.into_os_string().into_string().unwrap(),
                None => {
                    log::error!("Error accessing config file, please try using --config");
                    exit(1);
                }
            };
            format!("{}/drg_config.yaml", xdg)
        }),
    }
}

pub fn pretty_list(contexts: &Vec<Context>, active: Option<&String>) {
    let mut table = Table::new("{:<}  {:<}  {:<}");
    table.add_row(
        Row::new()
            .with_cell("NAME")
            .with_cell("ADDRESS")
            .with_cell("DEFAULT APP"),
    );

    for config in contexts {
        let name = if active.is_some() && active.unwrap() == &config.name {
            format!("{} *", config.name)
        } else {
            config.name.clone()
        };
        table.add_row(
            Row::new()
                .with_cell(&name)
                .with_cell(&config.drogue_cloud_url)
                .with_cell(
                    &config
                        .default_app
                        .as_ref()
                        .unwrap_or(&"<Not Set>".to_string()),
                ),
        );
    }

    print!("{}", table);
}
