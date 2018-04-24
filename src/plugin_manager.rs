use config::DEFAULT;
use lib::{Symbol, Library, Result};
use template;
use std::ffi::OsStr;
use std::rc::Rc;


pub type PluginTypeV1 = Rc<Box<template::plugin_api_v1::Plugin>>;
pub type PluginFuncV1 = unsafe fn() -> *mut template::plugin_api_v1::Plugin;

pub struct PluginApiV1 {
    pub plugin: PluginTypeV1,
    pub loaded_libraries: Library,
}

pub struct PluginManager {
    plugins_api_1: Vec<PluginApiV1>,
}

impl PluginManager {
    /// Create Plugin Manager object
    pub fn new() -> PluginManager {
        PluginManager {
            plugins_api_1: Vec::new(),
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
            let constructor: Symbol<PluginFuncV1> = lib.get(b"load\0")
                .expect("The `load` symbol wasn't found.");
            let boxed_raw = constructor();
            Box::from_raw(boxed_raw)
        };

        info!("Loaded plugin: {}", obj.name());

        // makes the first call after api object is loaded. This is the only call there the object can be modified my the plugin itself
        (&mut obj).on_plugin_load(
            template::plugin_api_v1::Slack {
                api_token: DEFAULT.slack.api_token.clone(),
                admin_api_token: DEFAULT.slack.admin_api_token.clone()
            },
            DEFAULT.plugin_config_path().clone()
        );

        // Both the api object pointer and the Library object are saved for later use.
        // The api object pointer are used for communicating with the plugin.
        // The Library object are saved to preserve it lifetime, because then the plugin is dropped
        // the plugin is unloaded and can no longer be called (BEST-Bot will crash if a call is make
        // to the plugin after the plugin has been unloaded).
        self.plugins_api_1.push(PluginApiV1 {
                plugin: Rc::new(obj),
                loaded_libraries: lib,
        });

//        Ok(())
    }

    /// returns a list of all plugins using api v1
    pub fn list_of_api_v1_plugins(&self) -> &Vec<PluginApiV1> {
        &self.plugins_api_1
    }

    /// figure out what api version the plugin uses and then loads the plugin
    pub fn load_plugin<P: AsRef<OsStr>>(&mut self, filename: P) {
        let lib = Library::new(filename.as_ref()).expect("Unable to load the plugin");

        let version = self.api_version(&lib);
        if version.is_ok() {
            let version = version.unwrap();
            match version {
                1 => self.load_plugin_api_v1(lib),
                _ => error!("The api version {} is not supported", &version),
            }
        } else {
            error!("The plugin '{}' does not work. Error: '{:?}'", &filename.as_ref().to_str().unwrap(), version.err().unwrap())
        }
    }
}
