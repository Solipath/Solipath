use serde::Deserialize;
use std::collections::HashMap;

use crate::solipath_platform::platform::Platform;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Template {
    name: String,
    #[serde(default = "default_variables")]
    variables: HashMap<String, String>,
    #[serde(default = "default_platform_filters")]
    platform_filters: Vec<Platform>,
}

impl Template {
    pub fn get_platform_filters(&self) -> &[Platform] {
        &self.platform_filters
    }
    pub fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

fn default_platform_filters() -> Vec<Platform> {
    Vec::new()
}

fn default_variables() -> HashMap<String, String> {
    HashMap::new()
}
