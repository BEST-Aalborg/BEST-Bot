#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate template;
extern crate libloading as lib;

#[macro_use]
extern crate log;
extern crate simple_logging;
mod config;

use config::CONFIG;
mod misc;

mod plugin_manager;

use plugin_manager::*;
mod slack_bot;

use slack_bot::MyHandler;
use slack_bot::MyEventHandler;

use std::io;
use std::process::exit;
use std::fs::create_dir_all;

use template::slack::Error as sError;

fn main() {
    if CONFIG.log().to_file() {
        let log = CONFIG.log();
        let mut path = log.path();
        create_dir_all(path.as_path()).expect(&format!("Failed to create the directory '{}'", path.to_str().unwrap()));
        path.push("BEST-Bot.log");
        simple_logging::log_to_file(path.as_path(), log.level());
    } else {
        simple_logging::log_to(io::stdout(), CONFIG.log().level());
    }

    // Init Plugin Manager
    let mut plugin_manager = PluginManager::new();

    info!("Looking for plugins in the folder {:?}", CONFIG.plugin_path());
    for path in misc::find_plugins() {
        plugin_manager.load_plugin(path);
    }

    // Init Slack Bot Handler
    let mut handler = MyHandler::new();


    plugin_api_v1(&plugin_manager, &mut handler);


    let _loop = true;
    while _loop {
        match handler.init() {
            Ok(_) => exit(0),
            Err(error) => {
                match error {
                    sError::WebSocket(ws_error) => warn!("WebSocket -> '{:?}'", ws_error),
                    _ => error!("Unknown -> '{:?}'", error)
                }
            }
        }
    }
}

/// Get a list of all the plugins using api v1 and the list of events they are subscript to and adds them to these events
fn plugin_api_v1(plugin_manager: &PluginManager, handler: &mut MyHandler) {
    for _plugin in plugin_manager.list_of_api_v1_plugins() {
        handler.subscript_to(_plugin);
    }
}

