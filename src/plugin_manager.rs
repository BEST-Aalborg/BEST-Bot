use config::CONFIG;
use lib::{Symbol, Library, Result};
use std::ffi::OsStr;
use std::sync::Arc;

use template::Name;
use template::logger::LoggerSender;
use template::plugin_api_v1;
use template::plugin_api_v2;

pub type RefCounter<T> = Arc<T>;
pub type PluginType<V> = RefCounter<Box<V>>;
pub type PluginFunc<V> = unsafe fn() -> *mut V;

pub struct PluginApi<V: ?Sized> {
    pub plugin: PluginType<V>,
    pub loaded_libraries: Library,
}

pub struct PluginManager {
    logger_sender: LoggerSender,
    plugin_sender: plugin_api_v2::Sender,
    plugins_api_1: Vec<PluginApi<plugin_api_v1::Plugin>>,
    plugins_api_2: Vec<PluginApi<plugin_api_v2::Plugin>>,
}

impl PluginManager {
    /// Create Plugin Manager object
    pub fn new(logger_sender: LoggerSender, plugin_sender: plugin_api_v2::Sender) -> PluginManager {
        PluginManager {
            logger_sender: logger_sender,
            plugin_sender: plugin_sender,
            plugins_api_1: Vec::new(),
            plugins_api_2: Vec::new(),
        }
    }

    /// Calls the plugin required function "api_version".
    /// If this call fails this library should not be loaded it is impossible to know the api version and the library is probably not a plugin for BEST-Bot
    fn api_version(&self, lib: &Library) -> Result<u32> {
        unsafe {
            let func: Symbol<unsafe extern "C" fn() -> u32> = lib.get(b"api_version\0")?;
            Ok(func())
        }
    }

    /// Loads plugins using API v1
    fn load_plugin_api_v1(&mut self, lib: Library) {
        let mut obj = unsafe {
            // TODO: Make it so that a error is written to the log, instead of stopping the program if the function "load" is not present.
            let constructor: Symbol<PluginFunc<plugin_api_v1::Plugin>> = lib.get(b"load\0")
                .expect("The `load` symbol wasn't found.");
            let boxed_raw = constructor();
            Box::from_raw(boxed_raw)
        };
        info!("Loaded plugin: {}", obj.name());

        // makes the first call after api object is loaded. This is the only call there the object can be modified my the plugin itself
        (&mut obj).on_plugin_load(
            plugin_api_v1::Slack {
                api_token: CONFIG.slack.api_token.clone(),
                admin_api_token: CONFIG.slack.admin_api_token.clone()
            },
            CONFIG.plugin_config_path().clone()
        );

        // Both the api object pointer and the Library object are saved for later use.
        // The api object pointer are used for communicating with the plugin.
        // The Library object are saved to preserve it lifetime, because then the plugin is dropped
        // the plugin is unloaded and can no longer be called (BEST-Bot will crash if a call is make
        // to the plugin after the plugin has been unloaded).
        self.plugins_api_1.push(PluginApi::<plugin_api_v1::Plugin> {
            plugin: RefCounter::new(obj),
            loaded_libraries: lib,
        });

//        Ok(())
    }

    /// Loads plugins using API v2
    fn load_plugin_api_v2(&mut self, lib: Library) {
        let mut obj = load::<plugin_api_v2::Plugin>(&lib);

        // makes the first call after api object is loaded. This is the only call there the object can be modified my the plugin itself
        (&mut obj).on_plugin_load(
            self.logger_sender.clone(),
            self.plugin_sender.clone()
        );

        // Both the api object pointer and the Library object are saved for later use.
        // The api object pointer are used for communicating with the plugin.
        // The Library object are saved to preserve it lifetime, because then the plugin is dropped
        // the plugin is unloaded and can no longer be called (BEST-Bot will crash if a call is make
        // to the plugin after the plugin has been unloaded).
        self.plugins_api_2.push(PluginApi::<plugin_api_v2::Plugin> {
            plugin: RefCounter::new(obj),
            loaded_libraries: lib,
        });

//        Ok(())
    }

    /// returns a list of all plugins using api v1
    pub fn list_of_api_v1_plugins(&self) -> &Vec<PluginApi<plugin_api_v1::Plugin>> {
        &self.plugins_api_1
    }

    /// returns a list of all plugins using api v2
    pub fn list_of_api_v2_plugins(&self) -> &Vec<PluginApi<plugin_api_v2::Plugin>> {
        &self.plugins_api_2
    }

    /// figure out what api version the plugin uses and then loads the plugin
    pub fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) {
                let lib = Library::new(filename.as_ref()).expect("Unable to load the plugin");

                let version = self.api_version(&lib);
        if version.is_ok() {
            let version = version.unwrap();
            match version {
                1 => self.load_plugin_api_v1(lib),
                2 => self.load_plugin_api_v2(lib),
                _ => error!("The api version {} is not supported", &version),
            }
        } else {
            error!("The plugin '{}' does not work. Error: '{:?}'", &filename.as_ref().to_str().unwrap(), version.err().unwrap())
        }
    }
}

fn load<T: ?Sized + Name>(lib: &Library) -> Box<T> {
    let obj = unsafe {
        // TODO: Make it so that a error is written to the log, instead of stopping the program if the function "load" is not present.
        let constructor: Symbol<PluginFunc<T>> = lib.get(b"load\0")
            .expect("The `load` symbol wasn't found.");
        let boxed_raw = constructor();
        Box::from_raw(boxed_raw)
    };
    info!("Loaded plugin v2: {}", obj.name());
    obj
}