use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::exit;

use config::CONFIG;

/// Creates a list of all plugins present in the plugin folder
pub fn find_plugins() -> Vec<PathBuf> {
    let plugin_dir = CONFIG.plugin_path();
    let mut plugins = Vec::new();

    // Checks if the path exist and that it is a folder
    if plugin_dir.exists() & plugin_dir.is_dir() {

        // Get a list of all the files in the plugin folder
        match plugin_dir.read_dir() {
            Err(e) => {
                error!("{:?}", e);
                exit(1);
            },
            Ok(paths) => {
                for path in paths {
                    // get the plugins file name
                    let plugin = path.as_ref().unwrap().file_name();
                    let plugin = plugin.to_str();

                    if plugin.is_none() {
                        error!("Something is wrong with the file name '{}'", path.as_ref().unwrap().file_name().to_string_lossy());
                    } else {
                        let plugin = plugin.unwrap();

                        // if the file ends with ".so" it is assumed that it is a library/plugin for BEST-Bot
                        if plugin.ends_with(".so") {
                            // Add path of the plugin to the list of plugins
                            plugins.push(path.unwrap().path());
                        }
                    }
                }
            }
        }
    } else {
        error!("Cannot find the folder '{}' and will try to create it", plugin_dir.to_str().unwrap());
        create_dir_all(&plugin_dir).expect(&format!("Cannot create the folder '{}'", plugin_dir.to_str().unwrap()));
        exit(1);
    }

    plugins
}