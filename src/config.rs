
use std::{fs, path::Path};

use serde::{Serialize, Deserialize};





#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpiderTuiConfig{
    #[serde(default = "default_log_path")]
    pub log_path: String,
    #[serde(default = "default_state_data_path")]
    pub state_data_path: String,

    #[serde(default = "keyfile_path")]
    pub keyfile_path: String,

}


impl SpiderTuiConfig {
    pub fn from_file(path: &Path) -> Self {
        let data = match fs::read_to_string(&path){
            Ok(str) => str,
            Err(_) => String::from("{}"),
        };
        // let data = fs::read_to_string(&path).expect(&format!("Failed to read config file: {:?}", path));
		let config = serde_json::from_str(&data).expect("Failed to deserialize config");
        config
    }
}




// Defaults
fn default_log_path() -> String {
    "spider_tui.log".into()
}

fn default_state_data_path() -> String {
    "client_state.dat".into()
}

fn keyfile_path() -> String {
    "spider_keyfile.json".into()
}