use serde::Deserialize;

use crate::solipath_platform::{platform::Platform, platform_filter::HasPlatformFilter};
#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DownloadInstruction {
    url: String,
    destination_directory: String,
    #[serde(default = "default_platform_filters")]
    platform_filters: Vec<Platform>,
}

impl DownloadInstruction {
    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_destination_directory(&self) -> String {
        self.destination_directory.clone()
    }

}
fn default_platform_filters() -> Vec<Platform> {
    Vec::new()
}

impl HasPlatformFilter for DownloadInstruction {
    fn get_platform_filters(&self) -> &[Platform] {
        &self.platform_filters
    }
}
