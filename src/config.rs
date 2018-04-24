extern crate easy_toml_config;
use self::easy_toml_config::*;

use template::plugin_api_v1::Slack;

use std::path::{PathBuf};
use std::fs::File;
use std::env::home_dir;

static CONFIG_FILE: &'static str = "default.toml";


lazy_static! {
    pub static ref DEFAULT: Config = {
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
        },
    }
}
