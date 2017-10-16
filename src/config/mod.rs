extern crate serde;

pub mod default;
pub use self::default::CONFIG as DEFAULT;

use std::env::home_dir;
use std::fs::{create_dir_all,File};
#[allow(unused_imports)]
use std::io::{Read,Write};
use std::path::{PathBuf};

use toml;

use self::serde::Deserialize;
use self::serde::Serialize;

static CONFIG_FOLDER: &'static str = ".config/BEST-Bot";

fn init<'de, T>(filename: &str, config: T) -> File where T: Deserialize<'de> + Serialize + WriteConfig {
    let path_config_file = path_config_file(filename);
    let config_file: File;

    let path_config_dir = path_config_dir();
    create_dir_all(path_config_dir.as_path()).expect(&format!("Failed to create the directory '{}'", path_config_file.to_str().unwrap()));

    if path_config_file.exists() {
        if path_config_file.is_file() {
            config_file = File::open(&path_config_file).expect(&format!("unable to open the config file '{}'", path_config_file.to_str().unwrap()));
        } else {
            panic!("Cannot access the config file '{}'", path_config_file.to_str().unwrap());
        }
    } else {
        config.write();
        ::std::process::exit(1);
    }

    config_file
}

fn path_config_dir() -> PathBuf {
    let mut path_config_dir = home_dir().unwrap();
    path_config_dir.push(CONFIG_FOLDER);
    path_config_dir
}

fn path_config_file(filename: &str) -> PathBuf {
    let mut path_config_file = path_config_dir();
    path_config_file.push(filename);
    path_config_file
}

trait WriteConfig {
    fn write(&self);
}