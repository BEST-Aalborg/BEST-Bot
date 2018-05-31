extern crate easy_toml_config;
use self::easy_toml_config::*;

use std::path::{PathBuf};
use std::fs::File;
use std::env::home_dir;

use log::LevelFilter;

static CONFIG_FILE: &'static str = "default.toml";


lazy_static! {
    pub static ref CONFIG: Config = {
        set_config_dir(String::from(".config/BEST-Bot"));
        read_config(CONFIG_FILE, config_template())
    };
}

/// Reads the config file
fn read_config(config_file: &str, config: Config) -> Config {
    use std::io::Read;
    let mut config_file = init(config_file, config);

    let mut data = String::new();
    config_file.read_to_string(&mut data);
    error_handler(toml::from_str(&data))
}

/// Main struct for config
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    plugin_path: Option<String>,
    plugin_config_path: Option<String>,
    pub slack: Slack,
    log: Option<Log>,
}

/// Struct for handling Slack keys
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Slack {
    /// The token are meant to be from a Slack Bot, but can also be from a normal user.
    /// Look in to Legacy Tokens on api.slack.com
    pub api_token: String,

    /// The token have to be from a normal user with admin privileges.
    /// Look in to Legacy Tokens on api.slack.com to figure out have to generate the token.
    pub admin_api_token: String,

    /// The token is from the app Incoming WebHooks.
    pub incoming_webhooks_token: Option<String>,

    /// The token is from the app Outcoming WebHooks.
    pub outgoing_webhooks_token: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Log {
    level: Option<String>,
    to_file: Option<bool>,
    to_terminal: Option<bool>,
    log_path: Option<String>,
}

impl Config {
    /// Get the path for the plugins
    pub fn plugin_path(&self) -> PathBuf {
        let mut plugin_path = home_dir().unwrap();
        if self.plugin_path.is_none() {
            plugin_path.push(get_config_dir());
            plugin_path.push("libs");
            return plugin_path;
        } else {
            plugin_path.push(self.plugin_path.clone().unwrap());
            return plugin_path;
        }
    }

    /// Get the path for there to the plugins should store their config file
    pub fn plugin_config_path(&self) -> PathBuf {
        let mut plugin_config_path = home_dir().unwrap();
        if self.plugin_config_path.is_none() {
            plugin_config_path.push(get_config_dir());
            plugin_config_path.push("plugins");
            return plugin_config_path;
        } else {
            plugin_config_path.push(self.plugin_config_path.clone().unwrap());
            return plugin_config_path;
        }
    }

    pub fn log(&self) -> Log {

        if self.log.is_none() {
            return config_template().log.expect("the variable 'log' was not declared in struct 'Config' in the function 'config_template', fix it");
        } else {
            return self.log.clone().unwrap();
        }
    }
}

impl Log {
    pub fn level(&self) -> LevelFilter {
        match self.level.as_ref() {
            Some(level) => match level.to_uppercase().as_ref() {
                "OFF" => LevelFilter::Off,
                "ERROR" => LevelFilter::Error,
                "WARN" => LevelFilter::Warn,
                "INFO" => LevelFilter::Info,
                "DEBUG" => LevelFilter::Debug,
                "TRACE" => LevelFilter::Trace,
                _ => LevelFilter::Info,
            }
            None => LevelFilter::Info,
        }
    }

    pub fn to_file(&self) -> bool {
        self.to_file.unwrap_or(false)
    }

    pub fn to_terminal(&self) -> bool {
        self.to_terminal.unwrap_or(true)
    }

    pub fn path(&self) -> PathBuf {
        let mut log_path = home_dir().unwrap();
        if self.log_path.is_none() {
            log_path.push(get_config_dir());
            log_path.push("log");
            return log_path;
        } else {
            log_path.push(self.log_path.clone().unwrap());
            return log_path;
        }
    }
}

/// The setting/config are saved
impl WriteConfig for Config {
    fn write(&self) {
        use std::io::Write;
        let path_config_file = path_config_file(CONFIG_FILE);

        let mut config_file = File::create(&path_config_file).expect(&format!("Failed at creating a template config file '{}'", &path_config_file.to_str().unwrap()));

        let toml = toml::to_string(self).unwrap();
        config_file.write_all(toml.as_bytes()).expect(&format!("Failed to create a config file"));

        error!("Edit the config file '{}'", &path_config_file.to_str().unwrap());
    }
}

/// Create a example/default configuration
fn config_template() -> Config {
    Config {
        plugin_path: Some(String::from(format!("{}/libs", get_config_dir()))),
        plugin_config_path: Some(String::from(format!("{}/plugins", get_config_dir()))),
        slack: Slack {
            api_token: "zzzz-xxxxxxxxxxxx-yyyyyyyyyyyyyyyyyyyyyyyy".to_string(),
            admin_api_token: "zzzz-xxxxxxxxxxx-yyyyyyyyyyy-aaaaaaaaaaaa-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            incoming_webhooks_token: None,
            outgoing_webhooks_token: None,
        },
        log: Some(Log {
            level: Some(String::from("info")),
            to_file: Some(false),
            to_terminal: Some(true),
            log_path: None,
        }),
    }
}
