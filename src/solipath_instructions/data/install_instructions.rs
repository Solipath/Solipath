use crate::solipath_instructions::data::download_instruction::DownloadInstruction;
use crate::solipath_instructions::data::environment_variable::EnvironmentVariable;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct InstallInstructions {
    #[serde(default = "default_downloads")]
    downloads: Vec<DownloadInstruction>,

    #[serde(default = "default_environment_variable")]
    environment_variables: Vec<EnvironmentVariable>,
}

fn default_downloads() -> Vec<DownloadInstruction> {
    Vec::new()
}

fn default_environment_variable() -> Vec<EnvironmentVariable> {
    Vec::new()
}

impl InstallInstructions {
    pub fn get_downloads(&self) -> Vec<DownloadInstruction> {
        self.downloads.clone()
    }

    pub fn get_environment_variables(&self) -> Vec<EnvironmentVariable> {
        self.environment_variables.clone()
    }
}
