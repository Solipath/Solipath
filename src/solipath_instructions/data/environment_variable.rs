use serde::Deserialize;

use crate::solipath_platform::{platform::Platform, platform_filter::HasPlatformFilter};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EnvironmentVariable {
    name: String,
    relative_path: String,
    #[serde(default = "default_platform_filters")]
    platform_filters: Vec<Platform>,
}

impl EnvironmentVariable {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_relative_path(&self) -> String {
        self.relative_path.clone()
    }
    
}

impl HasPlatformFilter for EnvironmentVariable{
    fn get_platform_filters(&self) -> &[Platform] {
        &self.platform_filters
    }
}
fn default_platform_filters() -> Vec<Platform> {
    Vec::new()
}
