#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate template;
extern crate libloading as lib;

#[macro_use]
extern crate log;
extern crate simple_logging;
use log::LevelFilter;
use std::io;

mod config;
use config::DEFAULT;
mod misc;

mod plugin_manager;
use plugin_manager::*;

mod slack_bot;
use slack_bot::MyHandler;
use slack_bot::MyEventHandler;

use std::process::exit;

fn main() {
    // Hardcoded logging level to Info. Other options are "Off", "Error", "Warn", "Debug" and "Trace"
    // TODO: Make it possible to specify logging level from config
    simple_logging::log_to(io::stdout(), LevelFilter::Info);

    // Init Plugin Manager
    let mut plugin_manager = PluginManager::new();

    info!("Looking for plugins in the folder {:?}", DEFAULT.plugin_path());
    for path in misc::find_plugins() {
        plugin_manager.load_plugin(path);
    }

    // Init Slack Bot Handler
    let mut handler = MyHandler::new();

    
    plugin_api_v1(&plugin_manager, &mut handler);

    match handler.init() {
        Ok(_) => exit(0),
        Err(e) => {
            error!("{}", e);
        }
    }
}

/// Get a list of all the plugins using api v1 and the list of events they are subscript to and adds them to these events
fn plugin_api_v1(plugin_manager: &PluginManager, handler: &mut MyHandler) {
    for _plugin in plugin_manager.list_of_api_v1_plugins() {
        handler.subscript_to(_plugin);
    }
}

