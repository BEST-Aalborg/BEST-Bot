#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate libloading as lib;

#[macro_use]
extern crate log;
extern crate simple_logging;

extern crate template;
use template::slack::Error as sError;
use template::channel_return::unbounded;

mod config;
use config::CONFIG;

mod logger;

mod misc;

mod plugin_manager;
use plugin_manager::*;

mod slack_bot;
use slack_bot::MyHandler;
use slack_bot::MyEventHandler;

use std::process::exit;

fn main() {
    let logger_sender = logger::init().expect("BEST-Bot failed at starting the logging module");

    let (plugin_sender, plugin_receiver) = unbounded::<template::plugin_api_v2::Channel>();

    // Init Slack Bot Handler
    let mut handler = MyHandler::new(plugin_receiver);

    // Init Plugin Manager
    let mut plugin_manager = PluginManager::new(logger_sender, plugin_sender);

    info!("Looking for plugins in the folder {:?}", CONFIG.plugin_path());
    for path in misc::find_plugins() {
                plugin_manager.load_plugin(path);
    }

    plugin_api_v1(&plugin_manager, &mut handler);
    plugin_api_v2(&plugin_manager, &mut handler);


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
                handler.subscript_to_v1(_plugin);
    }
}

/// Get a list of all the plugins using api v2 and the list of events they are subscript to and adds them to these events
fn plugin_api_v2(plugin_manager: &PluginManager, handler: &mut MyHandler) {
    for _plugin in plugin_manager.list_of_api_v2_plugins() {
        handler.subscript_to_v2(_plugin);
    }
}
