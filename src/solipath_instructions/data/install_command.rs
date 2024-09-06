use std::collections::HashMap;

use serde::Deserialize;

use crate::solipath_platform::platform::Platform;


#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct InstallCommand {
    command: String,
    #[serde(default = "default_platform_filters")]
    platform_filters: Vec<Platform>,
    
    #[serde(default = "default_when_to_run_rules")]
    when_to_run_rules: HashMap<String, serde_json::Value>
}

impl InstallCommand {
    pub fn get_command(&self)-> String {
        self.command.to_string()
    }

    pub fn get_platform_filters(&self) -> &[Platform] {
        &self.platform_filters
    }

    pub fn get_when_to_run_rules(&self) -> &HashMap<String, serde_json::Value> {
        &self.when_to_run_rules
    }
}


fn default_platform_filters() -> Vec<Platform> {
    Vec::new()
}

fn default_when_to_run_rules() -> HashMap<String, serde_json::Value> {
    HashMap::new()
}