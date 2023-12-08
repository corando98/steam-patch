use lazy_static::lazy_static;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::{sync::Mutex, fs, path::PathBuf};

use crate::utils::get_username;
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_main_enabled")]
    pub main_enabled: bool,
    #[serde(default = "default_tdp_control")]
    pub tdp_control: bool,
    #[serde(default = "default_gpu_control")]
    pub gpu_control: bool,
    #[serde(default = "default_max_tdp")]
    pub max_tdp: i8,
    #[serde(default = "default_max_gpu")]
    pub max_gpu: i16,
    #[serde(default = "default_mapper")]
    pub mapper: bool,
    #[serde(default = "default_legacy_tdp")]
    pub legacy_tdp: bool,
    #[serde(default = "default_auto_nkey_recovery")]
    pub auto_nkey_recovery: bool,
    #[serde(default = "default_spoof_glyphs")]
    pub spoof_glyphs: bool,
    #[serde(default = "default_nintendo_glyphs")]
    pub nintendo_glyphs: bool,
}

// Default functions for each field
fn default_main_enabled() -> bool { false }
fn default_tdp_control() -> bool { true }
fn default_gpu_control() -> bool { true }
fn default_max_tdp() -> i8 { 15 }
fn default_max_gpu() -> i16 { 2000 }
fn default_mapper() -> bool { false }
fn default_legacy_tdp() -> bool { true }
fn default_auto_nkey_recovery() -> bool { false }
fn default_spoof_glyphs() -> bool { true }
fn default_nintendo_glyphs() -> bool { false }




lazy_static! {
    pub static ref CONFIG: Mutex<Option<Config>> = Mutex::new(None);
}

pub fn initialize_config() -> Config {
    let mut global_config = CONFIG.lock().unwrap();
    match read_config() {
        Ok(config) => {
            println!("CONFIG: Found config file!");
            *global_config = Some(config.clone()); // Clone the config for local use
            // *global_config = Some(config);
            config // Return the cloned config
        }
        Err(e) => {
            println!("CONFIG: Failed to read config: {}", e);
            // Handle error, perhaps by setting default values or terminating the application

            let default_config = Config {
                main_enabled: false, // Default values
                tdp_control: true,
                gpu_control: true,
                max_tdp: 15,
                max_gpu: 2000,
                mapper: false,
                legacy_tdp: true,
                auto_nkey_recovery: false,
                spoof_glyphs: true,
                nintendo_glyphs: false
            };
            *global_config = Some(default_config.clone());
            default_config
        }
    }
}
fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    //Read the file into a string
    let username = get_username();
    println!("Grabbed config user: {}", username);
    let config_path = PathBuf::from(format!("/home/{}/steam-patch/config.toml", username));
    println!("Current path {:?}", config_path);
    let contents = fs::read_to_string(config_path)?;
    //Parse string of data into config
    let config: Config = toml::from_str(&contents)?;
    println!("{:?}",config);
    Ok(config)
}
pub fn get_global_config() -> Config {
    CONFIG.lock().unwrap().clone().expect("Config should be init\'d")
}