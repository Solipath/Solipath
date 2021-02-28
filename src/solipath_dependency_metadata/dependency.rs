use serde::Deserialize;

use crate::solipath_platform::platform::Platform;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    #[serde(default = "default_platform_filters")]
    platform_filters: Vec<Platform>,
}

impl Dependency {
    #[allow(dead_code)]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            platform_filters: Vec::new(),
        }
    }

    pub fn get_platform_filters(&self) -> &[Platform] {
        &self.platform_filters
    }
}

fn default_platform_filters() -> Vec<Platform> {
    Vec::new()
}
