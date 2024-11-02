use serde::Deserialize;

use crate::solipath_platform::{platform::Platform, platform_filter::HasPlatformFilter};

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct EnvironmentVariable {
    name: String,
    relative_path: Option<String>,
    value: Option<String>,
    #[serde(default = "default_platform_filters")]
    platform_filters: Vec<Platform>,
}

impl EnvironmentVariable {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_relative_path(&self) -> &Option<String> {
        &self.relative_path
    }

    pub fn get_value(&self) -> &Option<String> {
        &self.value
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

#[cfg(test)]
mod tests {
    use super::EnvironmentVariable;

    #[test]
    fn can_set_only_relative_path() {
        let environment_variable = serde_json::from_str::<EnvironmentVariable>(
            r#"{"name": "RUST_TEST", "relative_path": "some/path/location"}"#,
        )
        .unwrap();
        
        assert_eq!(Some("some/path/location".to_string()), environment_variable.get_relative_path().clone());
        assert_eq!(None, environment_variable.get_value().clone());
    }
    
    #[test]
    fn can_set_only_value() {
        let environment_variable = serde_json::from_str::<EnvironmentVariable>(
            r#"{"name": "RUST_TEST", "value": "the value"}"#,
        )
        .unwrap();
    
        assert_eq!(None, environment_variable.get_relative_path().clone());
        assert_eq!(Some("the value".to_string()), environment_variable.get_value().clone());
    }
}
