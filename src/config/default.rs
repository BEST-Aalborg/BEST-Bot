use super::*;

use std::collections::BTreeMap;

static CONFIG_FILE: &'static str = "default.toml";

lazy_static! {
    pub static ref CONFIG: Config = {
        read_config(CONFIG_FILE, config_template())
    };
}

fn read_config(config_file: &str, config: Config) -> Config {
    let mut config_file = init(config_file, config);

    let mut data = String::new();
    config_file.read_to_string(&mut data);
    error_handler(toml::from_str(&data))
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub slack: Slack,
    pub channels: Option<BTreeMap<String, Channel>>,
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Slack {
    pub api_token: String,
    pub admin_api_token: String,
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Channel {
    pub plugins: Vec<String>,
}

impl WriteConfig for Config {
    fn write(&self) {
        let path_config_file = path_config_file(CONFIG_FILE);

        let mut config_file = File::create(&path_config_file).expect(&format!("Failed at creating a template config file '{}'", &path_config_file.to_str().unwrap()));

        let toml = toml::to_string(self).unwrap();
        config_file.write_all(toml.as_bytes()).expect(&format!("Failed to create a config file"));

        println!("Edit the config file '{}'", &path_config_file.to_str().unwrap());
    }
}

fn config_template() -> Config {
    Config {
        slack: Slack {
            api_token: "zzzz-xxxxxxxxxxxx-yyyyyyyyyyyyyyyyyyyyyyyy".to_string(),
            admin_api_token: "zzzz-xxxxxxxxxxx-yyyyyyyyyyy-aaaaaaaaaaaa-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        },
        channels: Some({
            let mut channels = BTreeMap::new();
            channels.insert("#test".to_string(), Channel { plugins: vec!["block_posts".to_string()] });
            channels
        })
    }
}
